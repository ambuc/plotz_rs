#![deny(missing_docs)]

//! A crate for reading GeoJSON files and parsing them to plotz_geometry
//! structs.

use {
    plotz_geometry::{
        point::Pt,
        polygon::{Multiline, MultilineConstructorError, Polygon, PolygonConstructorError},
    },
    serde_json::Value,
    std::collections::HashSet,
    string_interner::{symbol::SymbolU32, StringInterner},
    thiserror::Error,
    tracing::*,
};

type KeySymbol = SymbolU32;
type ValueSymbol = SymbolU32;
type TagsList = Vec<(KeySymbol, ValueSymbol)>;

/// A general error arising from converting GeoJSON to plotz-geometry Polygons.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum GeoJsonConversionError {
    /// Could not create a multiline.
    #[error("Could not create a multiline.")]
    MultilineConstructorError(#[from] MultilineConstructorError),
    /// Could not create a polygon.
    #[error("Could not create a polygon.")]
    PolygonConstructorError(#[from] PolygonConstructorError),
    /// Listed coordinates are not an array.
    #[error("Listed coordinates are not an array.")]
    CoordinatesNotArray,
}

lazy_static::lazy_static! {
    /// A set of known interesting keys.
    pub static ref INTERESTING_PROPERTIES: HashSet<&'static str> = {
        HashSet::from_iter([
           "administrative",
           "amenity",
           "apartments",
           "bare_rock",
           "bay",
           "beach",
           "boundary",
           "box",
           "brownfield",
           "building",
           "cemetery",
           "coastline",
           "commercial",
           "construction",
           "cycleway",
           "fitness_station",
           "footway",
           "garages",
           "garden",
           "grass",
           "greenfield",
           "highway",
           "industrial",
           "landuse",
           "leisure",
           "meadow",
           "natural",
           "park",
           "pedestrian",
           "pitch",
           "playground",
           "primary",
           "primary_link",
           "rail",
           "railway",
           "residential",
           "sand",
           "school",
           "scrub",
           "secondary",
           "secondary_link",
           "service",
           "steps",
           "swimming_pool",
           "tertiary",
           "tree",
           "water",
           "yes",
        ])
    };
}

/// Parses a GeoJSON file and returns a list of tagged polygons.
pub fn parse_geojson(
    interner: &mut StringInterner,
    geo_json: Value,
) -> Result<Vec<(Polygon, TagsList)>, GeoJsonConversionError> {
    let features = geo_json["features"].as_array().expect("features not array");

    info!("Parsing geojson file with {:?} features.", features.len());
    let mut lines: Vec<(Polygon, TagsList)> = vec![];

    for (_idx, feature) in features.iter().enumerate() {
        let tags = feature["properties"]
            .as_object()
            .expect("not object")
            .into_iter()
            .filter(|(k, _v)| INTERESTING_PROPERTIES.contains(k.as_str()))
            .filter(|(_k, v)| v.as_str().is_some())
            .filter(|(_k, v)| INTERESTING_PROPERTIES.contains(v.as_str().unwrap()))
            .map(|(k, v)| {
                (
                    interner.get_or_intern(k),
                    interner.get_or_intern(v.as_str().unwrap()),
                )
            })
            .collect::<TagsList>();

        if tags.is_empty() {
            continue;
        }

        let geom_type: &str = feature["geometry"]["type"]
            .as_str()
            .expect("type not string");

        let coords = &feature["geometry"]["coordinates"];

        if let Ok(polygons) = match geom_type {
            "LineString" => parse_to_linestring(coords),
            "MultiLineString" => parse_to_multilinestring(coords),
            "Polygon" => parse_to_polygon(coords),
            "MultiPolygon" => parse_to_multipolygon(coords),
            "Point" => Ok(vec![]),
            other => {
                unimplemented!("other: {:?}", other);
            }
        } {
            for polygon in polygons {
                // trace!(
                //     "#{:?} ({:10}, {:2}pts) w/ {:?}",
                //     idx,
                //     geom_type,
                //     polygon.pts.len(),
                //     tags.iter()
                //         .map(|(k, v)| (
                //             interner.resolve(*k).unwrap(),
                //             interner.resolve(*v).unwrap()
                //         ))
                //         .collect::<Vec<_>>(),
                // );
                lines.push((polygon, tags.clone()));
            }
        }
    }
    Ok(lines)
}

fn parse_to_linestring(coordinates: &Value) -> Result<Vec<Polygon>, GeoJsonConversionError> {
    Ok(vec![Multiline(
        coordinates.as_array().expect("not array").iter().map(|p| {
            Pt(
                p[0].as_f64().expect("value not f64"),
                p[1].as_f64().expect("value not f64"),
            )
        }),
    )?])
}

fn parse_to_multilinestring(coordinates: &Value) -> Result<Vec<Polygon>, GeoJsonConversionError> {
    let mut lines: Vec<Polygon> = vec![];
    for linestring in coordinates.as_array().expect("not array").iter() {
        lines.append(&mut parse_to_linestring(linestring)?);
    }
    Ok(lines)
}

fn parse_to_multipolygon(coordinates: &Value) -> Result<Vec<Polygon>, GeoJsonConversionError> {
    let mut lines: Vec<_> = vec![];
    for coordinates in coordinates.as_array().expect("not array") {
        lines.extend(parse_to_polygon(coordinates)?);
    }
    Ok(lines)
}

fn parse_to_polygon(coordinates: &Value) -> Result<Vec<Polygon>, GeoJsonConversionError> {
    Ok(coordinates
        .as_array()
        .expect("not array")
        .iter()
        .map(|points_list| {
            Polygon(points_list.as_array().expect("not array").iter().map(|p| {
                Pt(
                    p[0].as_f64().expect("value not f64"),
                    p[1].as_f64().expect("value not f64"),
                )
            }))
        })
        .collect::<Result<_, _>>()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;
    use itertools::Itertools;
    use serde_json::json;
    use string_interner::symbol::SymbolU32;

    fn assert_symbol_tuple_list<'a>(
        i: &StringInterner,
        symbol_tuple_list: Vec<(SymbolU32, SymbolU32)>,
        expected_list: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) {
        for ((s1, s2), (e1, e2)) in symbol_tuple_list
            .into_iter()
            .sorted()
            .zip(expected_list.into_iter())
        {
            assert_eq!(assert_matches!(i.resolve(s1), Some(a) => a), e1);
            assert_eq!(assert_matches!(i.resolve(s2), Some(a) => a), e2);
        }
    }

    #[test]
    fn test_parse_to_polygon() {
        let geojson = json!([[
            [-74.015_651_1, 40.721_544_6],
            [-74.015_493_9, 40.721_526_2],
            [-74.014_280_9, 40.721_384_4],
            [-74.014_248_1, 40.721_380_6],
            [-74.013_283_1, 40.721_267_8],
        ]]);
        assert_eq!(
            parse_to_polygon(&geojson).unwrap(),
            vec![Polygon([
                Pt(-74.015_651_1, 40.721_544_6),
                Pt(-74.015_493_9, 40.721_526_2),
                Pt(-74.014_280_9, 40.721_384_4),
                Pt(-74.014_248_1, 40.721_380_6),
                Pt(-74.013_283_1, 40.721_267_8),
            ])
            .unwrap()]
        );
    }

    #[test]
    fn test_parse_to_linestring() {
        let geojson = json!([
            [-74.015_651_1, 40.721_544_6],
            [-74.015_493_9, 40.721_526_2],
            [-74.014_280_9, 40.721_384_4],
            [-74.014_248_1, 40.721_380_6],
            [-74.013_283_1, 40.721_267_8],
        ]);
        assert_eq!(
            parse_to_linestring(&geojson).unwrap(),
            vec![Multiline([
                Pt(-74.015_651_1, 40.721_544_6),
                Pt(-74.015_493_9, 40.721_526_2),
                Pt(-74.014_280_9, 40.721_384_4),
                Pt(-74.014_248_1, 40.721_380_6),
                Pt(-74.013_283_1, 40.721_267_8),
            ])
            .unwrap()]
        );
    }

    #[test]
    fn test_parse_real_geojson() {
        let file = std::fs::File::open("testdata/example.geojson").unwrap();
        let reader = std::io::BufReader::new(file);
        let mut interner = StringInterner::new();

        let polygons =
            parse_geojson(&mut interner, serde_json::from_reader(reader).unwrap()).unwrap();

        assert_eq!(polygons.len(), 4);
        assert_eq!(
            polygons[0].0,
            Polygon([Pt(0, 0), Pt(1.0, 2.5), Pt(2.0, 5.0)]).unwrap()
        );

        assert_symbol_tuple_list(
            &interner,
            polygons[0].1.clone(),
            [
                ("natural", "water"),
                ("water", "river"),
                ("type", "multipolygon"),
            ],
        );

        assert_eq!(
            polygons[1].0,
            Multiline([Pt(1, 1), Pt(1.0, 2.5), Pt(2.0, 5.0)]).unwrap()
        );
        assert_symbol_tuple_list(
            &interner,
            polygons[1].1.clone(),
            [
                ("highway", "residential"),
                ("lanes", "1"),
                ("maxspeed", "25 mph"),
                ("name", "Old Slip"),
                ("oneway", "yes"),
                ("surface", "asphalt"),
            ],
        );

        assert_eq!(
            polygons[2].0,
            Polygon([Pt(2, 2), Pt(1.0, 2.5), Pt(2.0, 5.0)]).unwrap()
        );

        assert_eq!(
            polygons[3].0,
            Polygon([Pt(3, 3), Pt(1.0, 2.5), Pt(2.0, 5.0)]).unwrap()
        );
    }
}
