use crate::{shapes::point::Point, utils::Percent};
use nonempty::NonEmpty;

#[derive(PartialEq, Clone, Debug)]
pub enum SegmentOpinion {
    AlongSegment {
        at_point: Point,
        percent_along: Percent,
    },
}

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
    Segment(NonEmpty<SegmentOpinion>),
    Multiline(NonEmpty<MultilineOpinion>),
    Polygon(),
}
