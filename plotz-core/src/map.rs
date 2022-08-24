//! The core logic of plotz-core, including Map and MapConfig.

use crate::{
    bucket::{Area, Bucket},
    bucketer::{Bucketer, DefaultBucketer},
    colored_obj::{ColoredObj, Obj},
    colorer::{Colorer, DefaultColorer},
    colorer_builder::DefaultColorerBuilder,
    svg::{write_layer_to_svg, Size, SvgWriteError},
};
use float_ord::FloatOrd;
use lazy_static::lazy_static;
use log::info;
use plotz_color::ColorRGB;
use plotz_geojson::GeoJsonConversionError;
use plotz_geometry::{
    bounded::{streaming_bbox, Bounded, BoundingBoxError},
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
pub struct Map {
    config: MapConfig,
    layers: Vec<(Bucket, Vec<ColoredObj>)>,
}
impl Map {
    fn get_bbox(&self) -> Result<Polygon, MapError> {
        Ok(streaming_bbox(
            self.layers
                .iter()
                .flat_map(|(_, vec)| vec)
                .map(|ColoredObj { obj, .. }| obj),
        )?)
    }

    // NB: only applies transformation to polygons.
    fn apply_polygons(&mut self, f: &dyn Fn(&mut Polygon)) {
        for (_, vec) in self.layers.iter_mut() {
            for p in vec.iter_mut() {
                if let Obj::Polygon(polygon) = &mut p.obj {
                    f(polygon);
                }
            }
        }
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

    /// Consumes a Map, adjusts each polygon, and writes the results as SVG to
    /// file(s).
    pub fn render(mut self) -> Result<(), MapError> {
        info!("Rendering map.");
        // first compute current bbox and shift everything positive.

        self.apply_polygons(&|p| p.flip_y());
        self.apply_polygons(&|p| {
            p.pts
                .iter_mut()
                .for_each(|pt| pt.y.0 = latitude_to_y(pt.y.0))
        });

        let shift = self.get_bbox()?.bl_bound();
        self.apply_polygons(&|p| *p -= shift);

        let bbox = self.get_bbox()?;
        let scaling_factor = 1.0
            / std::cmp::max(FloatOrd(bbox.width().abs()), FloatOrd(bbox.height().abs())).0
            * self.config.size.max() as f64
            * 0.73; // why 0.73?
        self.apply_polygons(&|p| *p *= scaling_factor);

        self.apply_shading();

        // write layer 0 with all.
        info!("Writing all.");
        write_layer_to_svg(
            self.config.size,
            self.config.output_directory.join("0.svg"),
            self.layers.iter().flat_map(|(_bucket, vec)| vec),
        )?;

        // write each layer individually.
        for (idx, (bucket, polygons)) in self.layers.into_iter().enumerate() {
            info!("Writing {:?}", bucket);
            write_layer_to_svg(
                self.config.size,
                self.config
                    .output_directory
                    .join(format!("{}.svg", idx + 1)),
                &polygons,
            )?;
        }

        Ok(())
    }
}

/// A set of config arguments for reading geometry from a geojson file and
/// writing SVG(s) to output file(s).
pub struct MapConfig {
    input_files: Vec<File>,
    output_directory: PathBuf,
    size: Size,
}

impl MapConfig {
    /// Instantiates a new MapConfig from many file paths.
    pub fn new_from_files(
        file_paths: impl IntoIterator<Item = impl AsRef<Path>>,
        output_directory: PathBuf,
        size: Size,
    ) -> Result<MapConfig, MapError> {
        let mut files = vec![];
        for fp in file_paths {
            files.push(File::open(fp)?);
        }
        Ok(MapConfig {
            input_files: files,
            output_directory,
            size,
        })
    }

    /// Instantiates a new MapConfig from one file path.
    pub fn new_from_file(
        file_path: &str,
        output_directory: PathBuf,
        size: Size,
    ) -> Result<MapConfig, MapError> {
        Self::new_from_files(std::iter::once(file_path), output_directory, size)
    }

    /// Consumes MapConfig, performs bucketing and coloring, and returns an
    /// unadjusted Map instance.
    pub fn make_map(self) -> Result<Map, MapError> {
        use itertools::Itertools;

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

        Ok(Map {
            config: self,
            layers,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn test_render() {
        let tmp_dir = TempDir::new("example").unwrap();

        MapConfig::new_from_file(
            /*file_path=*/ "../testdata/example.geojson",
            /*output_file_prefix=*/ tmp_dir.path().to_path_buf(),
            /*size= */
            Size {
                width: 1024,
                height: 1024,
            },
        )
        .unwrap()
        .make_map()
        .unwrap();
    }
}
