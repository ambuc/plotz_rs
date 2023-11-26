use crate::{
    obj2::Obj2,
    overlaps::{opinion::segment_opinion::SegmentOp, totally_covers},
    shapes::{point::Point, polygon::Polygon, segment::Segment},
    utils::{
        Percent,
        Percent::{One, Zero},
    },
};
use anyhow::Result;
use nonempty::NonEmpty;
use std::usize;

use super::multiline_opinion::MultilineOp;

#[derive(PartialEq, Eq, Clone, Debug, PartialOrd, Ord)]
pub enum PolygonOp {
    Point(/*point index*/ usize, Point), // on a point of the polygon.
    EdgePoint(/*edge index*/ usize, Point, Percent), // a point some percent along an edge of this polygon.
    EdgeSubsegment(/*edge index*/ usize, Segment),   // a subsegment of an edge of the polygon.
    Edge(/*edge index*/ usize),                      // an entire edge of the polygon.
    AreaPoint(Point),                                // a point is within the area of the polygon.
    AreaSegment(Segment), // a segment is  within the area of the polygon.
    Subpolygon(Polygon),  // a subpolygon of the polygon.
    Entire,
}

impl PolygonOp {
    pub fn from_segment_opinion(index: usize, so: SegmentOp) -> PolygonOp {
        match so {
            SegmentOp::Point(at_point, percent_along) => match percent_along {
                Zero => PolygonOp::Point(index, at_point),
                One => PolygonOp::Point(index + 1, at_point),
                _ => PolygonOp::EdgePoint(index, at_point, percent_along),
            },
            SegmentOp::Subsegment(segment) => PolygonOp::EdgeSubsegment(index, segment),
            SegmentOp::Entire => PolygonOp::Edge(index),
        }
    }
    pub fn from_multiline_opinion(mo: MultilineOp) -> PolygonOp {
        match mo {
            MultilineOp::Point(idx, pt) => PolygonOp::Point(idx, pt),
            MultilineOp::SegmentPoint(idx, pt, pct) => PolygonOp::EdgePoint(idx, pt, pct),
            MultilineOp::Subsegment(idx, ss) => PolygonOp::EdgeSubsegment(idx, ss),
            MultilineOp::Segment(idx) => PolygonOp::Edge(idx),
            MultilineOp::Entire => PolygonOp::Entire,
            // of course, nothing maps to PolygonOp::{AreaPoint,AreaSegment,SubPolygon}.
        }
    }
    pub fn to_obj(&self, original: &Polygon) -> Obj2 {
        match self {
            PolygonOp::AreaPoint(p) | PolygonOp::Point(_, p) | PolygonOp::EdgePoint(_, p, _) => {
                Obj2::from(*p)
            }
            PolygonOp::AreaSegment(sg) | PolygonOp::EdgeSubsegment(_, sg) => Obj2::from(*sg),
            PolygonOp::Edge(idx) => Obj2::from(original.to_segments()[*idx]),
            PolygonOp::Subpolygon(pg) => Obj2::from(pg.clone()),
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
