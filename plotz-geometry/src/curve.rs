#![allow(unused)]
#![allow(missing_docs)]

use {
    crate::{bounded::Bounded, interpolate, point::Pt},
    float_ord::FloatOrd,
    std::f64::consts::PI,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CurveArc {
    pub ctr: Pt,
    pub angle_1: FloatOrd<f64>,
    pub angle_2: FloatOrd<f64>,
    pub radius: FloatOrd<f64>,
}

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
        0.0
    }
    fn left_bound(&self) -> f64 {
        0.0
    }
    fn top_bound(&self) -> f64 {
        match (quadrant(self.angle_1.0), quadrant(self.angle_2.0)) {
            (None, _) => panic!("angle_1 not in a quadrant"),
            (_, None) => panic!("angle_2 not in a quadrant"),
            (Some(Quadrant::I), Some(Quadrant::I)) => {
                self.ctr.y.0 + self.radius.0 * self.angle_2.0.sin()
            }
            (Some(Quadrant::I), _) => self.ctr.y.0 + self.radius.0,
            (Some(Quadrant::II), _) => self.ctr.y.0 + self.radius.0 * self.angle_1.0.sin(),
            (Some(Quadrant::III), Some(Quadrant::III)) => {
                self.ctr.y.0 + self.radius.0 * self.angle_1.0.sin()
            }
            (Some(Quadrant::III), Some(Quadrant::IV)) => {
                std::cmp::max(
                    FloatOrd(self.ctr.y.0 + self.radius.0 * self.angle_1.0.sin()),
                    FloatOrd(self.ctr.y.0 + self.radius.0 * self.angle_2.0.sin()),
                )
                .0
            }
            (Some(Quadrant::III), _) => panic!("impossible, a1<=a2"),
            (Some(Quadrant::IV), Some(Quadrant::IV)) => {
                self.ctr.y.0 + self.radius.0 * self.angle_2.0.sin()
            }
            (Some(Quadrant::IV), _) => panic!("impossible, a1<=a2"),
        }
    }
    fn bottom_bound(&self) -> f64 {
        0.0
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

#[cfg(test)]
mod test {
    use std::f64::consts::*;

    use float_eq::assert_float_eq;

    use crate::{bounded::Bounded, curve::CurveArc, point::Pt};

    #[test]
    fn test_bounded() {
        // full circle

        let cx = 1.0;
        let cy = 1.0;
        let ctr = Pt(1.0, 1.0);
        let rad = 1.1;
        for (a1, a2, expected) in [
            (0.0, FRAC_PI_4 * 0.0, cx),                         // 0, 0
            (0.0, FRAC_PI_4 * 1.0, cx + rad * FRAC_PI_4.sin()), // 0, Q1
            (0.0, FRAC_PI_4 * 2.0, cx + rad),                   // 0, PI/2
            (0.0, FRAC_PI_4 * 3.0, cx + rad),                   // 0, Q2
            (0.0, FRAC_PI_4 * 4.0, cx + rad),                   // 0, PI
            (0.0, FRAC_PI_4 * 5.0, cx + rad),                   // 0, Q3
            (0.0, FRAC_PI_4 * 6.0, cx + rad),                   // 0, 3PI/2
            (0.0, FRAC_PI_4 * 7.0, cx + rad),                   // 0, Q4
            (0.0, FRAC_PI_4 * 8.0, cx + rad),                   // 0, 2PI
            //
            (FRAC_PI_4, FRAC_PI_4 * 1.0, cx + rad * FRAC_PI_4.sin()), // Q1, Q1
            (FRAC_PI_4, FRAC_PI_4 * 2.0, cx + rad),                   // Q1, PI/2
            (FRAC_PI_4, FRAC_PI_4 * 3.0, cx + rad),                   // Q1, Q2
            (FRAC_PI_4, FRAC_PI_4 * 4.0, cx + rad),                   // Q1, PI
            (FRAC_PI_4, FRAC_PI_4 * 5.0, cx + rad),                   // Q1, Q3
            (FRAC_PI_4, FRAC_PI_4 * 6.0, cx + rad),                   // Q1, 3PI/2
            (FRAC_PI_4, FRAC_PI_4 * 7.0, cx + rad),                   // Q1, Q4
            (FRAC_PI_4, FRAC_PI_4 * 8.0, cx + rad),                   // Q1, 2PI
            //
            (FRAC_PI_2, FRAC_PI_4 * 2.0, cx + rad), // PI/2, PI/2
            (FRAC_PI_2, FRAC_PI_4 * 3.0, cx + rad), // PI/2, Q2
            (FRAC_PI_2, FRAC_PI_4 * 4.0, cx + rad), // PI/2, PI
            (FRAC_PI_2, FRAC_PI_4 * 5.0, cx + rad), // PI/2, Q3
            (FRAC_PI_2, FRAC_PI_4 * 6.0, cx + rad), // PI/2, 3PI/2
            (FRAC_PI_2, FRAC_PI_4 * 7.0, cx + rad), // PI/2, Q4
            (FRAC_PI_2, FRAC_PI_4 * 8.0, cx + rad), // PI/2, 2PI
            //
            (3.0 * FRAC_PI_4, FRAC_PI_4 * 3.0, cx + rad * FRAC_PI_4.sin()), // Q2, Q2
            (3.0 * FRAC_PI_4, FRAC_PI_4 * 4.0, cx + rad * FRAC_PI_4.sin()), // Q2, PI
            (3.0 * FRAC_PI_4, FRAC_PI_4 * 5.0, cx + rad * FRAC_PI_4.sin()), // Q2, Q3
            (3.0 * FRAC_PI_4, FRAC_PI_4 * 6.0, cx + rad * FRAC_PI_4.sin()), // Q2, 3PI/2
            (3.0 * FRAC_PI_4, FRAC_PI_4 * 7.0, cx + rad * FRAC_PI_4.sin()), // Q2, Q4
            (3.0 * FRAC_PI_4, FRAC_PI_4 * 8.0, cx + rad * FRAC_PI_4.sin()), // Q2, 2PI
            //
            (PI, FRAC_PI_4 * 4.0, cx), // PI, PI
            (PI, FRAC_PI_4 * 5.0, cx), // PI, Q3
            (PI, FRAC_PI_4 * 6.0, cx), // PI, 3PI/2
            (PI, FRAC_PI_4 * 7.0, cx), // PI, Q4
            (PI, FRAC_PI_4 * 8.0, cx), // PI, 2PI
            //
            (5.0 * FRAC_PI_4, FRAC_PI_4 * 5.0, cx - rad * FRAC_PI_4.sin()), // Q3, Q3
            (5.0 * FRAC_PI_4, FRAC_PI_4 * 6.0, cx - rad * FRAC_PI_4.sin()), // Q3, 3PI/2
            (5.0 * FRAC_PI_4, FRAC_PI_4 * 7.0, cx - rad * FRAC_PI_4.sin()), // Q3, Q4
            (5.0 * FRAC_PI_4, FRAC_PI_4 * 8.0, cx),                         // Q3, 2PI
            //
            (6.0 * FRAC_PI_4, FRAC_PI_4 * 6.0, cx - rad), // 3PI/2, 3PI/2
            (6.0 * FRAC_PI_4, FRAC_PI_4 * 7.0, cx - rad * FRAC_PI_4.sin()), // 3PI/2, Q4
            (6.0 * FRAC_PI_4, FRAC_PI_4 * 8.0, cx),       // 3PI/2, 2PI
            //
            (7.0 * FRAC_PI_4, FRAC_PI_4 * 7.0, cx - rad * FRAC_PI_4.sin()), // Q4, Q4
            (7.0 * FRAC_PI_4, FRAC_PI_4 * 8.0, cx),                         // Q4, 2PI
            //
            (8.0 * FRAC_PI_4, FRAC_PI_4 * 8.0, cx), // 2PI, 2PI
        ] {
            let ca = CurveArc::new(ctr, a1, a2, rad);
            assert_float_eq!(ca.top_bound(), expected, abs <= 0.000_01);
        }
    }
}
