use crate::{
    shapes::{point::Point, segment::Segment},
    utils::Percent,
};
use anyhow::Result;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum SegmentOp {
    PointAlongSegment(Point, Percent), // a point some percent along this segment.
    Subsegment(Segment),               // a subsegment of this segment.
    EntireSegment,                     // the whole segment.
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum MultilineOp {
    Point(usize, Point),                        // one of the points in the multiline.
    PointAlongSegmentOf(usize, Point, Percent), // a point some percent along a segment of this multiline.
    SubsegmentOf(usize, Segment),               // a subsegment of a segment of this multiline.
    EntireSubsegment(usize),                    // an entire subsegment of this multiline
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum PolygonOp {
    WithinArea,                            // within the area of the polygon.
    Point(usize, Point),                   // on a point of the polygon.
    PointAlongEdge(usize, Point, Percent), // a point some percent along an edge of this polygon.
    PartiallyWithinArea,                   // partially within the area of the polygon.
    SubsegmentOfEdge(usize, Segment),      // a subsegment of an edge of the polygon.
    AtSubpolygon,                          // a subpolygon of the polygon.
}

impl MultilineOp {
    // When would you need to convert a SegmentOpinion into a MultilineOpinion?
    // Well, what if you were traversing a multiline and found a collision along
    // one of its segments?
    //  - if that collision occurred along the segment at Percent::Zero, it would
    //    really be a MultilineOpinion::AtPoint { index, .. }.
    //  - and if that collision occurred along the segment at Percent::One, it
    //    would really be a MultilineOpinion::AtPoint{ index+1, ..}.
    // That's why.
    pub fn from_segment_opinion(index: usize, so: SegmentOp) -> MultilineOp {
        match so {
            SegmentOp::PointAlongSegment(at_point, percent_along) => match percent_along {
                Percent::Zero => MultilineOp::Point(index, at_point),
                Percent::One => MultilineOp::Point(index + 1, at_point),
                _ => MultilineOp::PointAlongSegmentOf(index, at_point, percent_along),
            },
            SegmentOp::Subsegment(segment) => MultilineOp::SubsegmentOf(index, segment),
            SegmentOp::EntireSegment => MultilineOp::EntireSubsegment(index),
        }
    }
}

pub fn rewrite_segment_opinions(
    segment_opinions: &mut Vec<SegmentOp>,
    original_sg: &Segment,
) -> Result<()> {
    segment_opinions.dedup();
    'edit: loop {
        let opinions_ = segment_opinions.clone();
        for (idx, op) in opinions_.iter().enumerate() {
            match op {
                SegmentOp::Subsegment(s) if (s == original_sg) || (s.flip() == *original_sg) => {
                    segment_opinions.remove(idx);
                    segment_opinions.push(SegmentOp::EntireSegment);
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
                (SegmentOp::PointAlongSegment(..), SegmentOp::EntireSegment) => {
                    segment_opinions.remove(idx1);
                    continue 'edit;
                }
                (SegmentOp::EntireSegment, SegmentOp::PointAlongSegment(..)) => {
                    segment_opinions.remove(idx2);
                    continue 'edit;
                }
                (SegmentOp::Subsegment(s1), SegmentOp::Subsegment(s2)) if s1.f == s2.i => {
                    segment_opinions.remove(idx2);
                    segment_opinions.remove(idx1);
                    segment_opinions.push(SegmentOp::Subsegment(Segment(s1.i, s2.f)));
                    continue 'edit;
                }
                (SegmentOp::Subsegment(s1), SegmentOp::Subsegment(s2)) if s2.f == s1.i => {
                    segment_opinions.remove(idx2);
                    segment_opinions.remove(idx1);
                    segment_opinions.push(SegmentOp::Subsegment(Segment(s2.i, s1.f)));
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

pub fn rewrite_multiline_opinions(multiline_opinions: &mut Vec<MultilineOp>) -> Result<()> {
    multiline_opinions.dedup();
    'edit: loop {
        let ops_ = multiline_opinions.clone();
        for ((idx1, op1), (idx2, op2)) in
            ops_.iter().enumerate().zip(ops_.iter().enumerate().skip(1))
        {
            match (op1, op2) {
                (MultilineOp::Point(pt_idx, _), MultilineOp::EntireSubsegment(sg_idx))
                    if (sg_idx + 1 == *pt_idx || pt_idx == sg_idx) =>
                {
                    //
                    multiline_opinions.remove(idx1);
                    continue 'edit;
                }
                (MultilineOp::EntireSubsegment(sg_idx), MultilineOp::Point(pt_idx, _))
                    if (pt_idx == sg_idx || *pt_idx == sg_idx + 1) =>
                {
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
