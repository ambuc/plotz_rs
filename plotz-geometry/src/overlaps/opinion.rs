use super::totally_covers;
use crate::{
    obj2::Obj2,
    shapes::{point::Point, polygon::Polygon, segment::Segment},
    utils::Percent,
};
use anyhow::{anyhow, Result};
use nonempty::NonEmpty;
use std::usize;

#[derive(PartialEq, Eq, Copy, Clone, Debug, PartialOrd, Ord)]
pub enum SegmentOp {
    PointAlongSegment(Point, Percent), // a point some percent along this segment.
    Subsegment(Segment),               // a subsegment of this segment.
    EntireSegment,                     // the whole segment.
}

impl SegmentOp {
    pub fn to_obj(&self, original_sg: &Segment) -> Obj2 {
        match self {
            SegmentOp::PointAlongSegment(p, _) => Obj2::from(*p),
            SegmentOp::Subsegment(ss) => Obj2::from(*ss),
            SegmentOp::EntireSegment => Obj2::from(*original_sg),
        }
    }

    // a SegmentOp totally "covers" another SegmentOp it is equal to or larger than it.
    pub fn totally_covers(&self, other: &SegmentOp, original_sg: &Segment) -> Result<bool> {
        totally_covers(&self.to_obj(original_sg), &other.to_obj(original_sg))
    }
}

#[derive(Clone, Debug)]
pub struct SegmentOpSet {
    sg_ops: Vec<SegmentOp>,
    original: Segment,
}

impl SegmentOpSet {
    pub fn new(original: &Segment) -> SegmentOpSet {
        SegmentOpSet {
            sg_ops: vec![],
            original: *original,
        }
    }

    pub fn add(&mut self, sg_op: SegmentOp) -> Result<()> {
        // If the incoming op is covered by an extant one, discard it.
        for extant_op in self.sg_ops.iter() {
            if extant_op.totally_covers(&sg_op, &self.original)? {
                return Ok(());
            }
        }

        // If the incoming op covers extant ones, discard them.
        let mut idxs_to_remove = vec![];
        for (idx, sg_op_extant) in self.sg_ops.iter().enumerate() {
            if sg_op.totally_covers(sg_op_extant, &self.original)? {
                idxs_to_remove.push(idx);
            }
        }
        idxs_to_remove.reverse();
        for idx_to_remove in idxs_to_remove {
            self.sg_ops.remove(idx_to_remove);
        }

        // need to deduplicate adjacent subsegments -- coverage doesn't take care of that.
        if let SegmentOp::Subsegment(s_new) = sg_op {
            if self.sg_ops.iter().any(
                |x| matches!(x, SegmentOp::Subsegment(ss) if *ss == s_new || ss.flip() == s_new),
            ) {
                // do not insert!
                return Ok(());
            }
            // if there is already a segment which lines up with this one, deduplicate them.
            if let Some(idx) = self.sg_ops.iter().position(|x| {
                matches!(
                    x,
                    SegmentOp::Subsegment(s_extant)
                    if s_new.slope() == s_extant.slope() && s_extant.f == s_new.i
                )
            }) {
                if let SegmentOp::Subsegment(s_extant) = self.sg_ops.remove(idx) {
                    self.sg_ops
                        .push(SegmentOp::Subsegment(Segment(s_extant.i, s_new.f)));
                    // do not insert the new value.
                    return Ok(());
                } else {
                    return Err(anyhow!("I thought you found a subsegment? what gives"));
                }
            }
            if let Some(idx) = self.sg_ops.iter().position(|x| {
                matches!(
                    x,
                    SegmentOp::Subsegment(s_extant)
                    if s_new.slope() == s_extant.slope() && s_new.f == s_extant.i
                )
            }) {
                if let SegmentOp::Subsegment(s_extant) = self.sg_ops.remove(idx) {
                    self.sg_ops
                        .push(SegmentOp::Subsegment(Segment(s_new.i, s_extant.f)));
                    // do not insert the new value.
                    return Ok(());
                } else {
                    return Err(anyhow!("I thought you found a subsegment? what gives"));
                }
            }
            // TODO(ambuc); there might be more tail-to-tail and tip-to-top things to cover here.

            // otherwise, OK to add.
            self.sg_ops.push(sg_op);
        } else {
            self.sg_ops.push(sg_op);
        }

        Ok(())
    }

    pub fn to_nonempty(mut self) -> Option<NonEmpty<SegmentOp>> {
        self.final_pass();

        let SegmentOpSet { mut sg_ops, .. } = self;

        sg_ops.sort();
        sg_ops.dedup();

        NonEmpty::from_vec(sg_ops)
    }

    fn final_pass(&mut self) {
        while let Some(idx) = self.sg_ops.iter().position(
            |x| matches!(x, SegmentOp::Subsegment(ss) if *ss == self.original || ss.flip() == self.original),
        ) {
            self.sg_ops.remove(idx);
            self.sg_ops.push(SegmentOp::EntireSegment);
        }
    }
}
#[derive(PartialEq, Eq, Copy, Clone, Debug, PartialOrd, Ord)]

pub enum MultilineOp {
    Point(usize, Point),                        // one of the points in the multiline.
    PointAlongSegmentOf(usize, Point, Percent), // a point some percent along a segment of this multiline.
    SubsegmentOf(usize, Segment),               // a subsegment of a segment of this multiline.
    EntireSubsegment(usize),                    // an entire subsegment of this multiline
    EntireMultiline,                            // the entire multiline // TODO(ambuc)
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

#[derive(PartialEq, Eq, Clone, Debug, PartialOrd, Ord)]
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
    multiline_opinions.sort();
    multiline_opinions.dedup();

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
        let PolygonOpSet { mut pg_ops } = self;
        pg_ops.sort();
        pg_ops.dedup();
        NonEmpty::from_vec(pg_ops)
    }
}
