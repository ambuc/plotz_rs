use crate::bucket::Subway;

use {
    crate::bucket::{Area, Bucket, Path},
    thiserror::Error,
    tracing::*,
};

pub trait Bucketer {
    type Tag;
    type Bucket;
    type Error;
    /// Given a set of tags, sort into a bucket or return an error.
    fn bucket(&self, tags: Self::Tag) -> Result<Self::Bucket, Self::Error>;
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum BucketerError {
    #[error("could not bucket")]
    BucketerError,
}

pub trait Bucketer2 {
    type Tag;
    type Bucket;
    type Error;
    /// Given a set of tags, sort into bucket or return error.
    fn bucket(&self, tags: &[Self::Tag]) -> Result<Self::Bucket, Self::Error>;
}

#[derive(Debug, PartialEq, Eq)]
enum Seq {
    AnyOf(Vec<(&'static str, &'static str)>),
    AllOf(Vec<(&'static str, &'static str)>),
}

pub struct DefaultBucketer2 {}

impl Bucketer2 for DefaultBucketer2 {
    type Tag = (String, String);
    type Bucket = Bucket;
    type Error = BucketerError;
    fn bucket(&self, tags: &[Self::Tag]) -> Result<Self::Bucket, Self::Error> {
        if tags
            .iter()
            .any(|(k, v)| matches!((k.as_str(), v.as_str()), ("natural", "beach" | "sand")))
        {
            Ok(Bucket::Area(Area::Beach))
        } else if tags.iter().any(|(k, v)| {
            matches!(
                (k.as_str(), v.as_str()),
                ("building", "apartments" | "garages" | "yes")
                    | ("landuse", "commercial" | "construction" | "industrial")
            )
        }) {
            Ok(Bucket::Area(Area::Building))
        } else if tags.iter().any(|(k, v)| {
            matches!(
                (k.as_str(), v.as_str()),
                ("amenity", "school")
                    | ("fitness_station", "box")
                    | ("leisure", "pitch" | "playground" | "swimming_pool")
            )
        }) {
            Ok(Bucket::Area(Area::Fun))
        } else if tags.iter().any(|(k, v)| {
            matches!(
                (k.as_str(), v.as_str()),
                (
                    "landuse",
                    "brownfield" | "cemetery" | "grass" | "greenfield" | "meadow"
                ) | ("leisure", "garden" | "park")
                    | ("natural", "scrub")
            )
        }) {
            Ok(Bucket::Area(Area::Park))
        } else if tags
            .iter()
            .any(|(k, v)| matches!((k.as_str(), v.as_str()), ("natural", "bare_rock")))
        {
            Ok(Bucket::Area(Area::NaturalRock))
        } else if tags
            .iter()
            .any(|(k, v)| matches!((k.as_str(), v.as_str()), ("landuse", "railway")))
        {
            Ok(Bucket::Area(Area::Rail))
        } else if tags
            .iter()
            .any(|(k, v)| matches!((k.as_str(), v.as_str()), ("natural", "tree")))
        {
            Ok(Bucket::Area(Area::Tree))
        } else if tags.iter().any(|(k, v)| {
            matches!(
                (k.as_str(), v.as_str()),
                ("natural", "bay" | "coastline" | "water")
            )
        }) {
            Ok(Bucket::Area(Area::Water))
        } else if tags
            .iter()
            .any(|(k, v)| matches!((k.as_str(), v.as_str()), ("boundary", "administrative")))
        {
            Ok(Bucket::Path(Path::Boundary))
        } else if tags
            .iter()
            .any(|(k, v)| matches!((k.as_str(), v.as_str()), ("highway", "cycleway")))
        {
            Ok(Bucket::Path(Path::Cycleway))
        } else if tags
            .iter()
            .any(|(k, v)| matches!((k.as_str(), v.as_str()), ("highway", "primary")))
        {
            Ok(Bucket::Path(Path::Highway1))
        } else if tags
            .iter()
            .any(|(k, v)| matches!((k.as_str(), v.as_str()), ("highway", "secondary")))
        {
            Ok(Bucket::Path(Path::Highway2))
        } else if tags
            .iter()
            .any(|(k, v)| matches!((k.as_str(), v.as_str()), ("highway", "tertiary")))
        {
            Ok(Bucket::Path(Path::Highway3))
        } else if tags.iter().any(|(k, v)| {
            matches!(
                (k.as_str(), v.as_str()),
                ("highway", "primary_link" | "secondary_link" | "service")
            )
        }) {
            Ok(Bucket::Path(Path::Highway4))
        } else if tags.iter().any(|(k, v)| {
            matches!(
                (k.as_str(), v.as_str()),
                (
                    "highway",
                    "footway" | "pedestrian" | "residential" | "steps"
                )
            )
        }) {
            Ok(Bucket::Path(Path::Pedestrian))
        } else {
            // subways
            for (seq, b) in [
                //
                (
                    Seq::AllOf(vec![("route", "subway"), ("ref", "A")]),
                    Bucket::Path(Path::Subway(Subway::_ACE)),
                ),
                (
                    Seq::AllOf(vec![("route", "subway"), ("ref", "C")]),
                    Bucket::Path(Path::Subway(Subway::_ACE)),
                ),
                (
                    Seq::AllOf(vec![("route", "subway"), ("ref", "E")]),
                    Bucket::Path(Path::Subway(Subway::_ACE)),
                ),
                //
                (
                    Seq::AllOf(vec![("route", "subway"), ("ref", "B")]),
                    Bucket::Path(Path::Subway(Subway::_BDFM)),
                ),
                (
                    Seq::AllOf(vec![("route", "subway"), ("ref", "D")]),
                    Bucket::Path(Path::Subway(Subway::_BDFM)),
                ),
                (
                    Seq::AllOf(vec![("route", "subway"), ("ref", "F")]),
                    Bucket::Path(Path::Subway(Subway::_BDFM)),
                ),
                (
                    Seq::AllOf(vec![("route", "subway"), ("ref", "M")]),
                    Bucket::Path(Path::Subway(Subway::_BDFM)),
                ),
                //
                (
                    Seq::AllOf(vec![("route", "subway"), ("ref", "G")]),
                    Bucket::Path(Path::Subway(Subway::_G)),
                ),
                (
                    Seq::AllOf(vec![("route", "subway"), ("ref", "L")]),
                    Bucket::Path(Path::Subway(Subway::_L)),
                ),
                (
                    Seq::AllOf(vec![("route", "subway"), ("ref", "J")]),
                    Bucket::Path(Path::Subway(Subway::_JZ)),
                ),
                (
                    Seq::AllOf(vec![("route", "subway"), ("ref", "Z")]),
                    Bucket::Path(Path::Subway(Subway::_JZ)),
                ),
                //
                (
                    Seq::AllOf(vec![("route", "subway"), ("ref", "N")]),
                    Bucket::Path(Path::Subway(Subway::_NQRW)),
                ),
                (
                    Seq::AllOf(vec![("route", "subway"), ("ref", "Q")]),
                    Bucket::Path(Path::Subway(Subway::_NQRW)),
                ),
                (
                    Seq::AllOf(vec![("route", "subway"), ("ref", "R")]),
                    Bucket::Path(Path::Subway(Subway::_NQRW)),
                ),
                (
                    Seq::AllOf(vec![("route", "subway"), ("ref", "W")]),
                    Bucket::Path(Path::Subway(Subway::_NQRW)),
                ),
                //
                (
                    Seq::AllOf(vec![("route", "subway"), ("ref", "1")]),
                    Bucket::Path(Path::Subway(Subway::_123)),
                ),
                (
                    Seq::AllOf(vec![("route", "subway"), ("ref", "2")]),
                    Bucket::Path(Path::Subway(Subway::_123)),
                ),
                (
                    Seq::AllOf(vec![("route", "subway"), ("ref", "3")]),
                    Bucket::Path(Path::Subway(Subway::_123)),
                ),
                //
                (
                    Seq::AllOf(vec![("route", "subway"), ("ref", "4")]),
                    Bucket::Path(Path::Subway(Subway::_456)),
                ),
                (
                    Seq::AllOf(vec![("route", "subway"), ("ref", "5")]),
                    Bucket::Path(Path::Subway(Subway::_456)),
                ),
                (
                    Seq::AllOf(vec![("route", "subway"), ("ref", "6")]),
                    Bucket::Path(Path::Subway(Subway::_456)),
                ),
                //
                (
                    Seq::AllOf(vec![("route", "subway"), ("ref", "7")]),
                    Bucket::Path(Path::Subway(Subway::_7)),
                ),
                (
                    Seq::AllOf(vec![("route", "subway"), ("ref", "T")]),
                    Bucket::Path(Path::Subway(Subway::_T)),
                ),
                (
                    Seq::AllOf(vec![("route", "subway"), ("ref", "S")]),
                    Bucket::Path(Path::Subway(Subway::_S)),
                ),
                (
                    Seq::AnyOf(vec![("railway", "rail"), ("route", "subway")]),
                    Bucket::Path(Path::Rail),
                ),
            ] {
                if match seq {
                    Seq::AllOf(seq) => seq.iter().all(|tag| {
                        tags.iter()
                            .any(|found| found.0 == tag.0 && found.1 == tag.1)
                    }),
                    Seq::AnyOf(seq) => seq.iter().any(|tag| {
                        tags.iter()
                            .any(|found| found.0 == tag.0 && found.1 == tag.1)
                    }),
                } {
                    return Ok(b);
                }
            }

            Err(BucketerError::BucketerError)
        }
    }
}

#[cfg(test)]
mod test_super {
    use super::*;

    #[test]
    fn test_bucket() {
        let bucketer = DefaultBucketer2 {};
        assert_eq!(
            bucketer.bucket(&vec![("natural".to_string(), "sand".to_string())]),
            Ok(Bucket::Area(Area::Beach))
        );
        assert_eq!(
            bucketer.bucket(&vec![("natural".to_string(), "".to_string())]),
            Err(BucketerError::BucketerError)
        );
    }
}
