use crate::{
    shapes::{point::Point, polygon::Polygon, segment::Segment},
    utils::Percent,
};
use anyhow::Result;
use nonempty::NonEmpty;
use std::usize;

#[derive(PartialEq, Eq, Copy, Clone, Debug, PartialOrd, Ord)]
pub enum SegmentOp {
    PointAlongSegment(Point, Percent), // a point some percent along this segment.
    Subsegment(Segment),               // a subsegment of this segment.
    EntireSegment,                     // the whole segment.
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum MultilineOp {
    Point(usize, Point),                        // one of the points in the multiline.
    PointAlongSegmentOf(usize, Point, Percent), // a point some percent along a segment of this multiline.
    SubsegmentOf(usize, Segment),               // a subsegment of a segment of this multiline.
    EntireSubsegment(usize),                    // an entire subsegment of this multiline
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

#[derive(PartialEq, Clone, Debug)]
pub enum PolygonOp {
    WithinArea,                            // within the area of the polygon.
    Point(usize, Point),                   // on a point of the polygon.
    PointAlongEdge(usize, Point, Percent), // a point some percent along an edge of this polygon.
    PartiallyWithinArea,                   // partially within the area of the polygon.
    SubsegmentOfEdge(usize, Segment),      // a subsegment of an edge of the polygon.
    EntireEdge(usize),                     // an entire edge of the polygon.
    AtSubpolygon(Polygon),                 // a subpolygon of the polygon.
}

impl PolygonOp {
    pub fn from_segment_opinion(index: usize, so: SegmentOp) -> PolygonOp {
        match so {
            SegmentOp::PointAlongSegment(at_point, percent_along) => match percent_along {
                Percent::Zero => PolygonOp::Point(index, at_point),
                Percent::One => PolygonOp::Point(index + 1, at_point),
                _ => PolygonOp::PointAlongEdge(index, at_point, percent_along),
            },
            SegmentOp::Subsegment(segment) => PolygonOp::SubsegmentOfEdge(index, segment),
            SegmentOp::EntireSegment => PolygonOp::EntireEdge(index),
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

#[derive(Clone, Default, Debug)]
pub struct PolygonOpSet {
    pg_ops: Vec<PolygonOp>,
}
impl PolygonOpSet {
    pub fn add(&mut self, pg_op: PolygonOp, original: &Polygon) {
        let original_pts_len = original.pts.len();
        match pg_op {
            PolygonOp::WithinArea => {
                self.pg_ops.push(pg_op);
            }
            PolygonOp::Point(n, _) if n == 0 => {
                if let Some(idx) = self
                    .pg_ops
                    .iter()
                    .position(|x| matches!(x, PolygonOp::Point(n, _) if *n == original_pts_len))
                {
                    self.pg_ops.remove(idx);
                }
                self.pg_ops.push(pg_op);
            }
            PolygonOp::Point(n, _) if n == original_pts_len => {
                if !self
                    .pg_ops
                    .iter()
                    .any(|x| matches!(x, PolygonOp::Point(0, _)))
                {
                    self.pg_ops.push(pg_op);
                }
            }
            PolygonOp::Point(..) => {
                if !self.pg_ops.contains(&pg_op) {
                    self.pg_ops.push(pg_op);
                }
            }
            PolygonOp::PointAlongEdge(_, _, _) => {
                self.pg_ops.push(pg_op);
            }
            PolygonOp::PartiallyWithinArea => {
                self.pg_ops.push(pg_op);
            }
            PolygonOp::SubsegmentOfEdge(_, _) => {
                self.pg_ops.push(pg_op);
            }
            PolygonOp::EntireEdge(_) => {
                self.pg_ops.push(pg_op);
            }
            PolygonOp::AtSubpolygon(_) => {
                self.pg_ops.push(pg_op);
            }
        }
    }
    pub fn to_nonempty(self) -> Option<NonEmpty<PolygonOp>> {
        NonEmpty::from_vec(self.pg_ops)
    }
}
