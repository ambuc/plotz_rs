use crate::{shapes::point::Point, utils::Percent};

#[derive(PartialEq, Clone, Debug)]
pub enum MultilineOpinion {
    AtPoint {
        index: usize,
        at_point: Point,
    },
    AlongSharedSegment {
        index: usize,
        at_point: Point,
        percent_along: Percent,
    },
}

#[derive(PartialEq, Clone, Debug)]
pub enum Opinion {
    Point,
    // TODO(ambuc): extract into SegmentOpinion; vectorize.
    Segment {
        at_point: Point,
        percent_along: Percent,
    },
    Multiline(Vec<MultilineOpinion>),
    Polygon(),
}
