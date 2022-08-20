//! The core logic of plotz-core, including Map and MapConfig.

use crate::{
    bucket::Bucket,
    bucketer::{Bucketer, DefaultBucketer},
    colored_polygon::ColoredPolygon,
    colorer::{Colorer, DefaultColorer},
    colorer_builder::DefaultColorerBuilder,
    svg::{write_layer_to_svg, SvgWriteError},
};
use float_ord::FloatOrd;
use plotz_color::ColorRGB;
use plotz_geojson::GeoJsonConversionError;
use plotz_geometry::{
    bounded::{streaming_bbox, Bounded, BoundingBoxError},
    point::Pt,
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
        // first compute current bbox and shift everything positive.
        dbg!(&self.get_bbox());

        let shift = self.get_bbox()?.bl_bound();
        self.apply(&|p| *p -= shift);

        dbg!(&self.get_bbox());

        let bbox = self.get_bbox()?;
        let scaling_factor = std::cmp::max(FloatOrd(bbox.width()), FloatOrd(bbox.height())).0;
        self.apply(&|p| *p *= 1.0 / scaling_factor);

        dbg!(&self.get_bbox());

        for (idx, layer) in self.layers.into_iter().enumerate() {
            write_layer_to_svg(
                /*width,height=*/ (1024.0, 1024.0),
                /*path=*/ self.config.output_directory.join(format!("{}.svg", idx)),
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
}

impl MapConfig {
    /// Instantiates a new MapConfig from many file paths.
    pub fn new_from_files(
        file_paths: impl IntoIterator<Item = impl AsRef<Path>>,
        output_directory: PathBuf,
    ) -> Result<MapConfig, MapError> {
        let mut files = vec![];
        for fp in file_paths {
            files.push(File::open(fp)?);
        }
        Ok(MapConfig {
            input_files: files,
            output_directory,
        })
    }

    /// Instantiates a new MapConfig from one file path.
    pub fn new_from_file(
        file_path: &str,
        output_directory: PathBuf,
    ) -> Result<MapConfig, MapError> {
        Self::new_from_files(std::iter::once(file_path), output_directory)
    }

    /// Consumes MapConfig, performs bucketing and coloring, and returns an
    /// unadjusted Map instance.
    pub fn make_map(self) -> Result<Map, MapError> {
        let mut interner = StringInterner::new();
        let bucketer = DefaultBucketer::new(&mut interner);
        let colorer: DefaultColorer = DefaultColorerBuilder::default();

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

                    Some(AnnotatedPolygon {
                        polygon: polygon.clone(),
                        _bucket: bucket,
                        color: colorer.color(bucket).ok()?,
                        _tags: tags.clone(),
                    })
                })
                .collect::<Vec<_>>())
            })
            .collect::<Result<_, MapError>>()?;

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
        )
        .unwrap()
        .make_map()
        .unwrap();
    }
}
