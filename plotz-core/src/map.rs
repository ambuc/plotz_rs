//! The core logic of plotz-core, including Map and MapConfig.

#![allow(clippy::let_unit_value)]

use crate::{
    bucket::{Area, Bucket, Highway, Path as BucketPath, Subway},
    bucketer::{Bucketer2, DefaultBucketer2},
    canvas::Canvas,
    frame::make_frame,
    svg::Size,
};
use anyhow::Result;
use float_ord::FloatOrd;
use itertools::Itertools;
use lazy_static::lazy_static;
use plotz_color::{subway::*, *};
use plotz_geometry::{
    bounded::Bounded,
    crop::Croppable,
    obj::Obj2,
    shading::{shade_config::ShadeConfig, shade_polygon},
    shapes::{point::Point, polygon::Polygon, segment::Segment},
    style::Style,
};
use rand::{thread_rng, Rng};
use std::{
    cmp::Ord,
    collections::HashSet,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};
use tracing::*;
use typed_builder::TypedBuilder;

#[derive(Debug)]
/// A polygon with some annotations (bucket, color, tags, etc.).
pub struct AnnotatedObject2d {
    obj: Obj2,
    style: Style,
    bucket: Bucket,
    _tags: Vec<(String, String)>,
}
impl AnnotatedObject2d {
    /// Consumes an AnnotatedPolygon and casts down to a ColoredPolygon.
    pub fn to_object2d(self) -> (Obj2, Style) {
        (self.obj, self.style)
    }
}

fn latitude_to_y(latitude: f64) -> f64 {
    use std::f64::consts::PI;
    (((latitude + 90.0) / 360.0 * PI).tan()).ln() / PI * 180.0
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Whether to render the inner shading and/or the outer edge of a thing.
pub enum ShadeAndOutline {
    /// Just the inner shading.
    JustShade,
    /// Both the inner shading and the outer edge.
    Both,
}

fn map_bucket_to_color(bucket: &Bucket) -> Option<ColorRGB> {
    match bucket {
        Bucket::Frame => Some(BLACK),

        Bucket::Color(c) => Some(*c),

        Bucket::Area(area) => match area {
            Area::Beach => Some(TAN),
            Area::Fun => Some(LIGHTCYAN),
            Area::NaturalRock => Some(DARKGRAY),
            Area::Land => Some(PINK),
            Area::Park => Some(GREEN),
            Area::Parking => Some(LIGHTCORAL),
            Area::Water => Some(LIGHTBLUE),
            Area::Building => Some(LIGHTGRAY),
            Area::Rail | Area::Tree => Some(LIGHTPINK),
        },

        Bucket::Path(path) => match path {
            BucketPath::Subway(subway) => match subway {
                Subway::_123 => Some(RED_123),
                Subway::_456 => Some(GREEN_456),
                Subway::_7 => Some(PURPLE_7),
                Subway::_ACE => Some(BLUE_ACE),
                Subway::_BDFM => Some(ORANGE_BDFM),
                Subway::_G => Some(LIME_G),
                Subway::_JZ => Some(BROWN_JZ),
                Subway::_L => Some(GREY_L),
                Subway::_NQRW => Some(YELLOW_NQRW),
                Subway::_S => Some(GRAY_S),
                Subway::_T => Some(TEAL_T),
                Subway::Other => None,
            },
            BucketPath::Highway(highway) => match highway {
                Highway::Primary | Highway::PrimaryLink => Some(ORANGE),
                Highway::Secondary
                | Highway::SecondaryLink
                | Highway::Tertiary
                | Highway::TertiaryLink => Some(ORANGE),
                Highway::Elevator | Highway::MotorwayLink => Some(ORANGERED),
                Highway::Track | Highway::Unclassified => Some(LIGHTGOLDENROD),
                Highway::RoadMarking => Some(LIGHTSALMON),
                Highway::Service => Some(LIGHTPINK),
                Highway::Road | Highway::Path | Highway::Platform => Some(LIGHTSTEELBLUE),
            },

            BucketPath::Bridge => Some(AQUAMARINE),
            BucketPath::Pedestrian => Some(LIGHTGRAY),

            BucketPath::Bus => Some(WHEAT),
            BucketPath::Barrier => Some(PINK),
            BucketPath::Cycleway => Some(LIGHTGREEN),
            BucketPath::Rail => Some(LIGHTYELLOW),
            BucketPath::Cable => Some(LIMEGREEN),
            //
            BucketPath::Boundary => None,
        },
    }
}

fn map_bucket_to_shadeconfig(bucket: &Bucket) -> Option<(ShadeAndOutline, ShadeConfig)> {
    let gap = 2.0;
    match bucket {
        Bucket::Area(Area::Land) => Some((
            ShadeAndOutline::JustShade,
            ShadeConfig::builder().gap(gap).slope(-2.0).build(),
        )),
        Bucket::Area(Area::Building) => Some((
            ShadeAndOutline::JustShade,
            ShadeConfig::builder().gap(3.0).slope(1.0).build(),
        )),
        Bucket::Area(Area::Parking) => Some((
            ShadeAndOutline::JustShade,
            ShadeConfig::builder().gap(gap).slope(-1.0).build(),
        )),
        Bucket::Area(Area::Park) => Some((
            ShadeAndOutline::JustShade,
            ShadeConfig::builder().gap(gap).slope(2.0).build(),
        )),
        Bucket::Area(Area::Water) => Some((
            ShadeAndOutline::JustShade,
            ShadeConfig::builder().gap(1.5).slope(0.0).build(),
        )),
        Bucket::Area(Area::Fun) => Some((
            ShadeAndOutline::JustShade,
            ShadeConfig::builder().gap(gap).slope(3.0).build(),
        )),
        _ => None,
    }
}

lazy_static! {
    /// How thick the default line is.
    pub static ref DEFAULT_THICKNESS: f64 = 1.0;
}

/// An unadjusted set of annotated polygons, ready to be printed to SVG.
#[derive(Debug)]
pub struct Map {
    canvas: Canvas,

    // user-configurable, there might be a desired pt to put at the center of the output.
    center: Option<Point>,
}
impl Map {
    /// Consumes MapConfig, performs bucketing and coloring, and returns an
    /// unadjusted Map instance.
    pub fn new(map_config: &MapConfig, center: Option<Point>) -> Result<Map> {
        let bucketer = DefaultBucketer2 {};

        let mut canvas = Canvas::default();

        map_config
            .input_files
            .iter()
            .map(|file| {
                trace!("processing file: {:?}", file);
                Ok(plotz_geojson::parse_geojson(
                    serde_json::from_reader(BufReader::new(file)).expect("read"),
                )?
                .iter()
                .flat_map(|(obj_inner, tags)| {
                    bucketer.bucket(tags).into_iter().flat_map(|bucket| {
                        map_bucket_to_color(&bucket).map(|color| AnnotatedObject2d {
                            obj: obj_inner.clone(),
                            style: Style {
                                color,
                                thickness: *DEFAULT_THICKNESS,
                                ..Default::default()
                            },
                            bucket,
                            _tags: tags.clone(),
                        })
                    })
                })
                .collect::<Vec<AnnotatedObject2d>>())
            })
            .flatten_ok()
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .sorted_by(|ap_1, ap_2| Ord::cmp(&ap_1.bucket, &ap_2.bucket))
            .group_by(|ap| ap.bucket)
            .into_iter()
            .for_each(|(bucket, v)| {
                canvas
                    .dos_by_bucket
                    .entry(Some(bucket))
                    .or_default()
                    .extend(v.into_iter().map(|v| v.to_object2d()));
            });

        trace!("made {:?} layers", canvas.dos_by_bucket.len());

        Ok(Map { canvas, center })
    }

    fn adjust_flip_y(&mut self) {
        // flip all points across the y axis.
        self.canvas.mutate_all(|pt| {
            pt.flip_y();
        });

        if let Some(center) = &mut self.center {
            center.flip_y();
        }
    }

    fn adjust_latitude_transform(&mut self) {
        // adjust all point y values according to latitude transform
        self.canvas.mutate_all(|pt| {
            pt.y = latitude_to_y(pt.y);
        });
        if let Some(center) = &mut self.center {
            center.y = latitude_to_y(center.y);
        }
    }

    fn adjust_bl_shift(&mut self) -> Result<()> {
        let canvas_bounds = self.canvas.bounds()?;
        self.canvas.mutate_all(|pt| {
            *pt -= canvas_bounds.x_min_y_min();
        });
        if let Some(center) = &mut self.center {
            *center -= canvas_bounds.x_min_y_min();
        }
        Ok(())
    }

    fn adjust_centering(&mut self, dest_size: &Size) -> Result<()> {
        let shift = match self.center {
            Some(desired_center) => Point(
                dest_size.width as f64 / 2.0 - desired_center.x,
                dest_size.height as f64 / 2.0 - desired_center.y,
            ),
            None => {
                let canvas_bounds = self.canvas.bounds()?;
                Point(
                    (dest_size.width as f64 - canvas_bounds.x_max) / 2.0,
                    (dest_size.height as f64 - canvas_bounds.y_max) / 2.0,
                )
            }
        };
        self.canvas.mutate_all(|pt| *pt += shift);
        Ok(())
    }

    fn adjust_scaling(&mut self, scale_factor: f64, dest_size: &Size) -> Result<()> {
        let canvas_bounds = self.canvas.bounds()?;
        let scaling_factor = std::cmp::max(
            FloatOrd(dest_size.height as f64 / canvas_bounds.y_span().abs()),
            FloatOrd(dest_size.width as f64 / canvas_bounds.x_span().abs()),
        )
        .0 * scale_factor;
        self.canvas.mutate_all(|obj| {
            *obj *= scaling_factor;
        });

        if let Some(center) = &mut self.center {
            // something about lat long encoding here vs. there. oops
            *center *= scaling_factor;
        }

        Ok(())
    }

    fn apply_shading_to_objects(&mut self) {
        for (bucket, layers) in self.canvas.dos_by_bucket.iter_mut() {
            if let Some(bucket) = bucket {
                if let Some((shade_and_outline, ref shade_config)) =
                    map_bucket_to_shadeconfig(bucket)
                {
                    let mut v: Vec<(Obj2, Style)> = vec![];
                    // keep the frame, add the crosshatchings.
                    let crosshatchings: Vec<(Obj2, Style)> = layers
                        .iter()
                        .filter_map(|(obj, style)| match &obj {
                            Obj2::Polygon(p) => match shade_polygon(shade_config, p) {
                                Err(_) => None,
                                Ok(segments) => Some(
                                    segments
                                        .into_iter()
                                        .map(|s| {
                                            (
                                                Obj2::Segment(s),
                                                Style {
                                                    thickness: shade_config.thickness,
                                                    ..*style
                                                },
                                            )
                                        })
                                        .collect::<Vec<_>>(),
                                ),
                            },
                            _ => None,
                        })
                        .flatten()
                        .collect();
                    match shade_and_outline {
                        ShadeAndOutline::JustShade => {
                            v.extend(crosshatchings.iter().map(|so2| (so2.0.clone(), so2.1)));
                        }
                        ShadeAndOutline::Both => {
                            v.extend(crosshatchings.iter().map(|so2| (so2.0.clone(), so2.1)));
                            v.extend(layers.clone());
                        }
                    }
                    *layers = v;
                }
            }
        }
    }

    /// Adjusts the map for scale/transform issues.
    pub fn do_all_adjustments(&mut self, scale_factor: f64, dest_size: &Size) -> Result<()> {
        self.adjust_flip_y();
        self.adjust_latitude_transform();
        self.adjust_bl_shift()?;
        self.adjust_scaling(scale_factor, dest_size)?;
        self.adjust_centering(dest_size)?;
        Ok(())
    }

    /// For every circle in every layer, jog it by a random bit. Not sure this
    /// should stay.
    pub fn randomize_circles(&mut self) {
        for (_bucket, dos) in self.canvas.dos_by_bucket.iter_mut() {
            for (ref mut obj, _style) in dos.iter_mut() {
                if let Obj2::CurveArc(mut ca) = &obj {
                    ca.ctr += (
                        thread_rng().gen_range(-2.0..=2.0),
                        thread_rng().gen_range(-2.0..=2.0),
                    );
                }
            }
        }
    }

    /// Crop everything everywhere to the frame polygon. (Passed in here for
    /// Flexibility.)
    pub fn crop_to_frame(&mut self, frame: &Polygon) -> Result<()> {
        trace!("Cropping all to frame.");
        for (_bucket, dos) in self.canvas.dos_by_bucket.iter_mut() {
            *dos = dos
                .iter_mut()
                .map(|(obj, style)| Ok(obj.crop_to(frame)?.into_iter().map(|obj| (obj, *style))))
                .flatten_ok()
                .collect::<Result<Vec<_>>>()?;
        }
        Ok(())
    }

    /// For every polygon in every layer, replace it with segments with the same
    /// color. Useful for simplification. I think axicli can stitch these back
    /// together for fast plotting but I'm not sure.
    pub fn polygons_to_segments(&mut self) {
        trace!("Turning polygons into segments.");
        for (_bucket, dos) in self.canvas.dos_by_bucket.iter_mut() {
            *dos = dos
                .iter_mut()
                .flat_map(|(obj, style)| {
                    match obj.clone() {
                        Obj2::Polygon(pg) => pg.to_segments().into_iter().map(Obj2::from).collect(),
                        x => vec![x],
                    }
                    .into_iter()
                    .map(|inner| (inner, *style))
                })
                .collect();
        }
    }

    /// Simplifies the inner layers for faster, less repetitive plotting.
    /// Innards of this are subject to change. For now I think we want to
    /// segmentify things, quantize them, dedup them, ...? magic ..? and then
    /// restore one copy of the original unquantized version.
    pub fn simplify_layers(&mut self) {
        trace!("Simplifying layers.");
        self.polygons_to_segments(); // prereq

        for (bucket, dos) in self.canvas.dos_by_bucket.iter_mut() {
            // at this point there are no polygons, only segments.
            let color = bucket
                .map(|bucket| map_bucket_to_color(&bucket))
                .unwrap_or(Some(BLACK))
                .unwrap();
            let mut hs = HashSet::<Segment>::new();
            for (obj, _style) in dos.iter() {
                if let Obj2::Segment(sg) = obj {
                    hs.insert(*sg);
                    // TODO(ambuc): really, deduplicate this way but then store and restore the original.
                }
            }
            *dos = hs
                .into_iter()
                .map(|sg| {
                    (
                        Obj2::Segment(sg),
                        Style {
                            color,
                            ..Default::default()
                        },
                    )
                })
                .collect::<Vec<_>>();
        }
    }

    /// Consumes a Map, adjusts each polygon, and writes the results as SVG to
    /// file(s).
    pub fn render(mut self, config: &MapConfig) -> Result<()> {
        trace!(config = ?config.input_files);

        let () = self.do_all_adjustments(config.scale_factor, &config.size)?;

        // let () = self.randomize_circles();
        let () = self.apply_shading_to_objects();

        // self.simplify_layers();

        if config.draw_frame {
            info!("Adding frame.");
            let margin = 20.0;
            let frame = make_frame(
                // yes these are backwards. oops
                (
                    config.size.height as f64 - 2.0 * margin,
                    config.size.width as f64 - 2.0 * margin,
                ),
                Point(margin, margin),
            )?;
            let frame_pg: Polygon = frame.0.clone().try_into().unwrap();
            self.canvas.frame = Some(frame);
            let () = self.crop_to_frame(&frame_pg)?;
        }

        self.canvas
            .write_to_svg(config.size, config.output_directory.to_str().unwrap())?;

        Ok(())
    }
}

/// A set of config arguments for reading geometry from a geojson file and
/// writing SVG(s) to output file(s).
#[derive(Debug, TypedBuilder)]
pub struct MapConfig {
    #[builder(setter(transform = |x: impl IntoIterator<Item = impl AsRef<Path>> + std::fmt::Debug| paths_to_files(x).collect()))]
    input_files: Vec<File>,
    output_directory: PathBuf,
    size: Size,
    draw_frame: bool,
    scale_factor: f64,
}

/// Helper fn for transforming filepaths to files.
fn paths_to_files(
    file_paths: impl IntoIterator<Item = impl AsRef<Path>> + std::fmt::Debug,
) -> impl Iterator<Item = File> {
    file_paths
        .into_iter()
        .map(|fp| File::open(fp).expect("could not open file"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use plotz_geometry::shapes::polygon::Polygon;
    use tempdir::TempDir;

    #[test]
    fn test_render() -> Result<()> {
        let tmp_dir = TempDir::new("example").unwrap();

        let map_config = MapConfig::builder()
            .input_files(vec!["../testdata/example.geojson"])
            .output_directory(tmp_dir.path().to_path_buf())
            .size(Size {
                width: 1024,
                height: 1024,
            })
            .draw_frame(false)
            .scale_factor(0.9)
            .build();

        let mut map = Map::new(&map_config, None).unwrap();

        {
            // let mut rolling_bbox = BoundsCollector::default();
            // map.canvas.dos_by_bucket.iter().for_each(|(_bucket, dos)| {
            //     dos.iter().for_each(|(obj, _)| {
            //         rolling_bbox.incorporate(obj).expect("?");
            //     });
            // });
            // assert_eq!(rolling_bbox.items_seen(), 4);

            // // ^
            // // 5---+
            // // |   |
            // // +---3>
            // let b = rolling_bbox.bounds()?;
            // assert_eq!(b.x_min, 0.0);
            // assert_eq!(b.y_min, 0.0);
            // assert_eq!(b.y_max, 5.0);
            // assert_eq!(b.x_max, 3.0);
        }

        let () = map.do_all_adjustments(0.9, &map_config.size).unwrap();

        {
            // let mut rolling_bbox = BoundsCollector::default();
            // map.canvas.dos_by_bucket.iter().for_each(|(_bucket, dos)| {
            //     dos.iter().for_each(|(obj, _)| {
            //         rolling_bbox.incorporate(obj).expect("?");
            //     })
            // });
            // let b = rolling_bbox.bounds()?;
            // assert_float_eq!(b.x_min, 51.200, abs <= 0.000_01);
            // assert_float_eq!(b.y_min, -256.976635, abs <= 0.000_01);
            // assert_float_eq!(b.y_max, 1280.976635, abs <= 0.000_01);
            // assert_float_eq!(b.x_max, 972.8, abs <= 0.000_01);
        }

        let () = map.render(&map_config).unwrap();
        Ok(())
    }

    #[test]
    fn test_bl_shift() -> Result<()> {
        use plotz_color::*;

        for (initial, expected) in [
            // no shift
            ([(0, 0), (0, 1), (1, 0)], [(0, 0), (0, 1), (1, 0)]),
            // shift positive
            ([(-1, -1), (-1, 0), (0, -1)], [(0, 0), (0, 1), (1, 0)]),
            // shift negative
            ([(1, 1), (1, 2), (2, 1)], [(0, 0), (0, 1), (1, 0)]),
        ] {
            let obj = Obj2::Polygon(Polygon(initial)?);
            let mut map = Map {
                canvas: {
                    let mut canvas = Canvas::default();
                    canvas.dos_by_bucket.insert(
                        Some(Bucket::Area(Area::Beach)),
                        vec![(
                            obj,
                            Style {
                                color: ALICEBLUE,
                                ..Default::default()
                            },
                        )],
                    );
                    canvas
                },
                center: None,
            };
            map.adjust_bl_shift().unwrap();

            let mut x = map.canvas.dos_by_bucket.values();

            assert_eq!(x.next().unwrap()[0].0, Obj2::Polygon(Polygon(expected)?));
        }
        Ok(())
    }

    #[test]
    fn test_apply_scaling() -> Result<()> {
        use plotz_color::*;

        for (size, scale_factor, initial, expected) in [
            // rescale: 1024 * 0.9 = 921.6
            (
                Size {
                    width: 1024,
                    height: 1024,
                },
                0.9,
                [Point(0, 0), Point(0, 1.0), Point(1.0, 0)],
                [Point(0, 0), Point(0, 921.60), Point(921.60, 0)],
            ),
            // rescale: 100 * 0.9 = 90
            (
                Size {
                    width: 1000,
                    height: 1000,
                },
                0.9,
                [Point(0, 0), Point(0, 1), Point(1, 0)],
                [Point(0, 0), Point(0, 900), Point(900, 0)],
            ),
        ] {
            let obj = Obj2::Polygon(Polygon(initial)?);
            let mut map = Map {
                center: None,
                canvas: {
                    let mut canvas = Canvas::default();
                    canvas.dos_by_bucket.insert(
                        Some(Bucket::Area(Area::Beach)),
                        vec![(
                            obj,
                            Style {
                                color: ALICEBLUE,
                                ..Default::default()
                            },
                        )],
                    );
                    canvas
                },
            };
            map.adjust_scaling(scale_factor, &size).unwrap();

            let mut x = map.canvas.dos_by_bucket.values();

            assert_eq!(x.next().unwrap()[0].0, Obj2::Polygon(Polygon(expected)?));
        }
        Ok(())
    }
}
