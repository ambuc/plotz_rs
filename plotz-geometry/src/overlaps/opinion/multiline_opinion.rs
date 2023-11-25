use crate::{
    obj2::Obj2,
    overlaps::{opinion::segment_opinion::SegmentOp, totally_covers},
    shapes::{multiline::Multiline, point::Point, segment::Segment},
    utils::{
        Percent,
        Percent::{One, Zero},
    },
};
use anyhow::Result;
use nonempty::NonEmpty;
use std::usize;

#[derive(PartialEq, Eq, Copy, Clone, Debug, PartialOrd, Ord)]

pub enum MultilineOp {
    Point(/*point index*/ usize, Point), // one of the points in the multiline.
    SegmentPoint(/*segment index*/ usize, Point, Percent), // a point some percent along a segment of this multiline.
    Subsegment(/*segment index*/ usize, Segment), // a subsegment of a segment of this multiline.
    Segment(/*segment index*/ usize),             // an entire segment of this multiline
    Entire,                                       // the entire multiline // TODO(ambuc)
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
            SegmentOp::Point(at_point, percent_along) => match percent_along {
                Zero => MultilineOp::Point(index, at_point),
                One => MultilineOp::Point(index + 1, at_point),
                _ => MultilineOp::SegmentPoint(index, at_point, percent_along),
            },
            SegmentOp::Subsegment(segment) => MultilineOp::Subsegment(index, segment),
            SegmentOp::Entire => MultilineOp::Segment(index),
        }
    }
    pub fn to_obj(&self, original_ml: &Multiline) -> Obj2 {
        match self {
            MultilineOp::Point(_, p) | MultilineOp::SegmentPoint(_, p, _) => Obj2::from(*p),
            MultilineOp::Subsegment(_, sg) => Obj2::from(*sg),
            MultilineOp::Segment(idx) => Obj2::from(original_ml.to_segments()[*idx]),
            MultilineOp::Entire => Obj2::from(original_ml.clone()),
        }
    }
    pub fn totally_covers(&self, other: &Self, original_ml: &Multiline) -> Result<bool> {
        totally_covers(&self.to_obj(original_ml), &other.to_obj(original_ml))
    }
}

pub struct MultilineOpSet {
    ml_ops: Vec<MultilineOp>,
    original: Multiline,
}

impl MultilineOpSet {
    pub fn new(original: &Multiline) -> MultilineOpSet {
        MultilineOpSet {
            ml_ops: vec![],
            original: original.clone(),
        }
    }

    pub fn add(&mut self, ml_op: MultilineOp) -> Result<()> {
        // If the incoming op is covered by an extant one, discard it.
        if self.any_ops_cover(ml_op)? {
            return Ok(());
        }

        // If the incoming op covers extant ones, discard them.
        self.ml_ops
            .retain(|extant| !ml_op.totally_covers(extant, &self.original).unwrap());

        match ml_op {
            MultilineOp::Subsegment(s_idx, s_new) => {
                // need to deduplicate adjacent subsegments -- coverage doesn't take care of that.
                self.add_subsegment_of(ml_op, s_idx, s_new)?;
            }
            _ => {
                self.ml_ops.push(ml_op);
            }
        }

        Ok(())
    }

    pub fn to_nonempty(mut self) -> Option<NonEmpty<MultilineOp>> {
        self.final_pass();

        let MultilineOpSet { mut ml_ops, .. } = self;

        ml_ops.sort();
        ml_ops.dedup();

        NonEmpty::from_vec(ml_ops)
    }

    fn any_ops_cover(&self, incoming: MultilineOp) -> Result<bool> {
        for extant in &self.ml_ops {
            if extant.totally_covers(&incoming, &self.original)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn final_pass(&mut self) {
        // check for the case where we have all the necessary ml segments to comprise the entire ml.
        let mut idxs_to_remove = vec![];
        for sg_idx in 0..self.original.to_segments().len() {
            if let Some(idx) = self
                .ml_ops
                .iter()
                .position(|x| matches!(x, MultilineOp::Segment(i) if *i == sg_idx))
            {
                idxs_to_remove.push(idx);
            }
        }
        if idxs_to_remove.len() == self.original.to_segments().len() {
            idxs_to_remove.sort();
            idxs_to_remove.reverse();
            for idx in idxs_to_remove {
                self.ml_ops.remove(idx);
            }
            self.ml_ops.push(MultilineOp::Entire);
        }
    }

    fn add_subsegment_of(
        &mut self,
        ml_op: MultilineOp,
        s_idx: usize,
        s_new: Segment,
    ) -> Result<()> {
        // if we already have this subsegment or its flip, don't insert it and just return.
        if self.ml_ops.iter().any(|x| {
            matches!(
                x,
                MultilineOp::Subsegment(s_idx_extant, ss)
                if s_idx == *s_idx_extant && (*ss == s_new || ss.flip() == s_new)
            )
        }) {
            return Ok(());
        }

        // if this subsegment adjoins an existing one, (a) don't add the new
        // one, (b) remove the existing one, and (c) add the joined segment
        // instead.
        if let Some((idx, resultant)) = self.ml_ops.iter().find_map(|extant| match extant {
            MultilineOp::Subsegment(idx_old, s_old) if *idx_old == s_idx => {
                s_old.try_add(&s_new).map(|resultant| (idx_old, resultant))
            }
            _ => None,
        }) {
            self.ml_ops.remove(*idx);
            if resultant == self.original.to_segments()[s_idx] {
                self.add(MultilineOp::Segment(s_idx))?;
            } else {
                self.add(MultilineOp::Subsegment(s_idx, resultant))?;
            }
            return Ok(());
        }

        self.ml_ops.push(ml_op);

        Ok(())
    }
}
