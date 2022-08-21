#![deny(missing_docs)]

//! A crate for reading GeoJSON files and parsing them to plotz_geometry
//! structs.

use {
    lazy_static,
    log::info,
    plotz_geometry::{
        point::Pt,
        polygon::{Multiline, MultilineConstructorError, Polygon, PolygonConstructorError},
    },
    serde_json::Value,
    std::collections::{HashMap, HashSet},
    string_interner::{symbol::SymbolU32, StringInterner},
    thiserror::Error,
};

type KeySymbol = SymbolU32;
type ValueSymbol = SymbolU32;
type TagsMap = HashMap<KeySymbol, ValueSymbol>;
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
           "bare_rock",
           "bay",
           "beach",
           "boundary",
           "box",
           "brownfield",
           "cemetery",
           "coastline",
           "commercial",
           "construction",
           "cycleway",
           "fitness_station",
           "footway",
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
        ])
    };
}

/// Parses a GeoJSON file and returns a list of tagged polygons.
pub fn parse_geojson(
    interner: &mut StringInterner,
    geo_json: Value,
) -> Result<Vec<(Polygon, TagsList)>, GeoJsonConversionError> {
    let mut lines: Vec<(Polygon, TagsList)> = vec![];

    for (idx, feature) in geo_json["features"]
        .as_array()
        .expect("features not array")
        .iter()
        .enumerate()
    {
        let mut tags = TagsMap::new();

        for (k, v) in feature["properties"].as_object().expect("not obj") {
            if !INTERESTING_PROPERTIES.contains(k.as_str()) {
                continue;
            }
            if let Some(val_str) = v.as_str() {
                if !INTERESTING_PROPERTIES.contains(val_str) {
                    continue;
                }
                tags.insert(interner.get_or_intern(k), interner.get_or_intern(val_str));
            }
        }
        let tags_list: TagsList = tags.iter().map(|(k, v)| (*k, *v)).collect();

        info!(
            "parsing feature {:?} with tags {:?}",
            idx,
            tags_list
                .iter()
                .map(|(k, v)| (interner.resolve(*k).unwrap(), interner.resolve(*v).unwrap()))
                .collect::<Vec<_>>()
        );

        let geom_type: &str = feature["geometry"]["type"]
            .as_str()
            .expect("type not string");

        let coords = &feature["geometry"]["coordinates"];
        for polygon in match geom_type {
            "LineString" => parse_to_linestring(coords)?,
            "MultiLineString" => parse_to_multilinestring(coords)?,
            "Polygon" => parse_to_polygon(coords)?,
            "MultiPolygon" => parse_to_multipolygon(coords)?,
            "Point" => vec![],
            other @ _ => {
                unimplemented!("other: {:?}", other);
            }
        } {
            lines.push((polygon, tags_list.clone()));
        }
    }
    Ok(lines)
}

fn parse_to_linestring<'a>(coordinates: &Value) -> Result<Vec<Polygon>, GeoJsonConversionError> {
    let mut lines: Vec<_> = vec![];
    match coordinates {
        Value::Array(pts) => {
            if let Ok(ml) = Multiline(pts.iter().map(|p| {
                Pt(
                    p[0].as_f64().expect("value not f64"),
                    p[1].as_f64().expect("value not f64"),
                )
            })) {
                lines.push(ml);
            }
        }
        _ => {
            unimplemented!("?");
        }
    }
    Ok(lines)
}

fn parse_to_multilinestring<'a>(
    coordinates: &Value,
) -> Result<Vec<Polygon>, GeoJsonConversionError> {
    let mut lines: Vec<Polygon> = vec![];
    match coordinates {
        Value::Array(linestrings) => {
            for linestring in linestrings.iter() {
                lines.append(&mut parse_to_linestring(linestring)?);
            }
        }
        _ => unimplemented!("?"),
    }
    Ok(lines)
}

fn parse_to_multipolygon<'a>(coordinates: &Value) -> Result<Vec<Polygon>, GeoJsonConversionError> {
    let mut lines: Vec<_> = vec![];
    match coordinates {
        Value::Array(coordinates_list) => {
            for coordinates in coordinates_list {
                lines.extend(parse_to_polygon(coordinates)?);
            }
        }
        _ => {
            unimplemented!("?");
        }
    }
    Ok(lines)
}

fn parse_to_polygon<'a>(coordinates: &Value) -> Result<Vec<Polygon>, GeoJsonConversionError> {
    let mut lines: Vec<_> = vec![];
    match coordinates {
        Value::Array(points_lists) => {
            for points_list in points_lists {
                match points_list {
                    Value::Array(pts) => {
                        if let Ok(p) = Polygon(pts.iter().map(|p| {
                            Pt(
                                p[0].as_f64().expect("value not f64"),
                                p[1].as_f64().expect("value not f64"),
                            )
                        })) {
                            lines.push(p);
                        }
                    }
                    _ => {
                        unimplemented!("?");
                    }
                }
            }
        }
        _ => {
            unimplemented!("?");
        }
    }

    Ok(lines)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use string_interner::symbol::SymbolU32;

    fn assert_symbol(i: &StringInterner, s: SymbolU32, expected: &str) {
        assert_eq!(i.resolve(s).unwrap(), expected);
    }

    fn assert_symbol_tuple<'a>(
        i: &StringInterner,
        (s1, s2): (SymbolU32, SymbolU32),
        (e1, e2): (&str, &str),
    ) {
        assert_symbol(i, s1, e1);
        assert_symbol(i, s2, e2);
    }

    fn assert_symbol_tuple_list<'a>(
        i: &StringInterner,
        mut symbol_tuple_list: Vec<(SymbolU32, SymbolU32)>,
        expected_list: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) {
        symbol_tuple_list.sort();
        for (ss, es) in symbol_tuple_list.into_iter().zip(expected_list.into_iter()) {
            assert_symbol_tuple(i, ss, es);
        }
    }

    #[test]
    fn test_parse_to_polygon() {
        let geojson = json!([[
            [-74.0156511, 40.7215446],
            [-74.0154939, 40.7215262],
            [-74.0142809, 40.7213844],
            [-74.0142481, 40.7213806],
            [-74.0132831, 40.7212678],
        ]]);
        assert_eq!(
            parse_to_polygon(&geojson).unwrap(),
            vec![Polygon([
                Pt(-74.0156511, 40.7215446),
                Pt(-74.0154939, 40.7215262),
                Pt(-74.0142809, 40.7213844),
                Pt(-74.0142481, 40.7213806),
                Pt(-74.0132831, 40.7212678),
            ])
            .unwrap()]
        );
    }

    #[test]
    fn test_parse_to_linestring() {
        let geojson = json!([
            [-74.0156511, 40.7215446],
            [-74.0154939, 40.7215262],
            [-74.0142809, 40.7213844],
            [-74.0142481, 40.7213806],
            [-74.0132831, 40.7212678],
        ]);
        assert_eq!(
            parse_to_linestring(&geojson).unwrap(),
            vec![Multiline([
                Pt(-74.0156511, 40.7215446),
                Pt(-74.0154939, 40.7215262),
                Pt(-74.0142809, 40.7213844),
                Pt(-74.0142481, 40.7213806),
                Pt(-74.0132831, 40.7212678),
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
            &mut interner,
            polygons[0].1.clone(),
            [
                ("@id", "relation/2389611"),
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
            &mut interner,
            polygons[1].1.clone(),
            [
                ("@id", "way/5668999"),
                ("cycleway", "shared_lane"),
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
