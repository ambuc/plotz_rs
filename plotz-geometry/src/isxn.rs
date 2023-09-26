//! An intefield2section between two segments.

use crate::shapes::pt2::Pt2;
use std::fmt::Debug;

use float_cmp::approx_eq;
use float_ord::FloatOrd;

#[derive(Debug, PartialEq, Copy, Clone)]
/// Guaranteed to be 0.0 <= f <= 1.0. Witness type.
enum Pct {
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
    pub fn as_f64(&self) -> f64 {
        match self {
            Pct::Zero => 0.0,
            Pct::Val(f) => *f,
            Pct::One => 1.0,
        }
    }
}

#[derive(Debug, Copy, Clone)]
/// Which one?
pub enum Which {
    /// A?
    A,
    /// or B?
    B,
}
impl Which {
    /// Flip it.
    pub fn flip(&self) -> Which {
        match self {
            Which::A => Which::B,
            Which::B => Which::A,
        }
    }
}

/// two things, keyed by A / B
pub struct Pair<'a, T> {
    /// a
    pub a: &'a T,
    /// b
    pub b: &'a T,
}

impl<'a, T> Pair<'a, T> {
    /// get one.
    pub fn get(&'a self, which: Which) -> &'a T {
        match which {
            Which::A => self.a,
            Which::B => self.b,
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
#[derive(PartialEq, Copy, Clone)]
pub struct Intersection {
    /// pt
    pub pt: Pt2,
    /// a_pct
    a_pct: Pct,
    /// b_pct
    b_pct: Pct,
}

impl Intersection {
    /// A new intersection value, witnessed.
    pub fn new(pt: Pt2, a: f64, b: f64) -> Option<Intersection> {
        Some(Intersection {
            pt,
            a_pct: Pct::new(a)?,
            b_pct: Pct::new(b)?,
        })
    }

    /// The point.
    pub fn pt(&self) -> Pt2 {
        self.pt
    }

    /// The percent of the way along line A at which the intersection occurs.
    pub fn percent_along_a(&self) -> FloatOrd<f64> {
        FloatOrd(self.a_pct.as_f64())
    }
    /// The percent of the way along line B at which the intersection occurs.
    pub fn percent_along_b(&self) -> FloatOrd<f64> {
        FloatOrd(self.b_pct.as_f64())
    }

    /// The percent of the way along line |N| at which the intersection occurs.
    pub fn percent_along(&self, which: Which) -> FloatOrd<f64> {
        match which {
            Which::A => self.percent_along_a(),
            Which::B => self.percent_along_b(),
        }
    }

    /// Returns true if the intersection occurs at the head or tail of either
    /// intersecting segment.
    pub fn on_points_of_either(&self) -> bool {
        matches!(self.a_pct, Pct::Zero | Pct::One) || matches!(self.b_pct, Pct::Zero | Pct::One)
    }

    /// for whatever reason, some callers need to flip these.
    pub fn flip_pcts(self) -> Intersection {
        Intersection {
            pt: self.pt,
            a_pct: self.b_pct,
            b_pct: self.a_pct,
        }
    }
}

impl Debug for Intersection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Intersection { pt, a_pct, b_pct } = self;
        write!(
            f,
            "pt({:?}) {:.0}% along a, {:.0}% along b",
            pt,
            100.0 * a_pct.as_f64(),
            100.0 * b_pct.as_f64()
        )
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
#[derive(PartialEq, Copy, Clone)]
pub enum IsxnResult {
    /// Two line segments intersect at many points.
    MultipleIntersections(MultipleIntersections),
    /// Two line segments intersect at one point, defined by |Intersection|.
    OneIntersection(Intersection),
}

impl Debug for IsxnResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IsxnResult::MultipleIntersections(_) => write!(f, "multiple intersections."),
            IsxnResult::OneIntersection(isxn) => write!(f, "one: {:?}", isxn),
        }
    }
}
