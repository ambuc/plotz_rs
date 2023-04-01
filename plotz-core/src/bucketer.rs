use crate::bucket::Subway;

use crate::bucket::{Area, Bucket, Path};

pub trait Bucketer {
    type Tag;
    type Bucket;
    type Error;
    /// Given a set of tags, sort into a bucket or return an error.
    fn bucket(&self, tags: Self::Tag) -> Result<Self::Bucket, Self::Error>;
}

pub trait Bucketer2 {
    type Tag;
    type Bucket;
    /// Given a set of tags, sort into bucket or return error.
    fn bucket(&self, tags: &[Self::Tag]) -> Self::Bucket;
}

#[derive(Debug, PartialEq, Eq)]
enum Seq {
    AnyOf(Vec<(&'static str, &'static str)>),
    AllOf(Vec<(&'static str, &'static str)>),
}

pub struct DefaultBucketer2 {}

impl Bucketer2 for DefaultBucketer2 {
    type Tag = (String, String);
    type Bucket = Vec<Bucket>;
    fn bucket(&self, tags: &[Self::Tag]) -> Self::Bucket {
        let mut v = vec![];
        if tags
            .iter()
            .any(|(k, v)| matches!((k.as_str(), v.as_str()), ("natural", "beach" | "sand")))
        {
            v.push(Bucket::Area(Area::Beach));
        }
        if tags.iter().any(|(k, v)| {
            matches!(
                (k.as_str(), v.as_str()),
                ("building", "apartments" | "garages" | "yes")
                    | ("landuse", "commercial" | "construction" | "industrial")
            )
        }) {
            v.push(Bucket::Area(Area::Building));
        }
        if tags.iter().any(|(k, v)| {
            matches!(
                (k.as_str(), v.as_str()),
                ("amenity", "school")
                    | ("fitness_station", "box")
                    | ("leisure", "pitch" | "playground" | "swimming_pool")
            )
        }) {
            v.push(Bucket::Area(Area::Fun));
        }
        if tags.iter().any(|(k, v)| {
            matches!(
                (k.as_str(), v.as_str()),
                (
                    "landuse",
                    "brownfield" | "cemetery" | "grass" | "greenfield" | "meadow"
                ) | ("leisure", "garden" | "park")
                    | ("natural", "scrub")
            )
        }) {
            v.push(Bucket::Area(Area::Park));
        }
        if tags
            .iter()
            .any(|(k, v)| matches!((k.as_str(), v.as_str()), ("natural", "bare_rock")))
        {
            v.push(Bucket::Area(Area::NaturalRock));
        }
        if tags
            .iter()
            .any(|(k, v)| matches!((k.as_str(), v.as_str()), ("landuse", "railway")))
        {
            v.push(Bucket::Area(Area::Rail));
        }
        if tags
            .iter()
            .any(|(k, v)| matches!((k.as_str(), v.as_str()), ("natural", "tree")))
        {
            v.push(Bucket::Area(Area::Tree));
        }
        if tags.iter().any(|(k, v)| {
            matches!(
                (k.as_str(), v.as_str()),
                ("natural", "bay" | "coastline" | "water")
            )
        }) {
            v.push(Bucket::Area(Area::Water));
        }
        if tags
            .iter()
            .any(|(k, v)| matches!((k.as_str(), v.as_str()), ("boundary", "administrative")))
        {
            v.push(Bucket::Path(Path::Boundary));
        }
        if tags
            .iter()
            .any(|(k, v)| matches!((k.as_str(), v.as_str()), ("highway", "cycleway")))
        {
            v.push(Bucket::Path(Path::Cycleway));
        }
        if tags
            .iter()
            .any(|(k, v)| matches!((k.as_str(), v.as_str()), ("highway", "primary")))
        {
            v.push(Bucket::Path(Path::Highway1));
        }
        if tags
            .iter()
            .any(|(k, v)| matches!((k.as_str(), v.as_str()), ("highway", "secondary")))
        {
            v.push(Bucket::Path(Path::Highway2));
        }
        if tags
            .iter()
            .any(|(k, v)| matches!((k.as_str(), v.as_str()), ("highway", "tertiary")))
        {
            v.push(Bucket::Path(Path::Highway3));
        }
        if tags.iter().any(|(k, v)| {
            matches!(
                (k.as_str(), v.as_str()),
                ("highway", "primary_link" | "secondary_link" | "service")
            )
        }) {
            v.push(Bucket::Path(Path::Highway4));
        }
        if tags.iter().any(|(k, v)| {
            matches!(
                (k.as_str(), v.as_str()),
                (
                    "highway",
                    "footway" | "pedestrian" | "residential" | "steps"
                )
            )
        }) {
            v.push(Bucket::Path(Path::Pedestrian));
        }

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
                v.push(b);
            }
        }
        v
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
            vec![Bucket::Area(Area::Beach)]
        );
    }
}
