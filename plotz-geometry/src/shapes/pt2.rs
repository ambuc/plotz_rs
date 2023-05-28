//! A 2D point.

use crate::{
    bounded::{Bounded, Bounds},
    shapes::{pg2::abp, sg2::Sg2},
    traits::*,
};
use {
    float_cmp::approx_eq,
    float_ord::FloatOrd,
    std::hash::Hash,
    std::{convert::From, fmt::Debug, ops::*},
};

/// A point in 2D space.
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Pt2 {
    /// The x-coordinate of the point.
    pub x: FloatOrd<f64>,
    /// The y-coordinate of the point.
    pub y: FloatOrd<f64>,
}

/// Pt shortcut.
#[macro_export]
macro_rules! p2 {
    ($x:expr, $y:expr) => {
        Pt2($x, $y)
    };
}

impl Debug for Pt2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Pt2 { x, y } = self;
        write!(f, "Pt({:.10},{:.10})", x.0, y.0)
    }
}

/// An alternate constructor for points.
#[allow(non_snake_case)]
pub fn Pt2<T>(x: T, y: T) -> Pt2
where
    f64: From<T>,
{
    Pt2 {
        x: FloatOrd(x.into()),
        y: FloatOrd(y.into()),
    }
}

/// An alternate constructor for points which accepts an angle in radians.
#[allow(non_snake_case)]
pub fn PolarPt<T>(r: T, theta: T) -> Pt2
where
    f64: From<T>,
{
    let theta: f64 = theta.into();
    let r: f64 = r.into();
    Pt2 {
        x: FloatOrd(r * theta.cos()),
        y: FloatOrd(r * theta.sin()),
    }
}

impl From<(f64, f64)> for Pt2 {
    fn from((x, y): (f64, f64)) -> Pt2 {
        p2!(x, y)
    }
}

impl Rem<(f64, f64)> for Pt2 {
    type Output = Self;

    fn rem(self, modulus: (f64, f64)) -> Self::Output {
        p2!(self.x.0 % modulus.0, self.y.0 % modulus.1)
    }
}

impl Add<Pt2> for Pt2 {
    type Output = Self;
    fn add(self, rhs: Pt2) -> Self::Output {
        p2!(self.x.0 + rhs.x.0, self.y.0 + rhs.y.0)
    }
}
impl AddAssign<Pt2> for Pt2 {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: FloatOrd(self.x.0 + other.x.0),
            y: FloatOrd(self.y.0 + other.y.0),
        };
    }
}
impl Div<Pt2> for Pt2 {
    type Output = Self;
    fn div(self, rhs: Pt2) -> Self::Output {
        p2!(self.x.0 / rhs.x.0, self.y.0 / rhs.y.0)
    }
}
impl Div<f64> for Pt2 {
    type Output = Self;
    fn div(self, rhs: f64) -> Self::Output {
        p2!(self.x.0 / rhs, self.y.0 / rhs)
    }
}
impl DivAssign<Pt2> for Pt2 {
    fn div_assign(&mut self, rhs: Pt2) {
        self.x.0 /= rhs.x.0;
        self.y.0 /= rhs.y.0;
    }
}
impl DivAssign<f64> for Pt2 {
    fn div_assign(&mut self, rhs: f64) {
        self.x.0 /= rhs;
        self.y.0 /= rhs;
    }
}
impl Mul<Pt2> for Pt2 {
    type Output = Self;
    fn mul(self, rhs: Pt2) -> Self::Output {
        p2!(self.x.0 * rhs.x.0, self.y.0 * rhs.y.0)
    }
}
impl Mul<f64> for Pt2 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        p2!(self.x.0 * rhs, self.y.0 * rhs)
    }
}
impl MulAssign<Pt2> for Pt2 {
    fn mul_assign(&mut self, rhs: Pt2) {
        self.x.0 *= rhs.x.0;
        self.y.0 *= rhs.y.0;
    }
}
impl MulAssign<f64> for Pt2 {
    fn mul_assign(&mut self, rhs: f64) {
        self.x.0 *= rhs;
        self.y.0 *= rhs;
    }
}
impl RemAssign<Pt2> for Pt2 {
    fn rem_assign(&mut self, rhs: Pt2) {
        self.x.0 = self.x.0.rem_euclid(rhs.x.0);
        self.y.0 = self.y.0.rem_euclid(rhs.y.0);
    }
}
impl Sub<Pt2> for Pt2 {
    type Output = Self;
    fn sub(self, rhs: Pt2) -> Self::Output {
        p2!(self.x.0 - rhs.x.0, self.y.0 - rhs.y.0)
    }
}
impl SubAssign<Pt2> for Pt2 {
    fn sub_assign(&mut self, other: Self) {
        *self = Self {
            x: FloatOrd(self.x.0 - other.x.0),
            y: FloatOrd(self.y.0 - other.y.0),
        };
    }
}

impl Pt2 {
    /// A rotation operation, for rotating one point about another. Accepts a |by|
    /// argument in radians.
    pub fn rotate_inplace(&mut self, about: &Pt2, by: f64) {
        *self -= *about;
        *self = p2!(
            (by.cos() * self.x.0) - (by.sin() * self.y.0),
            (by.sin() * self.x.0) + (by.cos() * self.y.0)
        );
        *self += *about;
    }

    /// rotate
    #[must_use]
    pub fn rotate(&self, about: &Pt2, by: f64) -> Pt2 {
        let mut n = *self;
        n.rotate_inplace(about, by);
        n
    }

    /// Dot prouduct of (origin, self) • (origin, other)
    pub fn dot(&self, other: &Pt2) -> f64 {
        (self.x.0 * other.x.0) + (self.y.0 * other.y.0)
    }

    /// Distance between two points.
    pub fn dist(&self, other: &Pt2) -> f64 {
        Sg2(*self, *other).abs()
    }

    /// Average of two points.
    pub fn avg(&self, other: &Pt2) -> Pt2 {
        p2!((self.x.0 + other.x.0) / 2.0, (self.y.0 + other.y.0) / 2.0)
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
    pub fn angle_to(&self, other: &Pt2) -> f64 {
        let o = self;
        let j = other;
        let i = p2!(other.x.0, self.y.0);
        abp(o, &i, j)
    }
}

impl YieldPoints for Pt2 {
    fn yield_pts(&self) -> Box<dyn Iterator<Item = &Pt2> + '_> {
        Box::new(std::iter::once(self))
    }
}
impl YieldPointsMut for Pt2 {
    fn yield_pts_mut(&mut self) -> Box<dyn Iterator<Item = &mut Pt2> + '_> {
        Box::new(std::iter::once(self))
    }
}
impl Mutable for Pt2 {}

impl Bounded for Pt2 {
    fn bounds(&self) -> crate::bounded::Bounds {
        Bounds {
            top_bound: self.y.0,
            bottom_bound: self.y.0,
            left_bound: self.x.0,
            right_bound: self.x.0,
        }
    }
}

impl Translatable for Pt2 {}
impl Scalable<Pt2> for Pt2 {}
impl Scalable<f64> for Pt2 {}

impl Roundable for Pt2 {
    fn round_to_nearest(&mut self, f: f64) {
        self.x.0 -= self.x.0 % f;
        self.y.0 -= self.y.0 % f;
    }
}

impl Nullable for Pt2 {
    fn is_empty(&self) -> bool {
        false
    }
}

/// Returns true if all the points are colinear.
pub fn is_colinear_n(ch: &Vec<Pt2>) -> bool {
    if ch.len() <= 2 {
        return false;
    }
    ch[2..].iter().all(|p| is_colinear_3(ch[0], ch[1], *p))
}

fn is_colinear_3(p1: Pt2, p2: Pt2, p3: Pt2) -> bool {
    let a = p1.x.0;
    let b = p1.y.0;
    let m = p2.x.0;
    let n = p2.y.0;
    let x = p3.x.0;
    let y = p3.y.0;
    // (𝑛−𝑏)(𝑥−𝑚)=(𝑦−𝑛)(𝑚−𝑎)
    approx_eq!(f64, (n - b) * (x - m), (y - n) * (m - a))
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;
    use test_case::test_case;

    use super::*;

    #[test]
    fn test_rotate() {
        use float_eq::assert_float_eq;
        use std::f64::consts::PI;

        let origin = p2!(0.0, 0.0);
        let mut p = p2!(1.0, 0.0);

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
        assert_float_eq!(p2!(1.0, 1.0).dot(&p2!(1.0, 0.0)), 1.0, abs <= 0.000_1);
        assert_float_eq!(p2!(7.0, 2.0).dot(&p2!(3.0, 6.0)), 33.0, abs <= 0.000_1);
    }

    #[test]
    fn test_rem() {
        assert_eq!(p2!(1.5, 1.5) % (1.0, 1.0), p2!(0.5, 0.5));
    }

    #[test]
    fn test_div_assign() {
        let mut p = p2!(1.5, 1.5);
        p /= 2.0;
        assert_eq!(p, p2!(0.75, 0.75));
    }

    #[test]
    fn test_add() {
        assert_eq!(p2!(1, 2) + p2!(3, 4), p2!(4, 6));
    }

    #[test]
    fn test_add_assign() {
        let mut p = p2!(2, 4);
        p += p2!(1, 2);
        assert_eq!(p, p2!(3, 6));
    }

    #[test]
    fn test_sub() {
        assert_eq!(p2!(1, 2) - p2!(3, 4), p2!(-2, -2));
    }

    #[test]
    fn test_sub_assign() {
        let mut p = p2!(2, 4);
        p -= p2!(1, 2);
        assert_eq!(p, p2!(1, 2));
    }

    #[test]
    fn test_mul() {
        assert_eq!(p2!(1.0, 2.0) * 2.0, p2!(2.0, 4.0));
    }

    #[test]
    fn test_div() {
        assert_eq!(p2!(1.0, 2.0) / 2.0, p2!(0.5, 1.0)); // floats
    }

    #[test_case(p2!(0,0), p2!(1,1), p2!(2,2), true; "colinear diagonal")]
    #[test_case(p2!(0,0), p2!(1,0), p2!(2,0), true; "colinear vert")]
    #[test_case(p2!(0,0), p2!(0,1), p2!(0,2), true; "colinear horz")]
    #[test_case(p2!(0,0), p2!(0,1), p2!(2,2), false; "not colinear")]
    #[test_case(p2!(0,0), p2!(0,1), p2!(0.1, 0.1), false; "not colinear small")]
    #[test_case(p2!(0,0), p2!(0,1), p2!(0.0001, 0.0001), false; "not colinear very small")]
    fn test_is_colinear_3(a: Pt2, b: Pt2, c: Pt2, expectation: bool) {
        assert_eq!(is_colinear_3(a, b, c), expectation);
    }

    #[test_case(&[], false; "empty")]
    #[test_case(&[p2!(0,0)], false; "one")]
    #[test_case(&[p2!(0,0), p2!(0,1)], false; "two")]
    #[test_case(&[p2!(0,0), p2!(0,1), p2!(0,2)], true; "three colinear")]
    #[test_case(&[p2!(0,0), p2!(0,1), p2!(0,2), p2!(0,3)], true; "four colinear")]
    #[test_case(&[p2!(0,0), p2!(0,1), p2!(0,2), p2!(1,3)], false; "four not colinear")]
    #[test_case(&[p2!(0,0), p2!(0,1), p2!(0,2), p2!(0,3), p2!(0,4)], true; "five colinear")]
    #[test_case(&[p2!(0,0), p2!(0,1), p2!(0,2), p2!(0,3), p2!(1,4)], false; "five not colinear")]
    fn test_is_colinear_n(pts: &[Pt2], expectation: bool) {
        assert_eq!(is_colinear_n(&pts.to_vec()), expectation);
    }
}
