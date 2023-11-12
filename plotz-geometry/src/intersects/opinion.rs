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

impl MultilineOpinion {
    // When would you need to convert a SegmentOpinion into a MultilineOpinion?
    // Well, what if you were traversing a multiline and found a collision along
    // one of its segments?
    //  - if that collision occurred along the segment at Percent::Zero, it would
    //    really be a MultilineOpinion::AtPoint { index, .. }.
    //  - and if that collision occurred along the segment at Percent::One, it
    //    would really be a MultilineOpinion::AtPoint{ index+1, ..}.
    // That's why.
    pub fn from_segment_opinion(index: usize, so: SegmentOpinion) -> MultilineOpinion {
        match so {
            SegmentOpinion::AlongSegment {
                at_point,
                percent_along,
            } => match percent_along {
                Percent::Zero => MultilineOpinion::AtPoint { index, at_point },
                Percent::One => MultilineOpinion::AtPoint {
                    index: index + 1,
                    at_point,
                },
                _ => MultilineOpinion::AlongSharedSegment {
                    index,
                    at_point,
                    percent_along,
                },
            },
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum Opinion {
    Point,
    Segment(NonEmpty<SegmentOpinion>),
    Multiline(NonEmpty<MultilineOpinion>),
    Polygon(),
}
