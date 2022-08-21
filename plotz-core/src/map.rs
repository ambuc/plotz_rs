//! The core logic of plotz-core, including Map and MapConfig.

use crate::{
    bucket::Bucket,
    bucketer::{Bucketer, DefaultBucketer},
    colored_polygon::ColoredPolygon,
    colorer::{Colorer, DefaultColorer},
    colorer_builder::DefaultColorerBuilder,
    svg::{write_layer_to_svg, Size, SvgWriteError},
};
use float_ord::FloatOrd;
use log::info;
use plotz_color::ColorRGB;
use plotz_geojson::GeoJsonConversionError;
use plotz_geometry::{
    bounded::{streaming_bbox, Bounded, BoundingBoxError},
    polygon::Polygon,
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
    _bucket: Bucket,
    color: ColorRGB,
    _tags: Vec<(SymbolU32, SymbolU32)>,
}
impl AnnotatedPolygon {
    /// Consumes an AnnotatedPolygon and casts down to a ColoredPolygon.
    pub fn to_colored_polygon(self) -> ColoredPolygon {
        ColoredPolygon {
            polygon: self.polygon,
            color: self.color,
        }
    }
}

fn latitude_to_y(latitude: f64) -> f64 {
    use std::f64::consts::PI;
    (((latitude + 90.0) / 360.0 * PI).tan()).ln() / PI * 180.0
}

/// An unadjusted set of annotated polygons, ready to be printed to SVG.
pub struct Map {
    config: MapConfig,
    layers: Vec<Vec<AnnotatedPolygon>>,
}
impl Map {
    fn get_bbox(&self) -> Result<Polygon, MapError> {
        Ok(streaming_bbox(
            self.layers
                .iter()
                .flatten()
                .map(|AnnotatedPolygon { polygon, .. }| polygon),
        )?)
    }

    fn apply(&mut self, f: &dyn Fn(&mut Polygon)) {
        self.layers
            .iter_mut()
            .flatten()
            .for_each(|ap| f(&mut ap.polygon));
    }

    /// Consumes a Map, adjusts each polygon, and writes the results as SVG to
    /// file(s).
    pub fn render(mut self) -> Result<(), MapError> {
        info!("Rendering map.");
        // first compute current bbox and shift everything positive.

        self.apply(&|p| p.flip_y());
        self.apply(&|p| {
            p.pts
                .iter_mut()
                .for_each(|pt| pt.y.0 = latitude_to_y(pt.y.0))
        });

        let shift = self.get_bbox()?.bl_bound();
        self.apply(&|p| *p -= shift);

        let bbox = self.get_bbox()?;
        let scaling_factor = 1.0
            / std::cmp::max(FloatOrd(bbox.width().abs()), FloatOrd(bbox.height().abs())).0
            * self.config.size.max() as f64
            * 0.73; // why 0.73?
        self.apply(&|p| *p *= scaling_factor);

        for (idx, layer) in self.layers.into_iter().enumerate() {
            let path = self.config.output_directory.join(format!("{}.svg", idx));
            info!(
                "Writing layer #{:?} ({:?} polygons) to {:?}",
                idx,
                layer.len(),
                path
            );
            write_layer_to_svg(
                /*width,height=*/ self.config.size,
                /*path=*/ path,
                /*polygons=*/ layer.into_iter().map(|ap| ap.to_colored_polygon()),
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
        let mut interner = StringInterner::new();
        let bucketer = DefaultBucketer::new(&mut interner);
        let colorer: DefaultColorer = DefaultColorerBuilder::default();

        let mut buckets_histogram = std::collections::HashMap::<Bucket, usize>::new();
        let mut _colors_histogram = std::collections::HashMap::<ColorRGB, usize>::new();

        let layers: Vec<Vec<AnnotatedPolygon>> = self
            .input_files
            .iter()
            .map(|file| {
                Ok(plotz_geojson::parse_geojson(
                    &mut interner,
                    serde_json::from_reader(BufReader::new(file))?,
                )?
                .iter()
                .filter_map(|(polygon, tags)| {
                    let bucket = tags
                        .iter()
                        .map(|t| bucketer.bucket(*t))
                        .filter_map(|r| r.ok())
                        .next()?;
                    *buckets_histogram.entry(bucket).or_default() += 1;

                    let color = colorer.color(bucket).ok()?;
                    *_colors_histogram.entry(color).or_default() += 1;

                    Some(AnnotatedPolygon {
                        polygon: polygon.clone(),
                        _bucket: bucket,
                        color,
                        _tags: tags.clone(),
                    })
                })
                .collect::<Vec<_>>())
            })
            .collect::<Result<_, MapError>>()?;

        info!("Got buckets {:?}", buckets_histogram);

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
