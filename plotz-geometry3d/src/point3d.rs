//! A 3D point.
//!
use {
    float_cmp::approx_eq,
    float_ord::FloatOrd,
    std::{convert::From, fmt::Debug, hash::Hash, ops::*},
};

/// A point in 2D space.
#[derive(Hash, Copy, Clone, PartialOrd, Ord)]
pub struct Pt3d {
    /// The x-coordinate of the point.
    pub x: FloatOrd<f64>,
    /// The y-coordinate of the point.
    pub y: FloatOrd<f64>,
    /// The z-coordinate of the point.
    pub z: FloatOrd<f64>,
}

impl PartialEq<Pt3d> for Pt3d {
    // TOPO(ambuc): derive_hash_xor_eq ???
    fn eq(&self, other: &Pt3d) -> bool {
        approx_eq!(f64, self.x.0, other.x.0, epsilon = 0.0000003)
            && approx_eq!(f64, self.y.0, other.y.0, epsilon = 0.0000003)
            && approx_eq!(f64, self.z.0, other.z.0, epsilon = 0.0000003)
    }
}

impl Eq for Pt3d {}

impl Debug for Pt3d {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Pt({:.2}, {:.2}, {:.2})", self.x.0, self.y.0, self.z.0)
    }
}

/// An alternate constructor for points.
#[allow(non_snake_case)]
pub fn Pt3d<T>(x: T, y: T, z: T) -> Pt3d
where
    f64: From<T>,
{
    Pt3d {
        x: FloatOrd(x.into()),
        y: FloatOrd(y.into()),
        z: FloatOrd(z.into()),
    }
}

impl From<(f64, f64, f64)> for Pt3d {
    fn from((x, y, z): (f64, f64, f64)) -> Pt3d {
        Pt3d(x, y, z)
    }
}

impl Add<Pt3d> for Pt3d {
    type Output = Self;
    fn add(self, rhs: Pt3d) -> Self::Output {
        Pt3d(self.x.0 + rhs.x.0, self.y.0 + rhs.y.0, self.z.0 + rhs.z.0)
    }
}
impl AddAssign<Pt3d> for Pt3d {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: FloatOrd(self.x.0 + other.x.0),
            y: FloatOrd(self.y.0 + other.y.0),
            z: FloatOrd(self.z.0 + other.z.0),
        };
    }
}

impl Div<Pt3d> for Pt3d {
    type Output = Self;
    fn div(self, rhs: Pt3d) -> Self::Output {
        Pt3d(self.x.0 / rhs.x.0, self.y.0 / rhs.y.0, self.z.0 / rhs.z.0)
    }
}
impl Div<f64> for Pt3d {
    type Output = Self;
    fn div(self, rhs: f64) -> Self::Output {
        Pt3d(self.x.0 / rhs, self.y.0 / rhs, self.z.0 / rhs)
    }
}
impl DivAssign<Pt3d> for Pt3d {
    fn div_assign(&mut self, rhs: Pt3d) {
        self.x.0 /= rhs.x.0;
        self.y.0 /= rhs.y.0;
        self.z.0 /= rhs.z.0;
    }
}
impl DivAssign<f64> for Pt3d {
    fn div_assign(&mut self, rhs: f64) {
        self.x.0 /= rhs;
        self.y.0 /= rhs;
        self.z.0 /= rhs;
    }
}
impl Mul<Pt3d> for Pt3d {
    type Output = Self;
    fn mul(self, rhs: Pt3d) -> Self::Output {
        Pt3d(self.x.0 * rhs.x.0, self.y.0 * rhs.y.0, self.z.0 * rhs.z.0)
    }
}
impl Mul<f64> for Pt3d {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Pt3d(self.x.0 * rhs, self.y.0 * rhs, self.z.0 * rhs)
    }
}
impl MulAssign<Pt3d> for Pt3d {
    fn mul_assign(&mut self, rhs: Pt3d) {
        self.x.0 *= rhs.x.0;
        self.y.0 *= rhs.y.0;
        self.z.0 *= rhs.z.0;
    }
}
impl MulAssign<f64> for Pt3d {
    fn mul_assign(&mut self, rhs: f64) {
        self.x.0 *= rhs;
        self.y.0 *= rhs;
        self.z.0 *= rhs;
    }
}
impl Sub<Pt3d> for Pt3d {
    type Output = Self;
    fn sub(self, rhs: Pt3d) -> Self::Output {
        Pt3d(self.x.0 - rhs.x.0, self.y.0 - rhs.y.0, self.z.0 - rhs.z.0)
    }
}
impl SubAssign<Pt3d> for Pt3d {
    fn sub_assign(&mut self, other: Self) {
        *self = Self {
            x: FloatOrd(self.x.0 - other.x.0),
            y: FloatOrd(self.y.0 - other.y.0),
            z: FloatOrd(self.z.0 - other.z.0),
        };
    }
}

impl Pt3d {
    /// https://en.wikipedia.org/wiki/Dot_product
    pub fn dot(&self, other: &Pt3d) -> f64 {
        (self.x.0 * other.x.0) + (self.y.0 * other.y.0) + (self.z.0 * other.z.0)
    }
}
