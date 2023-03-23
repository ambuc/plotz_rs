use {
    crate::{char::Char, group::Group},
    derive_more::From,
    plotz_geometry::{
        bounded::Bounded,
        curve::CurveArc,
        point::Pt,
        polygon::Polygon,
        segment::Segment,
        traits::{Mutable, YieldPoints, YieldPointsMut},
    },
};

/// Either a polygon or a segment.
#[derive(Debug, PartialEq, Clone, From)]
pub enum DrawObjInner {
    /// A point.
    Point(Pt),
    /// A polygon.
    Polygon(Polygon),
    /// A segment.
    Segment(Segment),
    /// An arc.
    CurveArc(CurveArc),
    /// A character to be printed in SVG, at a point.
    Char(Char),
    /// A group of other drawobjects.
    Group(Group),
}

impl DrawObjInner {
    /// Returns true if the object is empty (i.e. zero points)
    pub fn is_empty(&self) -> bool {
        match self {
            DrawObjInner::Polygon(p) => p.pts.is_empty(),
            DrawObjInner::Group(dois) => dois.iter_dois().all(|doi| doi.is_empty()),
            DrawObjInner::Point(_)
            | DrawObjInner::Segment(_)
            | DrawObjInner::Char(_)
            | DrawObjInner::CurveArc(_) => false,
        }
    }

    pub fn inner_impl_bounded(&self) -> &dyn Bounded {
        match self {
            DrawObjInner::Char(ch) => ch,
            DrawObjInner::CurveArc(arc) => arc,
            DrawObjInner::Group(dos) => dos,
            DrawObjInner::Point(p) => p,
            DrawObjInner::Polygon(pg) => pg,
            DrawObjInner::Segment(s) => s,
        }
    }
    pub fn inner_impl_yield_points(&self) -> Option<&dyn YieldPoints> {
        match self {
            DrawObjInner::Point(p) => Some(p),
            DrawObjInner::Char(ch) => Some(ch),
            DrawObjInner::CurveArc(_) => None,
            DrawObjInner::Group(g) => Some(g),
            DrawObjInner::Polygon(pg) => Some(pg),
            DrawObjInner::Segment(sg) => Some(sg),
        }
    }
    pub fn inner_impl_yield_points_mut(&mut self) -> Option<&mut dyn YieldPointsMut> {
        match self {
            DrawObjInner::Point(p) => Some(p),
            DrawObjInner::Char(ch) => Some(ch),
            DrawObjInner::CurveArc(_) => None,
            DrawObjInner::Group(g) => Some(g),
            DrawObjInner::Polygon(pg) => Some(pg),
            DrawObjInner::Segment(sg) => Some(sg),
        }
    }
}

impl YieldPoints for DrawObjInner {
    fn yield_pts(&self) -> Option<Box<dyn Iterator<Item = &Pt> + '_>> {
        self.inner_impl_yield_points().and_then(|yp| yp.yield_pts())
    }
}

impl YieldPointsMut for DrawObjInner {
    fn yield_pts_mut(&mut self) -> Option<Box<dyn Iterator<Item = &mut Pt> + '_>> {
        self.inner_impl_yield_points_mut()
            .and_then(|ypm| ypm.yield_pts_mut())
    }
}

impl Mutable for DrawObjInner {}

impl Bounded for DrawObjInner {
    fn right_bound(&self) -> f64 {
        self.inner_impl_bounded().right_bound()
    }

    fn left_bound(&self) -> f64 {
        self.inner_impl_bounded().left_bound()
    }

    fn top_bound(&self) -> f64 {
        self.inner_impl_bounded().top_bound()
    }

    fn bottom_bound(&self) -> f64 {
        self.inner_impl_bounded().bottom_bound()
    }
}