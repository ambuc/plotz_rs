use crate::{
    bucket::Bucket,
    bucketer::{Bucketer, DefaultBucketer},
    colorer::{Colorer, DefaultColorer},
    colorer_builder::DefaultColorerBuilder,
};
use plotz_color::ColorRGB;
use plotz_geojson::GeoJsonConversionError;
use plotz_geometry::polygon::Polygon;
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
}

pub struct AnnotatedPolygon {
    polygon: Polygon,
    bucket: Bucket,
    color: ColorRGB,
    tags: Vec<(SymbolU32, SymbolU32)>,
}

pub struct Map {
    files: Vec<File>,
}

impl Map {
    pub fn new_from_files(file_paths: Vec<&str>) -> Result<Map, MapError> {
        let mut files = vec![];
        for fp in file_paths {
            files.push(File::open(fp)?);
        }
        Ok(Map { files })
    }
    pub fn new_from_file(file_path: &str) -> Result<Map, MapError> {
        Self::new_from_files(vec![file_path])
    }

    pub fn render(&self) -> Result<Vec<Vec<Polygon>>, MapError> {
        let mut interner = StringInterner::new();
        let bucketer = DefaultBucketer::new(&mut interner);
        let colorer: DefaultColorer = DefaultColorerBuilder::default();

        let mut layered_annotated_polygons: Vec<Vec<AnnotatedPolygon>> = vec![];

        for file in self.files.iter() {
            let annotated_polygons: Vec<AnnotatedPolygon> = plotz_geojson::parse_geojson(
                &mut interner,
                serde_json::from_reader(BufReader::new(file))?,
            )?
            .iter()
            .filter_map(|(polygon, tags): &(Polygon, Vec<(SymbolU32, SymbolU32)>)| -> Option<AnnotatedPolygon> {
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
            .collect();
            layered_annotated_polygons.push(annotated_polygons);
        }

        let layers: Vec<Vec<Polygon>> = vec![];

        Ok(layers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render() {
        let polygons = Map::new_from_file("testdata/example.geojson")
            .unwrap()
            .render()
            .unwrap();
        // one layer
        assert_eq!(polygons.len(), 0);
    }
}
