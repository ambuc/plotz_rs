use crate::bucket::{Area, Bucket, Path, Subway};

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
            ("building", "apartments" | "garages" | "yes")
                | ("landuse", "commercial" | "construction" | "industrial")
        ) {
            v.push(Bucket::Area(Area::Building));
        }
        if contains!(
            tags,
            ("amenity", "school")
                | ("fitness_station", "box")
                | ("leisure", "pitch" | "playground" | "swimming_pool")
        ) {
            v.push(Bucket::Area(Area::Fun));
        }
        if contains!(
            tags,
            (
                "landuse",
                "brownfield" | "cemetery" | "grass" | "greenfield" | "meadow"
            ) | ("leisure", "garden" | "park")
                | ("natural", "scrub")
        ) {
            v.push(Bucket::Area(Area::Park));
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
        if contains!(tags, ("natural", "bay" | "coastline" | "water")) {
            v.push(Bucket::Area(Area::Water));
        }
        if contains!(tags, ("boundary", "administrative")) {
            v.push(Bucket::Path(Path::Boundary));
        }
        if contains!(tags, ("highway", "cycleway")) {
            v.push(Bucket::Path(Path::Cycleway));
        }
        if contains!(tags, ("highway", "primary")) {
            v.push(Bucket::Path(Path::Highway1));
        }
        if contains!(tags, ("highway", "secondary")) {
            v.push(Bucket::Path(Path::Highway2));
        }
        if contains!(tags, ("highway", "tertiary")) {
            v.push(Bucket::Path(Path::Highway3));
        }
        if contains!(
            tags,
            ("highway", "primary_link" | "secondary_link" | "service")
        ) {
            v.push(Bucket::Path(Path::Highway4));
        }
        if contains!(
            tags,
            (
                "highway",
                "footway" | "pedestrian" | "residential" | "steps"
            )
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
                    if contains!(tags, ("railway", "rail")) {
                        v.push(Bucket::Path(Path::Rail));
                    }
                }
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
