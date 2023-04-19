//! A 2D point.

use crate::{
    bounded::{Bounded, Bounds},
    polygon::abp,
    segment::Segment,
    traits::*,
};
use {
    float_ord::FloatOrd,
    std::hash::Hash,
    std::{convert::From, fmt::Debug, ops::*},
};

/// A point in 2D space.
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Pt {
    /// The x-coordinate of the point.
    pub x: FloatOrd<f64>,
    /// The y-coordinate of the point.
    pub y: FloatOrd<f64>,
}

impl Debug for Pt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Pt({:.2}, {:.2})", self.x.0, self.y.0)
    }
}

/// An alternate constructor for points.
#[allow(non_snake_case)]
pub fn Pt<T>(x: T, y: T) -> Pt
where
    f64: From<T>,
{
    Pt {
        x: FloatOrd(x.into()),
        y: FloatOrd(y.into()),
    }
}

/// An alternate constructor for points which accepts an angle in radians.
#[allow(non_snake_case)]
pub fn PolarPt<T>(r: T, theta: T) -> Pt
where
    f64: From<T>,
{
    let theta: f64 = theta.into();
    let r: f64 = r.into();
    Pt {
        x: FloatOrd(r * theta.cos()),
        y: FloatOrd(r * theta.sin()),
    }
}

impl From<(f64, f64)> for Pt {
    fn from((x, y): (f64, f64)) -> Pt {
        Pt(x, y)
    }
}

impl Rem<(f64, f64)> for Pt {
    type Output = Self;

    fn rem(self, modulus: (f64, f64)) -> Self::Output {
        Pt(self.x.0 % modulus.0, self.y.0 % modulus.1)
    }
}

impl Add<Pt> for Pt {
    type Output = Self;
    fn add(self, rhs: Pt) -> Self::Output {
        Pt(self.x.0 + rhs.x.0, self.y.0 + rhs.y.0)
    }
}
impl AddAssign<Pt> for Pt {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: FloatOrd(self.x.0 + other.x.0),
            y: FloatOrd(self.y.0 + other.y.0),
        };
    }
}
impl Div<Pt> for Pt {
    type Output = Self;
    fn div(self, rhs: Pt) -> Self::Output {
        Pt(self.x.0 / rhs.x.0, self.y.0 / rhs.y.0)
    }
}
impl Div<f64> for Pt {
    type Output = Self;
    fn div(self, rhs: f64) -> Self::Output {
        Pt(self.x.0 / rhs, self.y.0 / rhs)
    }
}
impl DivAssign<Pt> for Pt {
    fn div_assign(&mut self, rhs: Pt) {
        self.x.0 /= rhs.x.0;
        self.y.0 /= rhs.y.0;
    }
}
impl DivAssign<f64> for Pt {
    fn div_assign(&mut self, rhs: f64) {
        self.x.0 /= rhs;
        self.y.0 /= rhs;
    }
}
impl Mul<Pt> for Pt {
    type Output = Self;
    fn mul(self, rhs: Pt) -> Self::Output {
        Pt(self.x.0 * rhs.x.0, self.y.0 * rhs.y.0)
    }
}
impl Mul<f64> for Pt {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Pt(self.x.0 * rhs, self.y.0 * rhs)
    }
}
impl MulAssign<Pt> for Pt {
    fn mul_assign(&mut self, rhs: Pt) {
        self.x.0 *= rhs.x.0;
        self.y.0 *= rhs.y.0;
    }
}
impl MulAssign<f64> for Pt {
    fn mul_assign(&mut self, rhs: f64) {
        self.x.0 *= rhs;
        self.y.0 *= rhs;
    }
}
impl RemAssign<Pt> for Pt {
    fn rem_assign(&mut self, rhs: Pt) {
        self.x.0 = self.x.0.rem_euclid(rhs.x.0);
        self.y.0 = self.y.0.rem_euclid(rhs.y.0);
    }
}
impl Sub<Pt> for Pt {
    type Output = Self;
    fn sub(self, rhs: Pt) -> Self::Output {
        Pt(self.x.0 - rhs.x.0, self.y.0 - rhs.y.0)
    }
}
impl SubAssign<Pt> for Pt {
    fn sub_assign(&mut self, other: Self) {
        *self = Self {
            x: FloatOrd(self.x.0 - other.x.0),
            y: FloatOrd(self.y.0 - other.y.0),
        };
    }
}

impl Pt {
    /// A rotation operation, for rotating one point about another. Accepts a |by|
    /// argument in radians.
    pub fn rotate_inplace(&mut self, about: &Pt, by: f64) {
        *self -= *about;
        *self = Pt(
            (by.cos() * self.x.0) - (by.sin() * self.y.0),
            (by.sin() * self.x.0) + (by.cos() * self.y.0),
        );
        *self += *about;
    }

    /// rotate
    #[must_use]
    pub fn rotate(&self, about: &Pt, by: f64) -> Pt {
        let mut n = *self;
        n.rotate_inplace(about, by);
        n
    }

    /// Dot prouduct of (origin, self) • (origin, other)
    pub fn dot(&self, other: &Pt) -> f64 {
        (self.x.0 * other.x.0) + (self.y.0 * other.y.0)
    }

    /// Distance between two points.
    pub fn dist(&self, other: &Pt) -> f64 {
        Segment(*self, *other).abs()
    }

    /// Average of two points.
    pub fn avg(&self, other: &Pt) -> Pt {
        Pt((self.x.0 + other.x.0) / 2.0, (self.y.0 + other.y.0) / 2.0)
    }

    /// Flip x
    pub fn flip_x(&mut self) {
        self.x.0 *= -1.0;
    }

    /// Flip y
    pub fn flip_y(&mut self) {
        self.y.0 *= -1.0;
    }

    /// angle from here to there.
    pub fn angle_to(&self, other: &Pt) -> f64 {
        let o = self;
        let j = other;
        let i = Pt(other.x.0, self.y.0);
        abp(o, &i, j)
    }
}

impl YieldPoints for Pt {
    fn yield_pts(&self) -> Option<Box<dyn Iterator<Item = &Pt> + '_>> {
        Some(Box::new(std::iter::once(self)))
    }
}
impl YieldPointsMut for Pt {
    fn yield_pts_mut(&mut self) -> Option<Box<dyn Iterator<Item = &mut Pt> + '_>> {
        Some(Box::new(std::iter::once(self)))
    }
}
impl Mutable for Pt {}

impl Bounded for Pt {
    fn bounds(&self) -> crate::bounded::Bounds {
        Bounds {
            top_bound: self.y.0,
            bottom_bound: self.y.0,
            left_bound: self.x.0,
            right_bound: self.x.0,
        }
    }
}

impl Translatable for Pt {}
impl Scalable<Pt> for Pt {}
impl Scalable<f64> for Pt {}

impl Roundable for Pt {
    fn round_to_nearest(&mut self, f: f64) {
        self.x.0 -= self.x.0 % f;
        self.y.0 -= self.y.0 % f;
    }
}

impl Nullable for Pt {
    fn is_empty(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;

    use super::*;

    #[test]
    fn test_rotate() {
        use float_eq::assert_float_eq;
        use std::f64::consts::PI;

        let origin = Pt(0.0, 0.0);
        let mut p = Pt(1.0, 0.0);

        p.rotate_inplace(/*about=*/ &origin, PI / 2.0);
        assert_float_eq!(p.x.0, 0.0, abs <= 0.000_1);
        assert_float_eq!(p.y.0, 1.0, abs <= 0.000_1);

        p.rotate_inplace(/*about=*/ &origin, PI / 2.0);
        assert_float_eq!(p.x.0, -1.0, abs <= 0.000_1);
        assert_float_eq!(p.y.0, 0.0, abs <= 0.000_1);

        p.rotate_inplace(/*about=*/ &origin, PI / 2.0);
        assert_float_eq!(p.x.0, 0.0, abs <= 0.000_1);
        assert_float_eq!(p.y.0, -1.0, abs <= 0.000_1);

        p.rotate_inplace(/*about=*/ &origin, PI / 2.0);
        assert_float_eq!(p.x.0, 1.0, abs <= 0.000_1);
        assert_float_eq!(p.y.0, 0.0, abs <= 0.000_1);
    }

    #[test]
    fn test_dot() {
        assert_float_eq!(Pt(1.0, 1.0).dot(&Pt(1.0, 0.0)), 1.0, abs <= 0.000_1);
        assert_float_eq!(Pt(7.0, 2.0).dot(&Pt(3.0, 6.0)), 33.0, abs <= 0.000_1);
    }

    #[test]
    fn test_rem() {
        assert_eq!(Pt(1.5, 1.5) % (1.0, 1.0), Pt(0.5, 0.5));
    }

    #[test]
    fn test_div_assign() {
        let mut p = Pt(1.5, 1.5);
        p /= 2.0;
        assert_eq!(p, Pt(0.75, 0.75));
    }

    #[test]
    fn test_add() {
        assert_eq!(Pt(1, 2) + Pt(3, 4), Pt(4, 6));
    }

    #[test]
    fn test_add_assign() {
        let mut p = Pt(2, 4);
        p += Pt(1, 2);
        assert_eq!(p, Pt(3, 6));
    }

    #[test]
    fn test_sub() {
        assert_eq!(Pt(1, 2) - Pt(3, 4), Pt(-2, -2));
    }

    #[test]
    fn test_sub_assign() {
        let mut p = Pt(2, 4);
        p -= Pt(1, 2);
        assert_eq!(p, Pt(1, 2));
    }

    #[test]
    fn test_mul() {
        assert_eq!(Pt(1.0, 2.0) * 2.0, Pt(2.0, 4.0));
    }

    #[test]
    fn test_div() {
        assert_eq!(Pt(1.0, 2.0) / 2.0, Pt(0.5, 1.0)); // floats
    }
}
