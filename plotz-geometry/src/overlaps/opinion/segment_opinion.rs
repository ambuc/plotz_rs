use crate::{
    interpolate::interpolate_2d_checked,
    obj2::Obj2,
    overlaps::totally_covers,
    shapes::{point::Point, segment::Segment},
    utils::{
        Percent,
        Percent::{One, Zero},
    },
};
use anyhow::Result;
use nonempty::NonEmpty;

#[derive(PartialEq, Eq, Copy, Clone, Debug, PartialOrd, Ord)]
pub enum SegmentOp {
    Point(Point, Percent), // a point some percent along this segment.
    Subsegment(Segment),   // a subsegment of this segment.
    Entire,                // the whole segment.
}

impl SegmentOp {
    pub fn to_obj(&self, original_sg: &Segment) -> Obj2 {
        match self {
            SegmentOp::Point(p, _) => Obj2::from(*p),
            SegmentOp::Subsegment(ss) => Obj2::from(*ss),
            SegmentOp::Entire => Obj2::from(*original_sg),
        }
    }

    // a SegmentOp totally "covers" another SegmentOp it is equal to or larger than it.
    pub fn totally_covers(&self, other: &SegmentOp, original_sg: &Segment) -> Result<bool> {
        totally_covers(&self.to_obj(original_sg), &other.to_obj(original_sg))
    }
}

#[derive(Clone, Debug)]
pub struct SegmentOpSet {
    pub sg_ops: Vec<SegmentOp>,
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
        // If the incoming op is covered by an extant one, discard it (by returning early).
        if self.any_ops_cover(sg_op)? {
            return Ok(());
        }
        // If the incoming op covers extant ones, discard them.
        self.sg_ops
            .retain(|extant| !sg_op.totally_covers(extant, &self.original).unwrap());

        // need to deduplicate adjacent subsegments -- coverage doesn't take care of that.
        if let SegmentOp::Subsegment(s_new) = sg_op {
            if self.sg_ops.iter().any(
                |x| matches!(x, SegmentOp::Subsegment(ss) if *ss == s_new || ss.flip() == s_new),
            ) {
                // do not insert!
                return Ok(());
            }

            // if there is a segment which adds to this segment to make a larger segment,
            // remove the extant one and add their sum instead.
            if let Some((idx, resultant)) = self
                .sg_ops
                .iter()
                .enumerate()
                .filter_map(|(idx, extant)| match extant {
                    SegmentOp::Subsegment(extant_sg) => {
                        s_new.try_add(extant_sg).map(|resultant| (idx, resultant))
                    }
                    _ => None,
                })
                .next()
            {
                self.sg_ops.remove(idx);
                self.sg_ops.push(SegmentOp::Subsegment(resultant));
                return Ok(());
            }

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

    // Returns a sorted, deduplicated vector of cut points along this segment.
    pub fn to_cuts(&self) -> Result<Vec<(Point, Percent)>> {
        let mut cuts: Vec<(Point, Percent)> = vec![];
        // add 0 and 1.
        cuts.push((self.original.i, Zero));
        cuts.push((self.original.f, One));
        // Add each existing cut point.
        for op in &self.sg_ops {
            match op {
                SegmentOp::Point(pt, pct) => {
                    cuts.push((*pt, *pct));
                }
                SegmentOp::Subsegment(ss) => {
                    cuts.push((
                        ss.i,
                        interpolate_2d_checked(self.original.i, self.original.f, ss.i)?,
                    ));
                    cuts.push((
                        ss.f,
                        interpolate_2d_checked(self.original.i, self.original.f, ss.f)?,
                    ));
                }
                SegmentOp::Entire => {}
            }
        }
        cuts.sort_by_key(|(_, pct)| *pct);
        cuts.dedup();
        Ok(cuts)
    }

    // Returns true if any extant sg_ops totally cover the incoming sg_op.
    fn any_ops_cover(&self, incoming: SegmentOp) -> Result<bool> {
        for extant in &self.sg_ops {
            if extant.totally_covers(&incoming, &self.original)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn final_pass(&mut self) {
        while let Some(idx) = self.sg_ops.iter().position(
            |x| matches!(x, SegmentOp::Subsegment(ss) if *ss == self.original || ss.flip() == self.original),
        ) {
            self.sg_ops.remove(idx);
            self.sg_ops.push(SegmentOp::Entire);
        }
    }
}
