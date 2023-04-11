use crate::bucket::{Area, Bucket, Highway, Path, Subway};

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

pub struct DefaultBucketer2 {}

macro_rules! contains {
    // NB: stolen from matches! impl
    ($ls:expr, $(|)? $( $pattern:pat_param )|+ $( if $guard: expr )? $(,)?) => {
        $ls.iter()
            .any(|(a, b)|
            match (a.as_str(), b.as_str()) {
                $( $pattern )|+ $( if $guard )? => true,
                _ => false
            }
        )
    };
}

impl Bucketer2 for DefaultBucketer2 {
    type Tag = (String, String);
    type Bucket = Vec<Bucket>;
    fn bucket(&self, tags: &[Self::Tag]) -> Self::Bucket {
        let mut v = vec![];

        if contains!(tags, ("natural", "beach" | "sand")) {
            v.push(Bucket::Area(Area::Beach));
        }

        if contains!(
            tags,
            ("building", _)
                | ("power", _)
                | ("building:part", _)
                | ("man_made", _)
                | (
                    "amenity",
                    "college" | "school" | "fuel" | "shelter" | "hospital" | "university"
                )
        ) {
            v.push(Bucket::Area(Area::Building));
        }
        if contains!(
            tags,
            (
                "landuse",
                "allotments"
                    | "commercial"
                    | "construction"
                    | "industrial"
                    | "residential"
                    | "religious"
            )
        ) {
            v.push(Bucket::Area(Area::Land));
        }
        if contains!(
            tags,
            ("amenity", "bench")
                | ("fitness_station", "box")
                | (
                    "leisure",
                    "pitch"
                        | "playground"
                        | "stadium"
                        | "swimming_pool"
                        | "golf_course"
                        | "track"
                        | "bleachers"
                        | "schoolyard"
                        | "dog_park"
                        | "sports_centre"
                )
                | ("tourism", "picnic_site")
                | ("golf", "hole")
                | ("playground", _)
                | ("shop", _)
        ) {
            v.push(Bucket::Area(Area::Fun));
        }
        if contains!(
            tags,
            (
                "landuse",
                "forest"
                    | "brownfield"
                    | "cemetery"
                    | "grass"
                    | "greenfield"
                    | "meadow"
                    | "recreation_ground"
            ) | ("leisure", "garden" | "park" | "nature_reserve")
                | (
                    "natural",
                    "scree"
                        | "scrub"
                        | "tree_row"
                        | "cliff"
                        | "wood"
                        | "wetland"
                        | "heath"
                        | "shingle"
                )
                | ("traffic_calming", _)
                | ("indoor", "area")
        ) {
            v.push(Bucket::Area(Area::Park));
        }
        if contains!(tags, ("amenity", "parking" | "parking_space")) {
            v.push(Bucket::Area(Area::Parking));
        }
        if contains!(tags, ("natural", "bare_rock")) {
            v.push(Bucket::Area(Area::NaturalRock));
        }
        if contains!(tags, ("landuse", "railway")) {
            v.push(Bucket::Area(Area::Rail));
        }
        if contains!(tags, ("natural", "tree")) {
            v.push(Bucket::Area(Area::Tree));
        }
        if contains!(
            tags,
            ("natural", "bay" | "coastline" | "water") | ("waterway", _) | ("leisure", "marina")
        ) {
            v.push(Bucket::Area(Area::Water));
        }
        if contains!(
            tags,
            ("boundary", "place" | "administrative") | ("place", "neighborhood")
        ) {
            v.push(Bucket::Path(Path::Boundary));
        }
        if contains!(tags, ("highway", "cycleway") | ("route", "bicycle")) {
            v.push(Bucket::Path(Path::Cycleway));
        }

        if contains!(tags, ("highway", "elevator")) {
            v.push(Bucket::Path(Path::Highway(Highway::Elevator)));
        }
        if contains!(tags, ("highway", "motorway_link")) {
            v.push(Bucket::Path(Path::Highway(Highway::MotorwayLink)));
        }
        if contains!(tags, ("highway", "path")) {
            v.push(Bucket::Path(Path::Highway(Highway::Path)));
        }
        if contains!(tags, ("highway", "platform")) {
            v.push(Bucket::Path(Path::Highway(Highway::Platform)));
        }
        if contains!(tags, ("highway", "primary")) {
            v.push(Bucket::Path(Path::Highway(Highway::Primary)));
        }
        if contains!(tags, ("highway", "primary_link")) {
            v.push(Bucket::Path(Path::Highway(Highway::PrimaryLink)));
        }
        if contains!(tags, ("highway", "secondary")) {
            v.push(Bucket::Path(Path::Highway(Highway::Secondary)));
        }
        if contains!(tags, ("highway", "secondary_link")) {
            v.push(Bucket::Path(Path::Highway(Highway::SecondaryLink)));
        }
        if contains!(tags, ("highway", "service")) {
            v.push(Bucket::Path(Path::Highway(Highway::Service)));
        }
        if contains!(tags, ("highway", "tertiary")) {
            v.push(Bucket::Path(Path::Highway(Highway::Tertiary)));
        }
        if contains!(tags, ("highway", "tertiary_link")) {
            v.push(Bucket::Path(Path::Highway(Highway::TertiaryLink)));
        }
        if contains!(tags, ("highway", "track")) {
            v.push(Bucket::Path(Path::Highway(Highway::Track)));
        }
        if contains!(tags, ("highway", "unclassified")) {
            v.push(Bucket::Path(Path::Highway(Highway::Unclassified)));
        }
        if contains!(tags, ("road_marking", _)) {
            v.push(Bucket::Path(Path::Highway(Highway::RoadMarking)));
        }
        if contains!(tags, ("route", "road")) {
            v.push(Bucket::Path(Path::Highway(Highway::Road)));
        }

        if contains!(tags, ("bridge:support", _) | ("bridge", _)) {
            v.push(Bucket::Path(Path::Bridge));
        }
        if contains!(
            tags,
            (
                "highway",
                "footway" | "pedestrian" | "residential" | "steps"
            ) | ("route", "hiking")
        ) {
            v.push(Bucket::Path(Path::Pedestrian));
        }

        if contains!(tags, ("route", "subway")) {
            if let Some((_, name)) = tags.iter().find(|(k, _v)| k == "name") {
                if !name.contains("weekends")
                    && !name.contains("am rush")
                    && !name.contains("pm rush")
                    && !name.contains("late nights")
                {
                    if contains!(tags, ("ref", "A" | "C" | "E")) {
                        v.push(Bucket::Path(Path::Subway(Subway::_ACE)))
                    }
                    if contains!(tags, ("ref", "B" | "D" | "F" | "M")) {
                        v.push(Bucket::Path(Path::Subway(Subway::_BDFM)))
                    }
                    if contains!(tags, ("ref", "G")) {
                        v.push(Bucket::Path(Path::Subway(Subway::_G)))
                    }
                    if contains!(tags, ("ref", "L")) {
                        v.push(Bucket::Path(Path::Subway(Subway::_L)))
                    }
                    if contains!(tags, ("ref", "J" | "Z")) {
                        v.push(Bucket::Path(Path::Subway(Subway::_JZ)))
                    }
                    if contains!(tags, ("ref", "N" | "Q" | "R" | "W")) {
                        v.push(Bucket::Path(Path::Subway(Subway::_NQRW)))
                    }
                    if contains!(tags, ("ref", "1" | "2" | "3")) {
                        v.push(Bucket::Path(Path::Subway(Subway::_123)))
                    }
                    if contains!(tags, ("ref", "4" | "5" | "6")) {
                        v.push(Bucket::Path(Path::Subway(Subway::_456)))
                    }
                    if contains!(tags, ("ref", "7")) {
                        v.push(Bucket::Path(Path::Subway(Subway::_7)))
                    }
                    if contains!(tags, ("ref", "T")) {
                        v.push(Bucket::Path(Path::Subway(Subway::_T)))
                    }
                    if contains!(tags, ("ref", "S")) {
                        v.push(Bucket::Path(Path::Subway(Subway::_S)))
                    }
                } else {
                    v.push(Bucket::Path(Path::Subway(Subway::Other)));
                }
            } else {
                v.push(Bucket::Path(Path::Subway(Subway::Other)));
            }
        }

        if contains!(
            tags,
            ("railway", "subway") | ("abandoned:railway", _) | ("disused:railway", _)
        ) {
            v.push(Bucket::Path(Path::Subway(Subway::Other)));
        }

        if contains!(tags, ("route", "bus")) {
            v.push(Bucket::Path(Path::Bus));
        }

        if contains!(
            tags,
            ("railway", "rail")
                | ("route", "railway" | "train")
                | ("railway:historic", _)
                | ("train", "yes")
        ) {
            v.push(Bucket::Path(Path::Rail));
        }

        if contains!(tags, ("barrier", _)) {
            v.push(Bucket::Path(Path::Barrier));
        }

        if contains!(tags, ("cables", _) | ("power", "cable")) {
            v.push(Bucket::Path(Path::Cable));
        }

        if v.is_empty() {
            println!("{:?}", tags);
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
