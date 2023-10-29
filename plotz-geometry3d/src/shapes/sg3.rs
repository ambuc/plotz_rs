//! A segment in 3d.

use crate::{
    bounded3::{Bounded3, Bounds3, Bounds3Collector},
    shapes::pt3::Pt3,
};
use anyhow::Result;
use float_ord::FloatOrd;
use std::{fmt::Debug, ops::*};

// A segment in 3d space, with initial and final points.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Sg3 {
    pub i: Pt3,
    pub f: Pt3,
}

#[allow(non_snake_case)]
pub fn Sg3(i: Pt3, f: Pt3) -> Sg3 {
    Sg3 { i, f }
}

impl Sg3 {
    // Returns the absolute value of the length of this segment.
    pub fn abs(&self) -> f64 {
        let two = 2_f64;
        (0_f64
            + (self.f.x.0 - self.i.x.0).powf(two)
            + (self.f.y.0 - self.i.y.0).powf(two)
            + (self.f.z.0 - self.i.z.0).powf(two))
        .sqrt()
    }

    // The average point of the polygon.
    pub fn average_pt(&self) -> Pt3 {
        self.i.avg(&self.f)
    }

    // The center of the object, projected along the view vector.
    pub fn dist_along(&self, view_vector: &Pt3) -> f64 {
        self.average_pt().dot(view_vector)
    }
    // the maximum distance along a vector.
    pub fn max_dist_along(&self, view_vector: &Pt3) -> f64 {
        [self.i, self.f]
            .iter()
            .map(|pt| FloatOrd(view_vector.dot(pt)))
            .max()
            .unwrap()
            .0
    }
    // the minimum distance along a vector.
    pub fn min_dist_along(&self, view_vector: &Pt3) -> f64 {
        [self.i, self.f]
            .iter()
            .map(|pt| FloatOrd(view_vector.dot(pt)))
            .min()
            .unwrap()
            .0
    }
}

impl Add<Pt3> for Sg3 {
    type Output = Sg3;
    fn add(self, rhs: Pt3) -> Self::Output {
        Sg3(self.i + rhs, self.f + rhs)
    }
}
impl AddAssign<Pt3> for Sg3 {
    fn add_assign(&mut self, rhs: Pt3) {
        *self = Sg3(self.i + rhs, self.f + rhs);
    }
}
impl Div<Pt3> for Sg3 {
    type Output = Sg3;
    fn div(self, rhs: Pt3) -> Self::Output {
        Sg3(self.i / rhs, self.f / rhs)
    }
}
impl Div<f64> for Sg3 {
    type Output = Sg3;
    fn div(self, rhs: f64) -> Self::Output {
        Sg3(self.i / rhs, self.f / rhs)
    }
}
impl DivAssign<Pt3> for Sg3 {
    fn div_assign(&mut self, rhs: Pt3) {
        *self = Sg3(self.i / rhs, self.f / rhs);
    }
}
impl DivAssign<f64> for Sg3 {
    fn div_assign(&mut self, rhs: f64) {
        *self = Sg3(self.i / rhs, self.f / rhs)
    }
}
impl Mul<Pt3> for Sg3 {
    type Output = Sg3;
    fn mul(self, rhs: Pt3) -> Self::Output {
        Sg3(self.i * rhs, self.f * rhs)
    }
}
impl Mul<f64> for Sg3 {
    type Output = Sg3;
    fn mul(self, rhs: f64) -> Self::Output {
        Sg3(self.i * rhs, self.f * rhs)
    }
}
impl MulAssign<Pt3> for Sg3 {
    fn mul_assign(&mut self, rhs: Pt3) {
        *self = Sg3(self.i * rhs, self.f * rhs);
    }
}
impl MulAssign<f64> for Sg3 {
    fn mul_assign(&mut self, rhs: f64) {
        *self = Sg3(self.i * rhs, self.f * rhs);
    }
}
impl Sub<Pt3> for Sg3 {
    type Output = Sg3;
    fn sub(self, rhs: Pt3) -> Self::Output {
        Sg3 {
            i: self.i - rhs,
            f: self.f - rhs,
        }
    }
}
impl SubAssign<Pt3> for Sg3 {
    fn sub_assign(&mut self, rhs: Pt3) {
        *self = Sg3(self.i - rhs, self.f - rhs);
    }
}

impl Bounded3 for Sg3 {
    fn bounds3(&self) -> Result<Bounds3> {
        let mut bc = Bounds3Collector::default();
        bc.incorporate(&self.i.bounds3()?)?;
        bc.incorporate(&self.f.bounds3()?)?;
        bc.bounds3()
    }
}
