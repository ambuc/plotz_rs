use crate::{
    bucket::Bucket,
    bucketer::{Bucketer, DefaultBucketer},
    colored_polygon::ColoredPolygon,
    colorer::{Colorer, DefaultColorer},
    colorer_builder::DefaultColorerBuilder,
    svg::{write_layers_to_svgs, SvgWriteError},
};
use plotz_color::ColorRGB;
use plotz_geojson::GeoJsonConversionError;
use plotz_geometry::{
    bounded::{streaming_bbox, Bounded, BoundingBoxError},
    polygon::Polygon,
};
use std::{fs::File, io::BufReader};
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

pub struct AnnotatedPolygon {
    polygon: Polygon,
    bucket: Bucket,
    color: ColorRGB,
    tags: Vec<(SymbolU32, SymbolU32)>,
}

pub struct Map {
    input_files: Vec<File>,
    output_file_prefix: String,
}

impl Map {
    pub fn new_from_files(
        file_paths: Vec<&str>,
        output_file_prefix: String,
    ) -> Result<Map, MapError> {
        let mut files = vec![];
        for fp in file_paths {
            files.push(File::open(fp)?);
        }
        Ok(Map {
            input_files: files,
            output_file_prefix,
        })
    }
    pub fn new_from_file(file_path: &str, output_file_prefix: String) -> Result<Map, MapError> {
        Self::new_from_files(vec![file_path], output_file_prefix)
    }

    pub fn render(&self) -> Result<(), MapError> {
        let mut interner = StringInterner::new();
        let bucketer = DefaultBucketer::new(&mut interner);
        let colorer: DefaultColorer = DefaultColorerBuilder::default();

        let mut layered_annotated_polygons: Vec<Vec<AnnotatedPolygon>> = self
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
                        bucket,
                        color: colorer.color(bucket).ok()?,
                        tags: tags.clone(),
                    })
                })
                .collect::<Vec<_>>())
            })
            .collect::<Result<_, MapError>>()?;

        // first compute current bbox and shift everything positive.
        let shift = streaming_bbox(
            layered_annotated_polygons
                .iter()
                .flatten()
                .map(|AnnotatedPolygon { polygon, .. }| polygon),
        )?
        .bl_bound();
        // then rotate (doesn't affect pan+scan later)
        // then scale up to hit frame.
        // center the whole thing.

        let layers: Vec<Vec<ColoredPolygon>> = vec![];
        let output_files = layers
            .iter()
            .enumerate()
            .map(|(idx, _layer)| format!("{}_{}.svg", self.output_file_prefix, idx))
            .collect::<Vec<_>>();
        write_layers_to_svgs((1024.0, 1024.0), output_files, layers.into_iter())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn test_render() {
        let tmp_dir = TempDir::new("example").unwrap();

        Map::new_from_file(
            "testdata/example.geojson",
            tmp_dir.path().as_os_str().to_string_lossy().to_string(),
        )
        .unwrap()
        .render()
        .unwrap();
    }
}
