use super::totally_covers;
use crate::{
    interpolate::interpolate_2d_checked,
    obj2::Obj2,
    shapes::{multiline::Multiline, point::Point, polygon::Polygon, segment::Segment},
    utils::{
        Percent,
        Percent::{One, Zero},
    },
};
use anyhow::Result;
use nonempty::NonEmpty;
use std::usize;

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
#[derive(PartialEq, Eq, Copy, Clone, Debug, PartialOrd, Ord)]

pub enum MultilineOp {
    Point(usize, Point),                      // one of the points in the multiline.
    PointAlongSegment(usize, Point, Percent), // a point some percent along a segment of this multiline.
    SubsegmentOf(usize, Segment),             // a subsegment of a segment of this multiline.
    EntireSubsegment(usize),                  // an entire subsegment of this multiline
    Entire,                                   // the entire multiline // TODO(ambuc)
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
                Percent::Zero => MultilineOp::Point(index, at_point),
                Percent::One => MultilineOp::Point(index + 1, at_point),
                _ => MultilineOp::PointAlongSegment(index, at_point, percent_along),
            },
            SegmentOp::Subsegment(segment) => MultilineOp::SubsegmentOf(index, segment),
            SegmentOp::Entire => MultilineOp::EntireSubsegment(index),
        }
    }
    pub fn to_obj(&self, original_ml: &Multiline) -> Obj2 {
        match self {
            MultilineOp::Point(_, p) | MultilineOp::PointAlongSegment(_, p, _) => Obj2::from(*p),
            MultilineOp::SubsegmentOf(_, sg) => Obj2::from(*sg),
            MultilineOp::EntireSubsegment(idx) => Obj2::from(original_ml.to_segments()[*idx]),
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
            MultilineOp::SubsegmentOf(s_idx, s_new) => {
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
                .position(|x| matches!(x, MultilineOp::EntireSubsegment(i) if *i == sg_idx))
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
                MultilineOp::SubsegmentOf(s_idx_extant, ss)
                if s_idx == *s_idx_extant && (*ss == s_new || ss.flip() == s_new)
            )
        }) {
            return Ok(());
        }

        // if this subsegment adjoins an existing one, (a) don't add the new
        // one, (b) remove the existing one, and (c) add the joined segment
        // instead.
        if let Some((idx, resultant)) = self.ml_ops.iter().find_map(|extant| match extant {
            MultilineOp::SubsegmentOf(idx_old, s_old) if *idx_old == s_idx => {
                s_old.try_add(&s_new).map(|resultant| (idx_old, resultant))
            }
            _ => None,
        }) {
            self.ml_ops.remove(*idx);
            if resultant == self.original.to_segments()[s_idx] {
                self.add(MultilineOp::EntireSubsegment(s_idx))?;
            } else {
                self.add(MultilineOp::SubsegmentOf(s_idx, resultant))?;
            }
            return Ok(());
        }

        self.ml_ops.push(ml_op);

        Ok(())
    }

    /*
    // returns a sorted vector of sorted, deduplicated vectors of cut points along each subsegment of this multiline.
    pub fn to_cuts(&self) -> Result<Vec<Vec<(Point, Percent)>>> {
        let mut cuts: Vec<Vec<(Point, Percent)>> = vec![];
        for (_sg_idx, sg) in self.original.to_segments().iter().enumerate() {
            let mut cuts_sg: Vec<(Point, Percent)> = vec![];
            cuts_sg.push((sg.i, Zero));
            cuts_sg.push((sg.i, One));
            cuts.push(cuts_sg);
        }

        for op in &self.ml_ops {
            match op {
                MultilineOp::PointAlongSegmentOf(sg_idx, pt, pct) => {
                    cuts[*sg_idx].push((*pt, *pct));
                }
                MultilineOp::SubsegmentOf(sg_idx, ss) => {
                    let sg = self.original.to_segments()[*sg_idx];
                    cuts[*sg_idx].push((ss.i, interpolate_2d_checked(sg.i, sg.f, ss.i)?));
                    cuts[*sg_idx].push((ss.f, interpolate_2d_checked(sg.i, sg.f, ss.f)?));
                }
                MultilineOp::Point(..)
                | MultilineOp::EntireSubsegment(..)
                | MultilineOp::EntireMultiline => {
                    // do nothing -- a junction along a multiline is already, a
                    // cut point.
                }
            }
        }

        for cuts_sg in cuts.iter_mut() {
            cuts_sg.sort_by_key(|(_, pct)| *pct);
            cuts_sg.dedup();
        }

        Ok(cuts)
    }
    */
}

#[derive(PartialEq, Eq, Clone, Debug, PartialOrd, Ord)]
pub enum PolygonOp {
    PointWithinArea(Point), // a point is within the area of the polygon.
    OnPoint(usize, Point),  // on a point of the polygon.
    PointAlongEdge(usize, Point, Percent), // a point some percent along an edge of this polygon.

    // NB: this doesn't represent 'totally' within, i.e. the points of the
    // segment might be on the points of edges of the polygon.
    SegmentWithinArea(Segment), // a segment is  within the area of the polygon.

    SubsegmentOfEdge(usize, Segment), // a subsegment of an edge of the polygon.
    EntireEdge(usize),                // an entire edge of the polygon.
    AtSubpolygon(Polygon),            // a subpolygon of the polygon.
    Entire,
}

impl PolygonOp {
    pub fn from_segment_opinion(index: usize, so: SegmentOp) -> PolygonOp {
        match so {
            SegmentOp::Point(at_point, percent_along) => match percent_along {
                Percent::Zero => PolygonOp::OnPoint(index, at_point),
                Percent::One => PolygonOp::OnPoint(index + 1, at_point),
                _ => PolygonOp::PointAlongEdge(index, at_point, percent_along),
            },
            SegmentOp::Subsegment(segment) => PolygonOp::SubsegmentOfEdge(index, segment),
            SegmentOp::Entire => PolygonOp::EntireEdge(index),
        }
    }
    pub fn to_obj(&self, original: &Polygon) -> Obj2 {
        match self {
            PolygonOp::PointWithinArea(p)
            | PolygonOp::OnPoint(_, p)
            | PolygonOp::PointAlongEdge(_, p, _) => Obj2::from(*p),
            PolygonOp::SegmentWithinArea(sg) | PolygonOp::SubsegmentOfEdge(_, sg) => {
                Obj2::from(*sg)
            }
            PolygonOp::EntireEdge(idx) => Obj2::from(original.to_segments()[*idx]),
            PolygonOp::AtSubpolygon(pg) => Obj2::from(pg.clone()),
            PolygonOp::Entire => Obj2::from(original.clone()),
        }
    }
    pub fn totally_covers(&self, other: &Self, original_pg: &Polygon) -> Result<bool> {
        totally_covers(&self.to_obj(original_pg), &other.to_obj(original_pg))
    }
}

#[derive(Clone, Debug)]
pub struct PolygonOpSet {
    pg_ops: Vec<PolygonOp>,
    original: Polygon,
}
impl PolygonOpSet {
    pub fn new(original: &Polygon) -> PolygonOpSet {
        PolygonOpSet {
            pg_ops: vec![],
            original: original.clone(),
        }
    }
    pub fn add(&mut self, pg_op: PolygonOp) -> Result<()> {
        // If the incoming op is covered by an extant one, discard it.
        if self.any_ops_cover(&pg_op)? {
            return Ok(());
        }

        self.pg_ops
            .retain(|extant| !pg_op.totally_covers(extant, &self.original).unwrap());

        // TODO(ambuc):  inline deduplication
        self.pg_ops.push(pg_op);

        Ok(())
    }
    pub fn to_nonempty(self) -> Option<NonEmpty<PolygonOp>> {
        // TODO(ambuc):  final pass
        let PolygonOpSet { mut pg_ops, .. } = self;
        pg_ops.sort();
        pg_ops.dedup();
        NonEmpty::from_vec(pg_ops)
    }

    fn any_ops_cover(&self, incoming: &PolygonOp) -> Result<bool> {
        for extant in &self.pg_ops {
            if extant.totally_covers(incoming, &self.original)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
