#![allow(unused)]
#![allow(missing_docs)]

use {
    crate::{bounded::Bounded, interpolate, point::Pt},
    float_ord::FloatOrd,
    std::f64::consts::*,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CurveArc {
    pub ctr: Pt,
    pub angle_1: FloatOrd<f64>,
    pub angle_2: FloatOrd<f64>,
    pub radius: FloatOrd<f64>,
}

#[allow(clippy::upper_case_acronyms)]
enum Quadrant {
    I,
    II,
    III,
    IV,
}

fn quadrant(angle: f64) -> Option<Quadrant> {
    if angle < 0.0 {
        None
    } else if angle <= PI / 2.0 {
        Some(Quadrant::I)
    } else if angle <= PI {
        Some(Quadrant::II)
    } else if angle <= 3.0 * PI / 2.0 {
        Some(Quadrant::III)
    } else if angle <= 2.0 * PI {
        Some(Quadrant::IV)
    } else {
        None
    }
}

impl Bounded for CurveArc {
    fn right_bound(&self) -> f64 {
        self.ctr.x.0
            + self.radius.0
                * if (self.angle_1.0..self.angle_2.0).contains(&TAU) {
                    1.0
                } else {
                    std::cmp::max(
                        FloatOrd(self.angle_1.0.cos()),
                        FloatOrd(self.angle_2.0.cos()),
                    )
                    .0
                }
    }
    fn left_bound(&self) -> f64 {
        self.ctr.x.0
            + self.radius.0
                * if (self.angle_1.0..self.angle_2.0).contains(&PI) {
                    -1.0
                } else {
                    std::cmp::min(
                        FloatOrd(self.angle_1.0.cos()),
                        FloatOrd(self.angle_2.0.cos()),
                    )
                    .0
                }
    }
    fn top_bound(&self) -> f64 {
        self.ctr.y.0
            + self.radius.0
                * if (self.angle_1.0..self.angle_2.0).contains(&FRAC_PI_2) {
                    1.0
                } else {
                    std::cmp::max(
                        FloatOrd(self.angle_1.0.sin()),
                        FloatOrd(self.angle_2.0.sin()),
                    )
                    .0
                }
    }
    fn bottom_bound(&self) -> f64 {
        self.ctr.y.0
            + self.radius.0
                * if (self.angle_1.0..self.angle_2.0).contains(&(3.0 * FRAC_PI_2)) {
                    -1.0
                } else {
                    std::cmp::min(
                        FloatOrd(self.angle_1.0.sin()),
                        FloatOrd(self.angle_2.0.sin()),
                    )
                    .0
                }
    }
}

impl CurveArc {
    pub fn new(ctr: Pt, angle_1: f64, angle_2: f64, radius: f64) -> CurveArc {
        assert!(angle_1 <= angle_2);
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
