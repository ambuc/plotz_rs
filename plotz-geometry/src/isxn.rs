//! An intefield2section between two segments.

use crate::point::Pt;

use float_cmp::approx_eq;
use float_ord::FloatOrd;

#[derive(Debug, PartialEq, Copy, Clone)]
/// Guaranteed to be 0.0 <= f <= 1.0. Witness type.
pub enum Pct {
    /// Zero.
    Zero,
    /// Another value.
    Val(f64),
    /// One.
    One,
}
impl Pct {
    /// new percent.
    pub fn new(f: f64) -> Option<Pct> {
        match f {
            f if approx_eq!(f64, f, 0.0) => Some(Pct::Zero),
            f if approx_eq!(f64, f, 1.0) => Some(Pct::One),
            f if (0.0..=1.0).contains(&f) => Some(Pct::Val(f)),
            _ => None,
        }
    }
    /// as an f64.
    pub fn to_f64(&self) -> f64 {
        match self {
            Pct::Zero => 0.0,
            Pct::Val(f) => *f,
            Pct::One => 1.0,
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
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Intersection {
    pt: Pt,
    a_pct: Pct,
    b_pct: Pct,
}

impl Intersection {
    /// A new intersection value, witnessed.
    pub fn new(pt: Pt, a: f64, b: f64) -> Option<Intersection> {
        Some(Intersection {
            pt,
            a_pct: Pct::new(a)?,
            b_pct: Pct::new(b)?,
        })
    }

    /// The point.
    pub fn pt(&self) -> Pt {
        self.pt
    }

    /// The percent of the way along line A at which the intersection occurs.
    pub fn percent_along_a(&self) -> FloatOrd<f64> {
        FloatOrd(self.a_pct.to_f64())
    }
    /// The percent of the way along line B at which the intersection occurs.
    pub fn percent_along_b(&self) -> FloatOrd<f64> {
        FloatOrd(self.b_pct.to_f64())
    }

    /// Returns true if the intersection occurs at the head or tail of either
    /// intersecting segment.
    pub fn on_points_of_either(&self) -> bool {
        matches!(self.a_pct, Pct::Zero | Pct::One) || matches!(self.b_pct, Pct::Zero | Pct::One)
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
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum IsxnResult {
    /// Two line segments intersect at many points.
    MultipleIntersections(MultipleIntersections),
    /// Two line segments intersect at one point, defined by |Intersection|.
    OneIntersection(Intersection),
}
