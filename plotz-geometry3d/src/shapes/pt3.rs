//! A 3D point.
//!
use crate::shapes::sg3::Sg3;
use float_ord::FloatOrd;
use std::{convert::From, fmt::Debug, hash::Hash, ops::*};

#[derive(Hash, Copy, Clone, PartialOrd, PartialEq, Eq, Ord)]
pub struct Pt3 {
    pub x: FloatOrd<f64>,
    pub y: FloatOrd<f64>,
    pub z: FloatOrd<f64>,
}

impl Debug for Pt3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Pt3 { x, y, z } = self;
        write!(f, "p3!({:.2},{:.2},{:.2})", x.0, y.0, z.0)
    }
}

// An alternate constructor for points.
#[allow(non_snake_case)]
pub fn Pt3<T1, T2, T3>(x: T1, y: T2, z: T3) -> Pt3
where
    f64: From<T1>,
    f64: From<T2>,
    f64: From<T3>,
{
    Pt3 {
        x: FloatOrd(x.into()),
        y: FloatOrd(y.into()),
        z: FloatOrd(z.into()),
    }
}

// Pt3 shortcut.
#[macro_export]
macro_rules! p3 {
    ($x:expr, $y:expr, $z:expr) => {
        Pt3($x, $y, $z)
    };
}

impl From<(f64, f64, f64)> for Pt3 {
    fn from((x, y, z): (f64, f64, f64)) -> Pt3 {
        p3!(x, y, z)
    }
}

impl Add<Pt3> for Pt3 {
    type Output = Self;
    fn add(self, rhs: Pt3) -> Self::Output {
        p3!(self.x.0 + rhs.x.0, self.y.0 + rhs.y.0, self.z.0 + rhs.z.0)
    }
}
impl AddAssign<Pt3> for Pt3 {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: FloatOrd(self.x.0 + other.x.0),
            y: FloatOrd(self.y.0 + other.y.0),
            z: FloatOrd(self.z.0 + other.z.0),
        };
    }
}

impl Div<Pt3> for Pt3 {
    type Output = Self;
    fn div(self, rhs: Pt3) -> Self::Output {
        p3!(self.x.0 / rhs.x.0, self.y.0 / rhs.y.0, self.z.0 / rhs.z.0)
    }
}
impl Div<f64> for Pt3 {
    type Output = Self;
    fn div(self, rhs: f64) -> Self::Output {
        p3!(self.x.0 / rhs, self.y.0 / rhs, self.z.0 / rhs)
    }
}
impl DivAssign<Pt3> for Pt3 {
    fn div_assign(&mut self, rhs: Pt3) {
        self.x.0 /= rhs.x.0;
        self.y.0 /= rhs.y.0;
        self.z.0 /= rhs.z.0;
    }
}
impl DivAssign<f64> for Pt3 {
    fn div_assign(&mut self, rhs: f64) {
        self.x.0 /= rhs;
        self.y.0 /= rhs;
        self.z.0 /= rhs;
    }
}
impl Mul<Pt3> for Pt3 {
    type Output = Self;
    fn mul(self, rhs: Pt3) -> Self::Output {
        p3!(self.x.0 * rhs.x.0, self.y.0 * rhs.y.0, self.z.0 * rhs.z.0)
    }
}
impl Mul<f64> for Pt3 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        p3!(self.x.0 * rhs, self.y.0 * rhs, self.z.0 * rhs)
    }
}
impl MulAssign<Pt3> for Pt3 {
    fn mul_assign(&mut self, rhs: Pt3) {
        self.x.0 *= rhs.x.0;
        self.y.0 *= rhs.y.0;
        self.z.0 *= rhs.z.0;
    }
}
impl MulAssign<f64> for Pt3 {
    fn mul_assign(&mut self, rhs: f64) {
        self.x.0 *= rhs;
        self.y.0 *= rhs;
        self.z.0 *= rhs;
    }
}
impl Sub<Pt3> for Pt3 {
    type Output = Self;
    fn sub(self, rhs: Pt3) -> Self::Output {
        p3!(self.x.0 - rhs.x.0, self.y.0 - rhs.y.0, self.z.0 - rhs.z.0)
    }
}
impl SubAssign<Pt3> for Pt3 {
    fn sub_assign(&mut self, other: Self) {
        *self = Self {
            x: FloatOrd(self.x.0 - other.x.0),
            y: FloatOrd(self.y.0 - other.y.0),
            z: FloatOrd(self.z.0 - other.z.0),
        };
    }
}

impl Pt3 {
    // https://en.wikipedia.org/wiki/Dot_product
    pub fn dot(&self, other: &Pt3) -> f64 {
        (self.x.0 * other.x.0) + (self.y.0 * other.y.0) + (self.z.0 * other.z.0)
    }
    // average of two points.
    pub fn avg(&self, other: &Pt3) -> Pt3 {
        let avg_x = (self.x.0 + other.x.0) / 2.0;
        let avg_y = (self.y.0 + other.y.0) / 2.0;
        let avg_z = (self.z.0 + other.z.0) / 2.0;
        p3!(avg_x, avg_y, avg_z)
    }

    /// Distance between two points.
    pub fn dist(&self, other: &Pt3) -> f64 {
        Sg3(*self, *other).abs()
    }
}
