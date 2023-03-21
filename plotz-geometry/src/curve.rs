#![allow(unused)]
#![allow(missing_docs)]

use {
    crate::{bounded::Bounded, interpolate, point::Pt},
    float_ord::FloatOrd,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CurveArc {
    pub ctr: Pt,
    pub angle_1: FloatOrd<f64>,
    pub angle_2: FloatOrd<f64>,
    pub radius: FloatOrd<f64>,
}

impl Bounded for CurveArc {
    fn right_bound(&self) -> f64 {
        0.0
    }
    fn left_bound(&self) -> f64 {
        0.0
    }
    fn top_bound(&self) -> f64 {
        0.0
    }
    fn bottom_bound(&self) -> f64 {
        0.0
    }
}

impl CurveArc {
    pub fn new(ctr: Pt, angle_1: f64, angle_2: f64, radius: f64) -> CurveArc {
        CurveArc {
            ctr,
            angle_1: FloatOrd(angle_1),
            angle_2: FloatOrd(angle_2),
            radius: FloatOrd(radius),
        }
    }
}

impl std::ops::Add<Pt> for CurveArc {
    type Output = Self;
    fn add(self, rhs: Pt) -> Self::Output {
        CurveArc {
            ctr: self.ctr + rhs,
            ..self
        }
    }
}
