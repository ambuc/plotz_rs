use crate::{
    shapes::{point::Point, segment::Segment},
    utils::Percent,
};
use anyhow::Result;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum SegmentOpinion {
    AtPointAlongSegment(/*at_point=*/ Point, /*percent_along=*/ Percent),

    // intersection is a subsegment of this segment.
    AlongSubsegment(Segment),

    // intersection point(s) comprise this entire segment.
    EntireSegment,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum MultilineOpinion {
    AtPoint(/*index=*/ usize, Point),

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
    EntireSubsegment(/*index=*/ usize),
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum PolygonOpinion {
    // polygon sees point:
    WithinArea,
    AtPoint {
        index: usize,
        at_point: Point,
    },
    AlongEdge {
        index: usize, // segment index
        at_point: Point,
        percent_along: Percent,
    },

    // polygon sees segment / multiline
    PartiallyWithinArea,

    AlongSubsegmentOfEdge {
        index: usize,
        subsegment: Segment,
    },

    // polygon sees polygon,
    AtSubpolygon,
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
            SegmentOpinion::AtPointAlongSegment(at_point, percent_along) => match percent_along {
                Percent::Zero => MultilineOpinion::AtPoint(index, at_point),
                Percent::One => MultilineOpinion::AtPoint(index + 1, at_point),
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
            SegmentOpinion::EntireSegment => MultilineOpinion::EntireSubsegment(index),
        }
    }
}

pub fn rewrite_segment_opinions(
    segment_opinions: &mut Vec<SegmentOpinion>,
    original_sg: &Segment,
) -> Result<()> {
    segment_opinions.dedup();
    'edit: loop {
        let opinions_ = segment_opinions.clone();
        for (idx, op) in opinions_.iter().enumerate() {
            match op {
                SegmentOpinion::AlongSubsegment(s)
                    if (s == original_sg) || (s.flip() == *original_sg) =>
                {
                    segment_opinions.remove(idx);
                    segment_opinions.push(SegmentOpinion::EntireSegment);
                    continue 'edit;
                }
                _ => {
                    // do nothing
                }
            }
        }
        for ((idx1, op1), (idx2, op2)) in opinions_
            .iter()
            .enumerate()
            .zip(opinions_.iter().enumerate().skip(1))
        {
            // these are... ugh... rewrite rules.
            match (op1, op2) {
                (SegmentOpinion::AtPointAlongSegment(_, _), SegmentOpinion::EntireSegment) => {
                    segment_opinions.remove(idx1);
                    continue 'edit;
                }
                (SegmentOpinion::EntireSegment, SegmentOpinion::AtPointAlongSegment(_, _)) => {
                    segment_opinions.remove(idx2);
                    continue 'edit;
                }
                (SegmentOpinion::AlongSubsegment(s1), SegmentOpinion::AlongSubsegment(s2))
                    if s1.f == s2.i =>
                {
                    segment_opinions.remove(idx2);
                    segment_opinions.remove(idx1);
                    segment_opinions.push(SegmentOpinion::AlongSubsegment(Segment(s1.i, s2.f)));
                    continue 'edit;
                }
                (SegmentOpinion::AlongSubsegment(s1), SegmentOpinion::AlongSubsegment(s2))
                    if s2.f == s1.i =>
                {
                    segment_opinions.remove(idx2);
                    segment_opinions.remove(idx1);
                    segment_opinions.push(SegmentOpinion::AlongSubsegment(Segment(s2.i, s1.f)));
                    continue 'edit;
                }
                _ => {
                    // do nothing
                }
            }
        }
        break;
    }

    Ok(())
}

pub fn rewrite_multiline_opinions(multiline_opinions: &mut Vec<MultilineOpinion>) -> Result<()> {
    multiline_opinions.dedup();
    'edit: loop {
        let ops_ = multiline_opinions.clone();
        for ((idx1, op1), (idx2, op2)) in
            ops_.iter().enumerate().zip(ops_.iter().enumerate().skip(1))
        {
            match (op1, op2) {
                (
                    MultilineOpinion::AtPoint(pt_idx, _),
                    MultilineOpinion::EntireSubsegment(sg_idx),
                ) if (sg_idx + 1 == *pt_idx || pt_idx == sg_idx) => {
                    //
                    multiline_opinions.remove(idx1);
                    continue 'edit;
                }
                (
                    MultilineOpinion::EntireSubsegment(sg_idx),
                    MultilineOpinion::AtPoint(pt_idx, _),
                ) if (pt_idx == sg_idx || *pt_idx == sg_idx + 1) => {
                    multiline_opinions.remove(idx2);
                    continue 'edit;
                }
                _ => {
                    // do nothing
                }
            }
        }
        break;
    }

    Ok(())
}
