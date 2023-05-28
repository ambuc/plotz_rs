#![deny(missing_docs)]

//! A crate for reading GeoJSON files and parsing them to plotz_geometry
//! structs.

use {
    plotz_geometry::{
        obj2::Obj2,
        p2,
        shapes::{
            pg2::{
                multiline::{Multiline, MultilineConstructorError},
                Pg2, PolygonConstructorError, TryPolygon,
            },
            pt2::Pt2,
        },
    },
    serde_json::Value,
    std::collections::HashMap,
    thiserror::Error,
    tracing::*,
};

type KeySymbol = String;
type ValueSymbol = String;
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

#[derive(Debug, PartialEq, Eq, Hash)]
enum GeomType {
    LineString,
    MultiLineString,
    Polygon,
    MultiPolygon,
    Point,
}

fn add_tags(value: &Value, tagslist: &mut TagsList) {
    if let Some(obj) = value.as_object() {
        for (k, v) in obj {
            match v {
                Value::String(v) => tagslist.push((k.to_string(), v.to_string())),
                Value::Object(_obj) => {
                    add_tags(v, tagslist);
                }
                Value::Array(arr) => {
                    for v in arr {
                        // BUG: this should be parsing as separate objects.
                        add_tags(v, tagslist);
                    }
                }
                Value::Null | Value::Bool(_) | Value::Number(_) => {
                    //
                }
            }
        }
    }
}

/// Parses aGeoJSON file and returns a list of tagged polygons.
pub fn parse_geojson(geo_json: Value) -> Result<Vec<(Obj2, TagsList)>, GeoJsonConversionError> {
    let features = geo_json["features"].as_array().expect("features not array");

    info!("Parsing geojson file with {:?} features.", features.len());
    let mut lines: Vec<(Obj2, TagsList)> = vec![];

    let mut stats = HashMap::<GeomType, usize>::new();

    for (_idx, feature) in features.iter().enumerate() {
        let mut tags: TagsList = vec![];

        add_tags(&feature["properties"], &mut tags);

        if tags.is_empty() {
            continue;
        }

        let geom_type: &str = feature["geometry"]["type"]
            .as_str()
            .expect("type not string");

        let coords = &feature["geometry"]["coordinates"];

        let result: Result<Vec<Obj2>, GeoJsonConversionError> = match geom_type {
            "LineString" => parse_to_linestring(coords).map(|v| {
                stats
                    .entry(GeomType::LineString)
                    .and_modify(|e| *e += 1)
                    .or_insert(1);
                v
            }),
            "MultiLineString" => parse_to_multilinestring(coords).map(|v| {
                stats
                    .entry(GeomType::MultiLineString)
                    .and_modify(|e| *e += 1)
                    .or_insert(1);
                v
            }),
            "Polygon" => parse_to_polygon(coords).map(|v| {
                stats
                    .entry(GeomType::Polygon)
                    .and_modify(|e| *e += 1)
                    .or_insert(1);
                v
            }),
            "MultiPolygon" => parse_to_multipolygon(coords).map(|v| {
                stats
                    .entry(GeomType::MultiPolygon)
                    .and_modify(|e| *e += 1)
                    .or_insert(1);
                v
            }),
            "Point" => parse_to_circle(coords).map(|v| {
                stats
                    .entry(GeomType::Point)
                    .and_modify(|e| *e += 1)
                    .or_insert(1);
                v
            }),
            other => {
                unimplemented!("other: {:?}", other);
            }
        };

        if let Ok(obj_inner_s) = result {
            for obj_inner in obj_inner_s {
                lines.push((obj_inner, tags.clone()));
            }
        }
        // else {
        //     error!("geojson parse failure: {:?}", result);
        // }
    }

    trace!("stats: {:?}", stats);
    Ok(lines)
}

fn parse_to_linestring(coordinates: &Value) -> Result<Vec<Obj2>, GeoJsonConversionError> {
    Ok(vec![Obj2::from(Multiline(
        coordinates.as_array().expect("not array").iter().map(|p| {
            p2!(
                p[0].as_f64().expect("value not f64"),
                p[1].as_f64().expect("value not f64")
            )
        }),
    )?)])
}

fn parse_to_multilinestring(coordinates: &Value) -> Result<Vec<Obj2>, GeoJsonConversionError> {
    let mut lines: Vec<Obj2> = vec![];
    for linestring in coordinates.as_array().expect("not array").iter() {
        lines.append(&mut parse_to_linestring(linestring)?);
    }
    Ok(lines)
}

fn parse_to_multipolygon(coordinates: &Value) -> Result<Vec<Obj2>, GeoJsonConversionError> {
    let mut lines: Vec<_> = vec![];
    for coordinates in coordinates.as_array().expect("not array") {
        lines.extend(parse_to_polygon(coordinates)?);
    }
    Ok(lines)
}

fn parse_to_polygon(coordinates: &Value) -> Result<Vec<Obj2>, GeoJsonConversionError> {
    Ok(coordinates
        .as_array()
        .expect("not array")
        .iter()
        .map(|points_list| {
            TryPolygon(points_list.as_array().expect("not array").iter().map(|p| {
                p2!(
                    p[0].as_f64().expect("value not f64"),
                    p[1].as_f64().expect("value not f64")
                )
            }))
        })
        .collect::<Result<_, _>>()?)
    .map(|v: Vec<Pg2>| v.into_iter().map(Obj2::from).collect::<Vec<_>>())
}

fn parse_to_circle(_coords: &Value) -> Result<Vec<Obj2>, GeoJsonConversionError> {
    // For now, don't print circles at all.
    Ok(vec![])
    // let array = &coords.as_array().expect("not array");
    // if let Some(x) = array.get(0).and_then(|o| o.as_f64()) {
    //     if let Some(y) = array.get(1).and_then(|o| o.as_f64()) {
    //         return Ok(vec![Object2d::from(CurveArc(
    //             p2!(x, y),
    //             0.0..=TAU,
    //             0.0001,
    //         ))]);
    //     }
    // }
    // Err(GeoJsonConversionError::CoordinatesNotArray)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // fn assert_symbol_tuple_list<'a>(
    //     symbol_tuple_list: Vec<(String, String)>,
    //     expected_list: impl IntoIterator<Item = (&'a str, &'a str)>,
    // ) {
    //     for ((s1, s2), (e1, e2)) in symbol_tuple_list
    //         .into_iter()
    //         .sorted()
    //         .zip(expected_list.into_iter())
    //     {
    //         assert_eq!(s1, e1);
    //         assert_eq!(s2, e2);
    //     }
    // }

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
            vec![Obj2::from(Pg2([
                p2!(-74.015_651_1, 40.721_544_6),
                p2!(-74.015_493_9, 40.721_526_2),
                p2!(-74.014_280_9, 40.721_384_4),
                p2!(-74.014_248_1, 40.721_380_6),
                p2!(-74.013_283_1, 40.721_267_8),
            ]))]
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
            vec![Obj2::from(
                Multiline([
                    p2!(-74.015_651_1, 40.721_544_6),
                    p2!(-74.015_493_9, 40.721_526_2),
                    p2!(-74.014_280_9, 40.721_384_4),
                    p2!(-74.014_248_1, 40.721_380_6),
                    p2!(-74.013_283_1, 40.721_267_8),
                ])
                .unwrap()
            )]
        );
    }

    #[test]
    fn test_parse_real_geojson() {
        let file = std::fs::File::open("testdata/example.geojson").unwrap();
        let reader = std::io::BufReader::new(file);

        let polygons = parse_geojson(serde_json::from_reader(reader).unwrap()).unwrap();

        assert_eq!(polygons.len(), 4);
        assert_eq!(
            polygons[0].0,
            Obj2::from(Pg2([p2!(0, 0), p2!(1.0, 2.5), p2!(2.0, 5.0)]))
        );

        // assert_symbol_tuple_list(
        //     polygons[0].1.clone(),
        //     [
        //         ("natural", "water"),
        //         ("water", "river"),
        //         ("type", "multipolygon"),
        //     ],
        // );

        assert_eq!(
            polygons[1].0,
            Obj2::from(Multiline([p2!(1, 1), p2!(1.0, 2.5), p2!(2.0, 5.0)]).unwrap())
        );
        // assert_symbol_tuple_list(
        //     polygons[1].1.clone(),
        //     [
        //         ("highway", "residential"),
        //         ("lanes", "1"),
        //         ("maxspeed", "25 mph"),
        //         ("name", "Old Slip"),
        //         ("oneway", "yes"),
        //         ("surface", "asphalt"),
        //     ],
        // );

        assert_eq!(
            polygons[2].0,
            Obj2::from(Pg2([p2!(2, 2), p2!(1.0, 2.5), p2!(2.0, 5.0)]))
        );

        assert_eq!(
            polygons[3].0,
            Obj2::from(Pg2([p2!(3, 3), p2!(1.0, 2.5), p2!(2.0, 5.0)]))
        );
    }
}
