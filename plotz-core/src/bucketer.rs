use crate::bucket::{Area, Bucket, Path};
use lazy_static::lazy_static;
use thiserror::Error;

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

pub struct DefaultBucketer {
    //
}
impl DefaultBucketer {
    pub fn new() -> DefaultBucketer {
        DefaultBucketer {}
    }
}

impl Bucketer for DefaultBucketer {
    type Tag = (&'static str, &'static str);
    type Bucket = Bucket;
    type Error = BucketerError;
    fn bucket(&self, tag: Self::Tag) -> Result<Self::Bucket, Self::Error> {
        TAGS.iter()
            .find_map(|(tags, bucket)| if *tags == tag { Some(*bucket) } else { None })
            .ok_or(BucketerError::BucketerError)
    }
}

lazy_static! {
    pub static ref TAGS: Vec<((&'static str, &'static str), Bucket)> = vec![
        // (("amenity", "fuel"), Bucket::None),
        // (("railway", "platform"), Bucket::None),
        (("boundary", "administrative"), Bucket::Path(Path::Boundary)),
        (("railway", "rail"), Bucket::Path(Path::Rail)),
        (("landuse", "railway"), Bucket::Area(Area::Rail)),
        (("landuse", "residential"), Bucket::Area(Area::Building)),
        // green
        (("leisure", "park"), Bucket::Area(Area::Park)),
        (("landuse", "grass"), Bucket::Area(Area::Park)),
        (("landuse", "greenfield"), Bucket::Area(Area::Park)),
        (("landuse", "meadow"), Bucket::Area(Area::Park)),
        (("natural", "scrub"), Bucket::Area(Area::Park)),
        (("leisure", "garden"), Bucket::Area(Area::Park)),
        (("landuse", "brownfield"), Bucket::Area(Area::Park)),
        (("landuse", "cemetery"), Bucket::Area(Area::Park)),
        (("landuse", "commercial"), Bucket::Area(Area::Business)),
        (("landuse", "industrial"), Bucket::Area(Area::Business)),
        (("landuse", "construction"), Bucket::Area(Area::Business)),
        // fun
        (("amenity", "school"), Bucket::Area(Area::Fun)),
        (("leisure", "playground"), Bucket::Area(Area::Fun)),
        (("leisure", "swimming_pool"), Bucket::Area(Area::Fun)),
        (("leisure", "pitch"), Bucket::Area(Area::Fun)),
        (("fitness_station", "box"), Bucket::Area(Area::Fun)),
        // highway
        (("highway", "primary"), Bucket::Path(Path::Highway1)),
        (("highway", "secondary"), Bucket::Path(Path::Highway2)),
        (("highway", "tertiary"), Bucket::Path(Path::Highway3)),
        (("highway", "service"), Bucket::Path(Path::Highway4)),
        (("highway", "footway"), Bucket::Path(Path::Pedestrian)),
        (("highway", "steps"), Bucket::Path(Path::Pedestrian)),
        (("highway", "residential"), Bucket::Path(Path::Pedestrian)),
        (("highway", "secondary_link"), Bucket::Path(Path::Highway4)),
        (("highway", "primary_link"), Bucket::Path(Path::Highway4)),
        (("highway", "cycleway"), Bucket::Path(Path::Cycleway)),
        (("highway", "pedestrian"), Bucket::Path(Path::Pedestrian)),
        // water
        (("natural", "coastline"), Bucket::Area(Area::Water)),
        (("natural", "bay"), Bucket::Area(Area::Water)),
        (("natural", "Area(Area::Water)"), Bucket::Area(Area::Water)),
        // tree
        (("natural", "tree"), Bucket::Area(Area::Tree)),
        // beach
        (("natural", "sand"), Bucket::Area(Area::Beach)),
        (("natural", "beach"), Bucket::Area(Area::Beach)),
        // rock
        (("natural", "bare_rock"), Bucket::Area(Area::NaturalRock)),
    ];
}

#[cfg(test)]
mod test_super {
    use super::*;

    #[test]
    fn test_bucket() {
        assert_eq!(
            DefaultBucketer::new().bucket(("natural", "sand")),
            Ok(Bucket::Area(Area::Beach))
        );
        assert_eq!(
            DefaultBucketer::new().bucket(("natural", "")),
            Err(BucketerError::BucketerError)
        );
    }
}
