//! A crate for reading shp files and parsing them to plotz_geometry
//! structs.

#![allow(missing_docs)]
#![allow(unused)]

use {
    plotz_geometry::shapes::polygon::Polygon,
    shapefile::{Reader, Shape},
    std::path::Path,
    string_interner::{symbol::SymbolU32, StringInterner},
    thiserror::Error,
};

type KeySymbol = SymbolU32;
type ValueSymbol = SymbolU32;
type TagsList = Vec<(KeySymbol, ValueSymbol)>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RecordError {
    #[error("Encountered null record")]
    NullRecord,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PolygonError {
    #[error("Encountered null shape")]
    NullShape,
    #[error("Encountered shape not yet handled")]
    NotYetHandled,
}

/// A general error arising from converting GeoJSON to plotz-geometry Polygons.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ShpConversionError {
    #[error("Could not convert a polygon")]
    PolygonError(PolygonError),
    #[error("Could not convert a record")]
    RecordError(RecordError),
}

pub fn parse_shp<P: AsRef<Path>>(
    interner: &mut StringInterner,
    shp_path: P,
) -> Result<Vec<(Polygon, TagsList)>, ShpConversionError> {
    let mut reader = Reader::from_path(shp_path).expect("quz");
    let retval = vec![];
    for (shape, record) in reader.iter_shapes_and_records().filter_map(Result::ok) {
        // let polygon = to_polygon(&shape).expect("bar");
        // let tagslist = to_tagslist(&record).expect("baz");
        // retval.push((polygon, tagslist));
    }
    Ok(retval)
}

fn to_polygon(shape: &Shape) -> Result<Option<Polygon>, PolygonError> {
    match shape {
        Shape::NullShape => Err(PolygonError::NullShape),
        Shape::Point(_) | Shape::PointM(_) | Shape::PointZ(_) => Ok(None),
        Shape::PolygonM(_)
        | Shape::PolygonZ(_)
        | Shape::Multipoint(_)
        | Shape::MultipointM(_)
        | Shape::MultipointZ(_)
        | Shape::Multipatch(_)
        | Shape::PolylineM(_)
        | Shape::PolylineZ(_) => Err(PolygonError::NotYetHandled),
        Shape::Polyline(pl) => unimplemented!(),
        _ => unimplemented!(),
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_foo() {
        //
    }
}
