//! The core logic of plotz-core, including Map and MapConfig.

#![allow(clippy::let_unit_value)]

use crate::{
    bucket::{Area, Bucket, Path as BucketPath},
    bucketer::{Bucketer, DefaultBucketer},
    draw_obj::{DrawObj, DrawObjInner},
    frame::make_frame,
    svg::{write_layer_to_svg, Size, SvgWriteError},
};
use float_ord::FloatOrd;
use itertools::Itertools;
use lazy_static::lazy_static;
use plotz_color::*;
use plotz_geojson::GeoJsonConversionError;
use plotz_geometry::{
    bounded::{streaming_bbox, Bounded, BoundingBoxError},
    point::Pt,
    polygon::Polygon,
    shading::{shade_polygon, ShadeConfig},
};
use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};
use string_interner::{symbol::SymbolU32, StringInterner};
use thiserror::Error;
use tracing::*;
use typed_builder::TypedBuilder;

#[derive(Debug, Error)]
/// A general error you might encounter when rendering a Map.
pub enum MapError {
    /// could not map
    #[error("could not map")]
    MapError,
    /// geojson conversion error
    #[error("geojson conversion error")]
    GeoJsonConversionError(#[from] GeoJsonConversionError),
    /// file read error
    #[error("file read error")]
    FileReadError(#[from] std::io::Error),
    /// serde parse error
    #[error("serde parse error")]
    SerdeParseError(#[from] serde_json::Error),
    /// bounding box error
    #[error("bounding box error")]
    BoundingBoxError(#[from] BoundingBoxError),
    /// svg write error
    #[error("svg write error")]
    SvgWriteError(#[from] SvgWriteError),
}

#[derive(Debug)]
/// A polygon with some annotations (bucket, color, tags, etc.).
pub struct AnnotatedPolygon {
    polygon: Polygon,
    bucket: Bucket,
    color: &'static ColorRGB,
    thickness: f64,
    _tags: Vec<(SymbolU32, SymbolU32)>,
}
impl AnnotatedPolygon {
    /// Consumes an AnnotatedPolygon and casts down to a ColoredPolygon.
    pub fn to_colored_polygon(self) -> DrawObj {
        DrawObj::from_polygon(self.polygon)
            .with_color(self.color)
            .with_thickness(self.thickness)
    }
}

fn latitude_to_y(latitude: f64) -> f64 {
    use std::f64::consts::PI;
    (((latitude + 90.0) / 360.0 * PI).tan()).ln() / PI * 180.0
}

lazy_static! {
    /// Which areas get shaded, and how much.
    pub static ref DEFAULT_COLORING: std::collections::HashMap<Bucket, ColorRGB> =
            HashMap::from([
                (Bucket::Path(BucketPath::Highway1), BLACK),
                (Bucket::Path(BucketPath::Highway2), DARKGRAY),
                (Bucket::Path(BucketPath::Highway3), GRAY),
                (Bucket::Path(BucketPath::Highway4), LIGHTGRAY),
                (Bucket::Path(BucketPath::Cycleway), DARKGRAY),
                (Bucket::Path(BucketPath::Pedestrian), DARKGRAY),
                (Bucket::Path(BucketPath::Rail), DARKGRAY),
                (Bucket::Path(BucketPath::Boundary), DARKGRAY),
                (Bucket::Area(Area::Beach), TAN),
                (Bucket::Area(Area::Building), DARKGREY),
                (Bucket::Area(Area::Business), DARKGREY),
                (Bucket::Area(Area::Fun), DARKGREEN),
                (Bucket::Area(Area::NaturalRock), DARKGRAY),
                (Bucket::Area(Area::Park), GREEN),
                (Bucket::Area(Area::Rail), ORANGE),
                (Bucket::Area(Area::Tree), BROWN),
                (Bucket::Area(Area::Water), LIGHTBLUE),
            ]);

    /// Which areas get shaded, and how much.
    pub static ref SHADINGS: std::collections::HashMap<Bucket, ShadeConfig> = [
        // TODO(jbuckland): Some of these scale poorly or fail to render. Can I
        // somehow autoderive this density?
        (
            Bucket::Area(Area::Park),
            ShadeConfig {
                gap: 5.0,
                slope: 10.0,
                thickness: 2.0,
            }
        ),
        (
            Bucket::Area(Area::Water),
            ShadeConfig {
                gap: 5.0,
                slope: -10.0,
                thickness: 2.0,
            }
        ),
    ].into();

    /// How thick the default line is.
    pub static ref DEFAULT_THICKNESS: f64 = 5.0;
}

/// An unadjusted set of annotated polygons, ready to be printed to SVG.
#[derive(Debug, PartialEq)]
pub struct Map {
    layers: Vec<(Bucket, Vec<DrawObj>)>,
}
impl Map {
    /// Consumes MapConfig, performs bucketing and coloring, and returns an
    /// unadjusted Map instance.
    pub fn new(map_config: &MapConfig) -> Result<Map, MapError> {
        let mut interner = StringInterner::new();
        let bucketer = DefaultBucketer::new(&mut interner);

        let layers = map_config
            .input_files
            .iter()
            .flat_map(|file| {
                plotz_geojson::parse_geojson(
                    &mut interner,
                    serde_json::from_reader(BufReader::new(file)).expect("read"),
                )
                .expect("parse")
                .iter()
                .filter_map(|(polygon, tags)| {
                    let bucket = tags
                        .iter()
                        .map(|t| bucketer.bucket(*t))
                        .find_map(|r| r.ok())?;

                    Some(AnnotatedPolygon {
                        polygon: polygon.clone(),
                        bucket,
                        color: &DEFAULT_COLORING[&bucket],
                        thickness: *DEFAULT_THICKNESS,
                        _tags: tags.clone(),
                    })
                })
                .collect::<Vec<_>>()
            })
            .sorted_by(|ap_1, ap_2| std::cmp::Ord::cmp(&ap_1.bucket, &ap_2.bucket))
            .group_by(|ap| ap.bucket)
            .into_iter()
            .map(|(k, v)| (k, v.map(|ap| ap.to_colored_polygon()).collect()))
            .collect::<Vec<(Bucket, Vec<DrawObj>)>>();

        Ok(Map { layers })
    }

    fn objs_iter(&self) -> impl Iterator<Item = &DrawObjInner> {
        self.layers
            .iter()
            .flat_map(|(_b, vec)| vec)
            .map(|co| &co.obj)
    }
    fn objs_iter_mut(&mut self) -> impl Iterator<Item = &mut DrawObjInner> {
        self.layers
            .iter_mut()
            .flat_map(|(_b, vec)| vec)
            .map(|co| &mut co.obj)
    }
    fn polygons_iter(&self) -> impl Iterator<Item = &Polygon> {
        self.objs_iter().filter_map(|o| match o {
            DrawObjInner::Polygon(p) => Some(p),
            _ => None,
        })
    }
    fn polygons_iter_mut(&mut self) -> impl Iterator<Item = &mut Polygon> {
        self.objs_iter_mut().filter_map(|o| match o {
            DrawObjInner::Polygon(p) => Some(p),
            _ => None,
        })
    }

    fn get_bbox(&self) -> Result<Polygon, MapError> {
        Ok(streaming_bbox(self.polygons_iter())?)
    }

    fn apply_bl_shift(&mut self) -> Result<(), MapError> {
        let curr_bbox = self.get_bbox()?;
        self.polygons_iter_mut().for_each(|p| {
            *p -= curr_bbox.bl_bound();
        });
        Ok(())
    }

    fn apply_centering(&mut self, dest_size: &Size) -> Result<(), MapError> {
        let curr_bbox = self.get_bbox()?;
        self.polygons_iter_mut().for_each(|p| {
            *p += Pt(
                (dest_size.width as f64 - curr_bbox.right_bound()) / 2.0,
                (dest_size.height as f64 - curr_bbox.top_bound()) / 2.0,
            );
        });
        Ok(())
    }

    fn apply_scaling(&mut self, scale_factor: f64, dest_size: &Size) -> Result<(), MapError> {
        let curr_bbox = self.get_bbox()?;
        let scaling_factor = std::cmp::max(
            FloatOrd(dest_size.height as f64 / curr_bbox.height().abs()),
            FloatOrd(dest_size.width as f64 / curr_bbox.width().abs()),
        )
        .0 * scale_factor;
        self.polygons_iter_mut().for_each(|p| *p *= scaling_factor);
        Ok(())
    }

    fn apply_shading(&mut self) {
        for (bucket, layers) in self.layers.iter_mut() {
            if let Some(shade_config) = SHADINGS.get(bucket) {
                // keep the frame, add the crosshatchings.
                let crosshatchings: Vec<DrawObj> = layers
                    .iter()
                    .filter_map(|co| match &co.obj {
                        DrawObjInner::Polygon(p) => Some(
                            shade_polygon(shade_config, p)
                                .expect("bad shade")
                                .into_iter()
                                .map(|s| DrawObj {
                                    obj: DrawObjInner::Segment(s),
                                    color: co.color,
                                    thickness: shade_config.thickness,
                                })
                                .collect::<Vec<_>>(),
                        ),
                        _ => None,
                    })
                    .flatten()
                    .collect();
                layers.extend(crosshatchings);
            }
        }
    }

    /// Adjusts the map for scale/transform issues.
    pub fn adjust(&mut self, scale_factor: f64, dest_size: &Size) -> Result<(), MapError> {
        // first compute current bbox and shift everything positive.
        self.polygons_iter_mut().for_each(|p| {
            p.flip_y();
            p.pts
                .iter_mut()
                .for_each(|pt| pt.y.0 = latitude_to_y(pt.y.0))
        });
        self.apply_bl_shift()?;
        self.apply_scaling(scale_factor, dest_size)?;
        self.apply_centering(dest_size)?;
        Ok(())
    }

    /// Adjusts the map for manual transform correction issues.
    pub fn shift(&mut self, shift_x: f64, shift_y: f64) -> Result<(), MapError> {
        //
        self.polygons_iter_mut().for_each(|p| {
            *p += (shift_x, shift_y).into();
        });
        Ok(())
    }

    /// Consumes a Map, adjusts each polygon, and writes the results as SVG to
    /// file(s).
    pub fn render(mut self, config: &MapConfig) -> Result<(), MapError> {
        trace!(config = ?config.input_files);

        let () = self.adjust(config.scale_factor, &config.size)?;
        let () = self.shift(config.shift_x, config.shift_y)?;
        self.apply_shading();

        if config.draw_frame {
            info!("Adding frame.");
            self.layers.push((
                Bucket::Frame,
                vec![make_frame(
                    (config.size.width as f64, config.size.height as f64),
                    Pt(0.0, 0.0),
                )],
            ));
        }

        // write layer 0 with all.
        {
            let path_0 = config.output_directory.join("0.svg");
            let num = write_layer_to_svg(
                config.size,
                &path_0,
                self.layers.iter().flat_map(|(_bucket, vec)| vec),
            )?;
            trace!("Wrote {:>4?} polygons to {:?} for _all_", num, path_0);
        }

        // write each layer individually.
        for (idx, (bucket, polygons)) in self.layers.into_iter().enumerate() {
            let path = config.output_directory.join(format!("{}.svg", idx + 1));
            let num = write_layer_to_svg(config.size, &path, &polygons)?;
            info!("Wrote {:>4?} polygons to {:?} for {:?}", num, path, bucket);
        }

        Ok(())
    }
}

/// A set of config arguments for reading geometry from a geojson file and
/// writing SVG(s) to output file(s).
#[derive(Debug, TypedBuilder)]
pub struct MapConfig {
    #[builder(setter(transform = |x: impl IntoIterator<Item = impl AsRef<Path>> + std::fmt::Debug| paths_to_files(x)))]
    input_files: Vec<File>,
    output_directory: PathBuf,
    size: Size,
    draw_frame: bool,
    scale_factor: f64,
    shift_x: f64,
    shift_y: f64,
}

/// Helper fn for transforming filepaths to files.
fn paths_to_files(
    file_paths: impl IntoIterator<Item = impl AsRef<Path>> + std::fmt::Debug,
) -> Vec<File> {
    file_paths
        .into_iter()
        .map(|fp| File::open(fp).expect("could not open file"))
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use float_eq::assert_float_eq;
    use plotz_geometry::bounded::BoundsCollector;
    use tempdir::TempDir;

    #[test]
    fn test_render() {
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
            .shift_x(0.0)
            .shift_y(0.0)
            .build();

        let mut map = Map::new(&map_config).unwrap();

        {
            let mut rolling_bbox = BoundsCollector::default();
            map.layers.iter().for_each(|(_, objs)| {
                objs.iter().for_each(|colored_obj| {
                    rolling_bbox.incorporate(&colored_obj.obj);
                })
            });
            assert_eq!(rolling_bbox.items_seen(), 4);

            // ^
            // 5---+
            // |   |
            // +---3>
            assert_eq!(rolling_bbox.left_bound(), 0.0);
            assert_eq!(rolling_bbox.bottom_bound(), 0.0);
            assert_eq!(rolling_bbox.top_bound(), 5.0);
            assert_eq!(rolling_bbox.right_bound(), 3.0);
        }

        let () = map.adjust(0.9, &map_config.size).unwrap();

        {
            let mut rolling_bbox = BoundsCollector::default();
            map.layers.iter().for_each(|(_, objs)| {
                objs.iter().for_each(|colored_obj| {
                    rolling_bbox.incorporate(&colored_obj.obj);
                })
            });
            assert_float_eq!(rolling_bbox.left_bound(), 51.200, abs <= 0.000_01);
            assert_float_eq!(rolling_bbox.bottom_bound(), -256.976635, abs <= 0.000_01);
            assert_float_eq!(rolling_bbox.top_bound(), 1280.976635, abs <= 0.000_01);
            assert_float_eq!(rolling_bbox.right_bound(), 972.8, abs <= 0.000_01);
        }

        let () = map.render(&map_config).unwrap();
    }

    #[test]
    fn test_bl_shift() {
        use plotz_color::*;

        for (initial, expected) in [
            // no shift
            (
                [Pt(0, 0), Pt(0, 1), Pt(1, 0)],
                [Pt(0, 0), Pt(0, 1), Pt(1, 0)],
            ),
            // shift positive
            (
                [Pt(-1, -1), Pt(-1, 0), Pt(0, -1)],
                [Pt(0, 0), Pt(0, 1), Pt(1, 0)],
            ),
            // shift negative
            (
                [Pt(1, 1), Pt(1, 2), Pt(2, 1)],
                [Pt(0, 0), Pt(0, 1), Pt(1, 0)],
            ),
        ] {
            let obj = DrawObjInner::Polygon(Polygon(initial).unwrap());
            let mut map = Map {
                layers: vec![(
                    Bucket::Area(Area::Beach),
                    vec![DrawObj {
                        obj: obj,
                        color: &ALICEBLUE,
                        thickness: 1.0,
                    }],
                )],
            };
            map.apply_bl_shift().unwrap();

            assert_eq!(
                map.layers[0].1[0].obj,
                DrawObjInner::Polygon(Polygon(expected).unwrap())
            );
        }
    }

    #[test]
    fn test_apply_scaling() {
        use plotz_color::*;

        for (size, scale_factor, initial, expected) in [
            // rescale: 1024 * 0.9 = 921.6
            (
                Size {
                    width: 1024,
                    height: 1024,
                },
                0.9,
                [Pt(0.0, 0.0), Pt(0.0, 1.0), Pt(1.0, 0.0)],
                [Pt(0.0, 0.0), Pt(0.0, 921.60), Pt(921.60, 0.0)],
            ),
            // rescale: 100 * 0.9 = 90
            (
                Size {
                    width: 1000,
                    height: 1000,
                },
                0.9,
                [Pt(0.0, 0.0), Pt(0.0, 1.0), Pt(1.0, 0.0)],
                [Pt(0.0, 0.0), Pt(0.0, 900.0), Pt(900.0, 0.0)],
            ),
        ] {
            let obj = DrawObjInner::Polygon(Polygon(initial).unwrap());
            let mut map = Map {
                layers: vec![(
                    Bucket::Area(Area::Beach),
                    vec![DrawObj {
                        obj: obj,
                        color: &ALICEBLUE,
                        thickness: 1.0,
                    }],
                )],
            };
            map.apply_scaling(scale_factor, &size).unwrap();

            assert_eq!(
                map.layers[0].1[0].obj,
                DrawObjInner::Polygon(Polygon(expected).unwrap())
            );
        }
    }
}
