use crate::{
    bucket::Bucket,
    bucketer::{Bucketer, DefaultBucketer},
    colored_polygon::ColoredPolygon,
    colorer::{Colorer, DefaultColorer},
    colorer_builder::DefaultColorerBuilder,
    svg::{write_layer_to_svg, SvgWriteError},
};
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
pub enum MapError {
    #[error("could not map")]
    MapError,
    #[error("geojson conversion error")]
    GeoJsonConversionError(#[from] GeoJsonConversionError),
    #[error("file read error")]
    FileReadError(#[from] std::io::Error),
    #[error("serde parse error")]
    SerdeParseError(#[from] serde_json::Error),
    #[error("bounding box error")]
    BoundingBoxError(#[from] BoundingBoxError),
    #[error("svg write error")]
    SvgWriteError(#[from] SvgWriteError),
}

#[derive(Debug)]
pub struct AnnotatedPolygon {
    polygon: Polygon,
    _bucket: Bucket,
    color: ColorRGB,
    _tags: Vec<(SymbolU32, SymbolU32)>,
}
impl AnnotatedPolygon {
    pub fn to_colored_polygon(self) -> ColoredPolygon {
        ColoredPolygon {
            polygon: self.polygon,
            color: self.color,
        }
    }
}

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

    fn get_shift(&self) -> Result<Pt, MapError> {
        Ok(self.get_bbox()?.bl_bound())
    }
    fn get_scaling_factor(&self) -> Result<f64, MapError> {
        unimplemented!();
        //
    }

    fn apply_shift(&mut self, shift: Pt) {
        self.layers
            .iter_mut()
            .flatten()
            .for_each(|ap| ap.polygon -= shift);
    }

    fn apply_scaling_factor(&mut self, scaling_factor: f64) {
        self.layers
            .iter_mut()
            .flatten()
            .for_each(|ap| ap.polygon *= scaling_factor);
    }

    pub fn render(mut self) -> Result<(), MapError> {
        // first compute current bbox and shift everything positive.
        dbg!(&self.get_bbox());

        let shift = self.get_shift()?;
        self.apply_shift(shift);

        dbg!(&self.get_bbox());

        let scaling_factor = self.get_scaling_factor()?;
        self.apply_scaling_factor(scaling_factor);

        dbg!(&self.get_bbox());

        // then scale up to hit frame.
        // center the whole thing.

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

pub struct MapConfig {
    input_files: Vec<File>,
    output_directory: PathBuf,
}

impl MapConfig {
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
    pub fn new_from_file(
        file_path: &str,
        output_directory: PathBuf,
    ) -> Result<MapConfig, MapError> {
        Self::new_from_files(std::iter::once(file_path), output_directory)
    }

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
