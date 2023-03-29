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
        for (seq, b) in [
            (
                Seq::AnyOf(vec![("natural", "beach"), ("natural", "sand")]),
                Bucket::Area(Area::Beach),
            ),
            (
                Seq::AnyOf(vec![
                    ("building", "apartments"),
                    ("building", "garages"),
                    ("building", "yes"),
                    ("landuse", "commercial"),
                    ("landuse", "construction"),
                    ("landuse", "industrial"),
                ]),
                Bucket::Area(Area::Building),
            ),
            (
                Seq::AnyOf(vec![
                    ("amenity", "school"),
                    ("fitness_station", "box"),
                    ("leisure", "pitch"),
                    ("leisure", "playground"),
                    ("leisure", "swimming_pool"),
                ]),
                Bucket::Area(Area::Fun),
            ),
            (
                Seq::AnyOf(vec![("natural", "bare_rock")]),
                Bucket::Area(Area::NaturalRock),
            ),
            (
                Seq::AnyOf(vec![
                    ("landuse", "brownfield"),
                    ("landuse", "cemetery"),
                    ("landuse", "grass"),
                    ("landuse", "greenfield"),
                    ("landuse", "meadow"),
                    ("leisure", "garden"),
                    ("leisure", "park"),
                    ("natural", "scrub"),
                ]),
                Bucket::Area(Area::Park),
            ),
            (
                Seq::AnyOf(vec![("landuse", "railway")]),
                Bucket::Area(Area::Rail),
            ),
            (
                Seq::AnyOf(vec![("natural", "tree")]),
                Bucket::Area(Area::Tree),
            ),
            (
                Seq::AnyOf(vec![
                    ("natural", "bay"),
                    ("natural", "coastline"),
                    ("natural", "water"),
                ]),
                Bucket::Area(Area::Water),
            ),
            (
                Seq::AnyOf(vec![("boundary", "administrative")]),
                Bucket::Path(Path::Boundary),
            ),
            (
                Seq::AnyOf(vec![("highway", "cycleway")]),
                Bucket::Path(Path::Cycleway),
            ),
            (
                Seq::AnyOf(vec![("highway", "primary")]),
                Bucket::Path(Path::Highway1),
            ),
            (
                Seq::AnyOf(vec![("highway", "secondary")]),
                Bucket::Path(Path::Highway2),
            ),
            (
                Seq::AnyOf(vec![("highway", "tertiary")]),
                Bucket::Path(Path::Highway3),
            ),
            (
                Seq::AnyOf(vec![
                    ("highway", "primary_link"),
                    ("highway", "secondary_link"),
                    ("highway", "service"),
                ]),
                Bucket::Path(Path::Highway4),
            ),
            (
                Seq::AnyOf(vec![
                    ("highway", "footway"),
                    ("highway", "pedestrian"),
                    ("highway", "residential"),
                    ("highway", "steps"),
                ]),
                Bucket::Path(Path::Pedestrian),
            ),
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
        println!("Skipping polygon with tags: {:?}", tags);

        Err(BucketerError::BucketerError)
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
