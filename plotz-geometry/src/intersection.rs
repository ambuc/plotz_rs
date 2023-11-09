//! An intefield2section between two segments.
#![allow(missing_docs)]

use crate::{
    shapes::point::Point,
    utils::{Percent, Which},
};
use float_ord::FloatOrd;
use std::fmt::Debug;

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
    pub pt: Point,
    a_pct: Percent,
    b_pct: Percent,
}

impl Intersection {
    /// A new intersection value, witnessed.
    pub fn new(pt: Point, a: f64, b: f64) -> Option<Intersection> {
        Some(Intersection {
            pt,
            a_pct: Percent::new(a)?,
            b_pct: Percent::new(b)?,
        })
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
        self.a_pct.is_at_boundary() || self.b_pct.is_at_boundary()
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

/// An enum representing whether an intersection occurred and where.
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum IntersectionResult {
    Ok(Intersection),
    ErrSegmentsAreTheSame,
    ErrSegmentsAreTheSameButReversed,
    ErrSegmentsAreColinear,
}
