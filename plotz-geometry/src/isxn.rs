//! An intersection between two segments.

use crate::point::Pt;

use float_cmp::approx_eq;
use float_ord::FloatOrd;

#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
/// Guaranteed to be 0.0 <= f <= 1.0. Witness type.
pub enum NormF {
    /// Zero.
    Zero,
    /// Another value.
    Val(FloatOrd<f64>),
    /// One.
    One,
}
impl NormF {
    /// new normf.
    pub fn new(f: f64) -> Option<NormF> {
        match f {
            f if approx_eq!(f64, f, 0.0) => Some(NormF::Zero),
            f if approx_eq!(f64, f, 1.0) => Some(NormF::One),
            f if (0.0..=1.0).contains(&f) => Some(NormF::Val(FloatOrd(f))),
            _ => None,
        }
    }
    /// as an f64.
    pub fn to_f64(&self) -> f64 {
        match self {
            NormF::Zero => 0.0,
            NormF::Val(f) => f.0,
            NormF::One => 1.0,
        }
    }
    /// as a FloatOrd<f64>.
    pub fn to_float_ord(&self) -> FloatOrd<f64> {
        FloatOrd(self.to_f64())
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
pub struct Intersection(Pt, NormF, NormF);

impl Intersection {
    /// A new intersection value, witnessed.
    pub fn new(pt: Pt, a: f64, b: f64) -> Option<Intersection> {
        let na = NormF::new(a)?;
        let nb = NormF::new(b)?;
        Some(Intersection(pt, na, nb))
    }

    /// The point.
    pub fn pt(&self) -> Pt {
        self.0
    }

    /// The percent of the way along line A at which the intersection occurs.
    pub fn percent_along_a(&self) -> FloatOrd<f64> {
        self.1.to_float_ord()
    }
    /// The percent of the way along line B at which the intersection occurs.
    pub fn percent_along_b(&self) -> FloatOrd<f64> {
        self.2.to_float_ord()
    }

    fn on_points_of_a(&self) -> bool {
        matches!(self.1, NormF::Zero | NormF::One)
    }
    fn on_points_of_b(&self) -> bool {
        matches!(self.2, NormF::Zero | NormF::One)
    }
    /// Returns true if the intersection occurs at the head or tail of either
    /// intersecting segment.
    pub fn on_points_of_either(&self) -> bool {
        self.on_points_of_a() || self.on_points_of_b()
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
