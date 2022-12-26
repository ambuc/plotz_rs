//! The core logic of plotz-core, including Map and MapConfig.

use crate::{
    bucket::{Area, Bucket},
    bucketer::{Bucketer, DefaultBucketer},
    colored_obj::{ColoredObj, Obj},
    colorer::{Colorer, DefaultColorer},
    colorer_builder::DefaultColorerBuilder,
    svg::{write_layer_to_svg, Size, SvgWriteError},
};
use itertools::Itertools;
use tracing::*;

use float_ord::FloatOrd;
use lazy_static::lazy_static;
use plotz_color::ColorRGB;
use plotz_geojson::GeoJsonConversionError;
use plotz_geometry::{
    bounded::{streaming_bbox, Bounded, BoundingBoxError},
    point::Pt,
    polygon::Polygon,
    shading::{shade_polygon, ShadeConfig},
};
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};
use string_interner::{symbol::SymbolU32, StringInterner};
use thiserror::Error;

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
    color: ColorRGB,
    _tags: Vec<(SymbolU32, SymbolU32)>,
}
impl AnnotatedPolygon {
    /// Consumes an AnnotatedPolygon and casts down to a ColoredPolygon.
    pub fn to_colored_polygon(self) -> ColoredObj {
        ColoredObj {
            obj: Obj::Polygon(self.polygon),
            color: self.color,
        }
    }
}

fn latitude_to_y(latitude: f64) -> f64 {
    use std::f64::consts::PI;
    (((latitude + 90.0) / 360.0 * PI).tan()).ln() / PI * 180.0
}

lazy_static! {
    /// Which areas get shaded, and how much.
    pub static ref SHADINGS: std::collections::HashMap<Bucket, ShadeConfig> = [
        // TODO(jbuckland): Some of these scale poorly or fail to render. Can I
        // somehow autoderive this density?
        // TODO(jbuckland): Make the svg scale thickness much smaller for crosshatching.
        (
            Bucket::Area(Area::Park),
            ShadeConfig {
                gap: 5.0,
                slope: 10.0
            }
        ),
        (
            Bucket::Area(Area::Water),
            ShadeConfig {
                gap: 5.0,
                slope: -10.0
            }
        ),
    ].into();
}

/// An unadjusted set of annotated polygons, ready to be printed to SVG.
#[derive(Debug, PartialEq)]
pub struct Map {
    layers: Vec<(Bucket, Vec<ColoredObj>)>,
}
impl Map {
    fn objs_iter(&self) -> impl Iterator<Item = &Obj> {
        self.layers
            .iter()
            .flat_map(|(_b, vec)| vec)
            .map(|co| &co.obj)
    }
    fn objs_iter_mut(&mut self) -> impl Iterator<Item = &mut Obj> {
        self.layers
            .iter_mut()
            .flat_map(|(_b, vec)| vec)
            .map(|co| &mut co.obj)
    }
    fn polygons_iter(&self) -> impl Iterator<Item = &Polygon> {
        self.objs_iter().filter_map(|o| match o {
            Obj::Polygon(p) => Some(p),
            _ => None,
        })
    }
    fn polygons_iter_mut(&mut self) -> impl Iterator<Item = &mut Polygon> {
        self.objs_iter_mut().filter_map(|o| match o {
            Obj::Polygon(p) => Some(p),
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
                let crosshatchings: Vec<ColoredObj> = layers
                    .iter()
                    .filter_map(|co| match &co.obj {
                        Obj::Polygon(p) => Some(
                            shade_polygon(shade_config, p)
                                .expect("bad shade")
                                .into_iter()
                                .map(|s| ColoredObj {
                                    obj: Obj::Segment(s),
                                    color: co.color,
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
        Ok(())
    }

    /// Consumes a Map, adjusts each polygon, and writes the results as SVG to
    /// file(s).
    pub fn render(mut self, config: &MapConfig) -> Result<(), MapError> {
        info!(config = ?config);

        let () = self.adjust(config.scale_factor, &config.size)?;
        self.apply_shading();

        if config.draw_frame {
            info!("Adding frame.");
            let (w, h) = (config.size.width as f64, config.size.height as f64);
            self.layers.push((
                Bucket::Frame,
                vec![ColoredObj {
                    obj: Obj::Polygon(
                        Polygon([Pt(0.0, 0.0), Pt(0.0, w), Pt(h, w), Pt(h, 0.0)]).unwrap(),
                    ),
                    color: plotz_color::BLACK,
                }],
            ));
        }

        // write layer 0 with all.
        write_layer_to_svg(
            config.size,
            config.output_directory.join("0.svg"),
            self.layers.iter().flat_map(|(_bucket, vec)| vec),
        )?;

        // write each layer individually.
        for (idx, (bucket, polygons)) in self.layers.into_iter().enumerate() {
            let path = config.output_directory.join(format!("{}.svg", idx + 1));
            let num = write_layer_to_svg(config.size, &path, &polygons)?;
            trace!("Wrote {:>4?} polygons to {:?} for {:?}", num, path, bucket);
        }

        Ok(())
    }
}

/// A set of config arguments for reading geometry from a geojson file and
/// writing SVG(s) to output file(s).
#[derive(Debug)]
pub struct MapConfig {
    input_files: Vec<File>,
    output_directory: PathBuf,
    size: Size,
    draw_frame: bool,
    scale_factor: f64,
}

impl MapConfig {
    /// Instantiates a new MapConfig from many file paths.
    pub fn new_from_files(
        file_paths: impl IntoIterator<Item = impl AsRef<Path>> + std::fmt::Debug,
        output_directory: PathBuf,
        size: Size,
        draw_frame: bool,
        scale_factor: f64,
    ) -> Result<MapConfig, MapError> {
        info!("Loading MapConfig from files: {:?}", file_paths);
        let mut files = vec![];
        for fp in file_paths {
            files.push(File::open(fp)?);
        }
        Ok(MapConfig {
            input_files: files,
            output_directory,
            size,
            draw_frame,
            scale_factor,
        })
    }

    /// Instantiates a new MapConfig from one file path.
    pub fn new_from_file(
        file_path: &str,
        output_directory: PathBuf,
        size: Size,
        draw_frame: bool,
        scale_factor: f64,
    ) -> Result<MapConfig, MapError> {
        trace!("MapConfig::new_from_file");
        Self::new_from_files(
            std::iter::once(file_path),
            output_directory,
            size,
            draw_frame,
            scale_factor,
        )
    }

    /// Consumes MapConfig, performs bucketing and coloring, and returns an
    /// unadjusted Map instance.
    pub fn make_map(&self) -> Result<Map, MapError> {
        let mut interner = StringInterner::new();
        let bucketer = DefaultBucketer::new(&mut interner);
        let colorer: DefaultColorer = DefaultColorerBuilder::default();

        let layers = self
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
                        color: colorer.color(bucket).expect("could not color"),
                        _tags: tags.clone(),
                    })
                })
                .collect::<Vec<_>>()
            })
            .sorted_by(|ap_1, ap_2| std::cmp::Ord::cmp(&ap_1.bucket, &ap_2.bucket))
            .group_by(|ap| ap.bucket)
            .into_iter()
            .map(|(k, v)| (k, v.map(|ap| ap.to_colored_polygon()).collect()))
            .collect::<Vec<(Bucket, Vec<ColoredObj>)>>();

        Ok(Map { layers })
    }
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

        let map_config = MapConfig::new_from_file(
            /*file_path=*/ "../testdata/example.geojson",
            /*output_file_prefix=*/ tmp_dir.path().to_path_buf(),
            /*size= */
            Size {
                width: 1024,
                height: 1024,
            },
            /*draw_frame */ false,
            /*scale_factor */ 0.9,
        )
        .unwrap();

        let mut map: Map = map_config.make_map().unwrap();

        {
            let mut rolling_bbox = BoundsCollector::new();
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
            let mut rolling_bbox = BoundsCollector::new();
            map.layers.iter().for_each(|(_, objs)| {
                objs.iter().for_each(|colored_obj| {
                    rolling_bbox.incorporate(&colored_obj.obj);
                })
            });
            assert_float_eq!(rolling_bbox.left_bound(), 0.0, abs <= 0.000_01);
            assert_float_eq!(rolling_bbox.bottom_bound(), 0.0, abs <= 0.000_01);
            assert_float_eq!(rolling_bbox.top_bound(), 1537.95327, abs <= 0.000_01);
            assert_float_eq!(rolling_bbox.right_bound(), 921.59999, abs <= 0.000_01);
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
            let obj = Obj::Polygon(Polygon(initial).unwrap());
            let mut map = Map {
                layers: vec![(
                    Bucket::Area(Area::Beach),
                    vec![ColoredObj {
                        obj: obj,
                        color: ALICEBLUE,
                    }],
                )],
            };
            map.apply_bl_shift().unwrap();

            assert_eq!(
                map.layers[0].1[0].obj,
                Obj::Polygon(Polygon(expected).unwrap())
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
            let obj = Obj::Polygon(Polygon(initial).unwrap());
            let mut map = Map {
                layers: vec![(
                    Bucket::Area(Area::Beach),
                    vec![ColoredObj {
                        obj: obj,
                        color: ALICEBLUE,
                    }],
                )],
            };
            map.apply_scaling(scale_factor, &size).unwrap();

            assert_eq!(
                map.layers[0].1[0].obj,
                Obj::Polygon(Polygon(expected).unwrap())
            );
        }
    }
}
