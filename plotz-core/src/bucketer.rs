use crate::bucket::{Area, Bucket, Path};
use lazy_static::lazy_static;
use string_interner::{symbol::SymbolU32, StringInterner};
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
    list: Vec<((SymbolU32, SymbolU32), Bucket)>,
}

impl DefaultBucketer {
    pub fn new(interner: &mut StringInterner) -> DefaultBucketer {
        let mut list = vec![];
        for ((tag0, tag1), bucket) in TAGS.iter() {
            list.push((
                (interner.get_or_intern(tag0), interner.get_or_intern(tag1)),
                *bucket,
            ));
        }
        DefaultBucketer { list }
    }
}

impl Bucketer for DefaultBucketer {
    type Tag = (SymbolU32, SymbolU32);
    type Bucket = Bucket;
    type Error = BucketerError;
    fn bucket(&self, tag: Self::Tag) -> Result<Self::Bucket, Self::Error> {
        self.list
            .iter()
            .find_map(|(tags, bucket)| if *tags == tag { Some(*bucket) } else { None })
            .ok_or(BucketerError::BucketerError)
    }
}

lazy_static! {
    pub static ref TAGS: Vec<((&'static str, &'static str), Bucket)> = vec![
        (("amenity", "school"), Bucket::Area(Area::Fun)),
        (("boundary", "administrative"), Bucket::Path(Path::Boundary)),
        (("building", "apartments"), Bucket::Area(Area::Building)),
        (("building", "garages"), Bucket::Area(Area::Building)),
        (("building", "yes"), Bucket::Area(Area::Building)),
        (("fitness_station", "box"), Bucket::Area(Area::Fun)),
        (("highway", "cycleway"), Bucket::Path(Path::Cycleway)),
        (("highway", "footway"), Bucket::Path(Path::Pedestrian)),
        (("highway", "pedestrian"), Bucket::Path(Path::Pedestrian)),
        (("highway", "primary"), Bucket::Path(Path::Highway1)),
        (("highway", "primary_link"), Bucket::Path(Path::Highway4)),
        (("highway", "residential"), Bucket::Path(Path::Pedestrian)),
        (("highway", "secondary"), Bucket::Path(Path::Highway2)),
        (("highway", "secondary_link"), Bucket::Path(Path::Highway4)),
        (("highway", "service"), Bucket::Path(Path::Highway4)),
        (("highway", "steps"), Bucket::Path(Path::Pedestrian)),
        (("highway", "tertiary"), Bucket::Path(Path::Highway3)),
        (("landuse", "brownfield"), Bucket::Area(Area::Park)),
        (("landuse", "cemetery"), Bucket::Area(Area::Park)),
        (("landuse", "commercial"), Bucket::Area(Area::Business)),
        (("landuse", "construction"), Bucket::Area(Area::Business)),
        (("landuse", "grass"), Bucket::Area(Area::Park)),
        (("landuse", "greenfield"), Bucket::Area(Area::Park)),
        (("landuse", "industrial"), Bucket::Area(Area::Business)),
        (("landuse", "meadow"), Bucket::Area(Area::Park)),
        (("landuse", "railway"), Bucket::Area(Area::Rail)),
        (("leisure", "garden"), Bucket::Area(Area::Park)),
        (("leisure", "park"), Bucket::Area(Area::Park)),
        (("leisure", "pitch"), Bucket::Area(Area::Fun)),
        (("leisure", "playground"), Bucket::Area(Area::Fun)),
        (("leisure", "swimming_pool"), Bucket::Area(Area::Fun)),
        (("natural", "bare_rock"), Bucket::Area(Area::NaturalRock)),
        (("natural", "bay"), Bucket::Area(Area::Water)),
        (("natural", "beach"), Bucket::Area(Area::Beach)),
        (("natural", "coastline"), Bucket::Area(Area::Water)),
        (("natural", "sand"), Bucket::Area(Area::Beach)),
        (("natural", "scrub"), Bucket::Area(Area::Park)),
        (("natural", "tree"), Bucket::Area(Area::Tree)),
        (("natural", "water"), Bucket::Area(Area::Water)),
        (("railway", "rail"), Bucket::Path(Path::Rail)),
    ];
}

#[cfg(test)]
mod test_super {
    use super::*;
    use plotz_geojson::INTERESTING_PROPERTIES;

    #[test]
    fn test_tags_marked_interesting() {
        for ((k, v), _) in TAGS.iter() {
            assert!(INTERESTING_PROPERTIES.contains(k), "{}", k);
            assert!(INTERESTING_PROPERTIES.contains(v), "{}", v);
        }
    }

    #[test]
    fn test_bucket() {
        let mut interner = StringInterner::new();
        let bucketer = DefaultBucketer::new(&mut interner);
        assert_eq!(
            bucketer.bucket((
                interner.get_or_intern("natural"),
                interner.get_or_intern("sand")
            )),
            Ok(Bucket::Area(Area::Beach))
        );
        assert_eq!(
            bucketer.bucket((
                interner.get_or_intern("natural"),
                interner.get_or_intern("")
            )),
            Err(BucketerError::BucketerError)
        );
    }
}
