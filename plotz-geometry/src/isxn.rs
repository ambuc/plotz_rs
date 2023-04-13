//! An intersection between two segments.

use {
    float_ord::FloatOrd,
    //
};

#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
/// Guaranteed to be 0.0 <= f <= 1.0. Witness type.
pub struct NormF {
    /// NOT PUB
    val: FloatOrd<f64>,
}
impl NormF {
    /// new normf.
    pub fn new(f: f64) -> Option<NormF> {
        if (0.0..=1.0).contains(&f) {
            Some(NormF { val: FloatOrd(f) })
        } else {
            None
        }
    }
}

/// A struct representing an intersection between two line segments.
/// Two values:
///    the first is the % of the way along line A at which the intersection
///    occurs. Guaranteed to be 0.0<=x<=1.0.
//       If this value is 0.0, the intersection is at self_i.
//       If this value is 1.0, the intersection is at self_f.
///    the second is the % of the way along line B at which the intersection
///    occurs. Guaranteed to be 0.0<=x<=1.0.
#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
pub struct Intersection(NormF, NormF);

impl Intersection {
    /// A new intersection value, witnessed.
    pub fn new(a: f64, b: f64) -> Option<Intersection> {
        let na = NormF::new(a)?;
        let nb = NormF::new(b)?;
        Some(Intersection(na, nb))
    }

    /// The percent of the way along line A at which the intersection occurs.
    pub fn percent_along_inner(&self) -> FloatOrd<f64> {
        self.0.val
    }
    /// The percent of the way along line B at which the intersection occurs.
    pub fn percent_along_frame(&self) -> FloatOrd<f64> {
        self.1.val
    }

    fn on_points_of_self(&self) -> bool {
        self.percent_along_inner().0 == 0.0 || self.percent_along_inner().0 == 1.0
    }
    fn on_points_of_other(&self) -> bool {
        self.percent_along_frame().0 == 0.0 || self.percent_along_frame().0 == 1.0
    }
    /// Returns true if the intersection occurs at the head or tail of either
    /// intersecting segment.
    pub fn on_points_of_either_polygon(&self) -> bool {
        self.on_points_of_self() || self.on_points_of_other()
    }
}

/// An enum representing two intersections.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum MultipleIntersections {
    /// Two line segments intersect because they are the same.
    LineSegmentsAreTheSame,
    /// Two line segments intersect because they are the same but reversed.
    LineSegmentsAreTheSameButReversed,
    /// Two line segments intersect at multiple points because they are colinear,
    /// but they are not the same.
    LineSegmentsAreColinear,
}

/// An enum representing whether an intersection occurred and where.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum IsxnResult {
    /// Two line segments intersect at many points.
    MultipleIntersections(MultipleIntersections),
    /// Two line segments intersect at one point, defined by |Intersection|.
    OneIntersection(Intersection),
}
