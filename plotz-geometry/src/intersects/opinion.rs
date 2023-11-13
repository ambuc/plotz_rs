use crate::{
    shapes::{point::Point, segment::Segment},
    utils::Percent,
};
use nonempty::NonEmpty;

#[derive(PartialEq, Clone, Debug)]
pub enum SegmentOpinion {
    AtPointAlongSegment {
        at_point: Point,
        percent_along: Percent,
    },

    // intersection is a subsegment of this segment.
    AlongSubsegment(Segment),

    // intersection point(s) comprise this entire segment.
    EntireSegment,
}

#[derive(PartialEq, Clone, Debug)]
pub enum MultilineOpinion {
    AtPoint {
        index: usize,
        at_point: Point,
    },

    AtPointAlongSharedSegment {
        index: usize,
        at_point: Point,
        percent_along: Percent,
    },

    // intersection point(s) comprise a subsegment of a segment of this
    // multiline.
    AlongSubsegmentOf {
        index: usize,
        subsegment: Segment,
    },

    // intersection point(s) comprise an entire subsegment of this multiline.
    EntireSubsegment {
        index: usize,
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
            SegmentOpinion::AtPointAlongSegment {
                at_point,
                percent_along,
            } => match percent_along {
                Percent::Zero => MultilineOpinion::AtPoint { index, at_point },
                Percent::One => MultilineOpinion::AtPoint {
                    index: index + 1,
                    at_point,
                },
                _ => MultilineOpinion::AtPointAlongSharedSegment {
                    index,
                    at_point,
                    percent_along,
                },
            },
            SegmentOpinion::AlongSubsegment(segment) => MultilineOpinion::AlongSubsegmentOf {
                index,
                subsegment: segment,
            },
            SegmentOpinion::EntireSegment => MultilineOpinion::EntireSubsegment { index },
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
