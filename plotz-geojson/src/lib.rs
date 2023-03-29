#![deny(missing_docs)]

//! A crate for reading GeoJSON files and parsing them to plotz_geometry
//! structs.

use {
    plotz_geometry::{
        point::Pt,
        polygon::{Multiline, MultilineConstructorError, Polygon, PolygonConstructorError},
    },
    serde_json::Value,
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

/// Parses a GeoJSON file and returns a list of tagged polygons.
pub fn parse_geojson(geo_json: Value) -> Result<Vec<(Polygon, TagsList)>, GeoJsonConversionError> {
    let features = geo_json["features"].as_array().expect("features not array");

    info!("Parsing geojson file with {:?} features.", features.len());
    let mut lines: Vec<(Polygon, TagsList)> = vec![];

    for (_idx, feature) in features.iter().enumerate() {
        let tags = feature["properties"]
            .as_object()
            .expect("not object")
            .into_iter()
            .filter(|(_k, v)| v.as_str().is_some())
            .map(|(k, v)| (k.clone(), v.as_str().unwrap().to_string()))
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

        let polygons = parse_geojson(serde_json::from_reader(reader).unwrap()).unwrap();

        assert_eq!(polygons.len(), 4);
        assert_eq!(
            polygons[0].0,
            Polygon([Pt(0, 0), Pt(1.0, 2.5), Pt(2.0, 5.0)]).unwrap()
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
            Multiline([Pt(1, 1), Pt(1.0, 2.5), Pt(2.0, 5.0)]).unwrap()
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
            Polygon([Pt(2, 2), Pt(1.0, 2.5), Pt(2.0, 5.0)]).unwrap()
        );

        assert_eq!(
            polygons[3].0,
            Polygon([Pt(3, 3), Pt(1.0, 2.5), Pt(2.0, 5.0)]).unwrap()
        );
    }
}
