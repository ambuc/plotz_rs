use plotz_geojson::GeoJsonConversionError;
use plotz_geometry::polygon::Polygon;
use std::{fs::File, io::BufReader};
use string_interner::StringInterner;
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

pub struct Map {
    file_path: String,
}

impl Map {
    pub fn new(file_path: &str) -> Map {
        Map {
            file_path: file_path.to_string(),
        }
    }

    pub fn render(&self) -> Result<Vec<Vec<Polygon>>, MapError> {
        let mut interner = StringInterner::new();
        let polygons = plotz_geojson::parse_geojson(
            &mut interner,
            serde_json::from_reader(BufReader::new(File::open(&self.file_path)?))?,
        )?;

        Ok(vec![polygons.iter().map(|(p, _)| p.clone()).collect()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render() {
        let polygons = Map::new("testdata/example.geojson").render().unwrap();
        // one layer
        assert_eq!(polygons.len(), 1);
    }
}
