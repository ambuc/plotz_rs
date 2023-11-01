//! A 2D point.

use crate::{
    bounded::{Bounded, Bounds},
    shapes::{pg::abp, sg::Sg},
    *,
};
use anyhow::Result;
use float_cmp::approx_eq;
use float_ord::FloatOrd;
use std::{convert::From, fmt::Debug, hash::Hash, ops::*};

/// A point in 2D space.
#[derive(Copy, Clone)]
pub struct Pt {
    /// The x-coordinate of the point.
    pub x: f64,
    /// The y-coordinate of the point.
    pub y: f64,
}

impl PartialOrd for Pt {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Pt {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        FloatOrd(self.x)
            .cmp(&FloatOrd(other.x))
            .then(FloatOrd(self.y).cmp(&FloatOrd(other.y)))
    }
}

impl PartialEq for Pt {
    fn eq(&self, other: &Self) -> bool {
        FloatOrd(self.x).eq(&FloatOrd(other.x)) && (FloatOrd(self.y).eq(&FloatOrd(other.y)))
    }
}

impl Eq for Pt {}

impl Hash for Pt {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        FloatOrd(self.x).hash(state);
        FloatOrd(self.y).hash(state);
    }
}

impl Debug for Pt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Pt { x, y } = self;
        write!(f, "Pt({:.1},{:.1})", x, y)
    }
}

/// An alternate constructor for points.
#[allow(non_snake_case)]
pub fn Pt<T1, T2>(x: T1, y: T2) -> Pt
where
    f64: From<T1>,
    f64: From<T2>,
{
    Pt {
        x: x.into(),
        y: y.into(),
    }
}

impl From<f64> for Pt {
    fn from(n: f64) -> Self {
        Pt { x: n, y: n }
    }
}

impl<T1, T2> From<(T1, T2)> for Pt
where
    f64: From<T1>,
    f64: From<T2>,
{
    fn from((x, y): (T1, T2)) -> Self {
        Pt {
            x: x.into(),
            y: y.into(),
        }
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
        x: r * theta.cos(),
        y: r * theta.sin(),
    }
}

impl Rem<(f64, f64)> for Pt {
    type Output = Self;

    fn rem(self, modulus: (f64, f64)) -> Self::Output {
        Pt(self.x % modulus.0, self.y % modulus.1)
    }
}

impl<T> Add<T> for Pt
where
    T: Into<Pt>,
{
    type Output = Self;
    fn add(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        Pt(self.x + rhs.x, self.y + rhs.y)
    }
}
impl<T> AddAssign<T> for Pt
where
    T: Into<Pt>,
{
    fn add_assign(&mut self, other: T) {
        let other = other.into();
        *self = Self {
            x: self.x + other.x,
            y: self.y + other.y,
        };
    }
}
impl<T> Div<T> for Pt
where
    T: Into<Pt>,
{
    type Output = Self;
    fn div(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        Pt(self.x / rhs.x, self.y / rhs.y)
    }
}
impl<T> DivAssign<T> for Pt
where
    T: Into<Pt>,
{
    fn div_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        self.x /= rhs.x;
        self.y /= rhs.y;
    }
}
impl<T> Mul<T> for Pt
where
    T: Into<Pt>,
{
    type Output = Self;
    fn mul(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        Pt(self.x * rhs.x, self.y * rhs.y)
    }
}
impl<T> MulAssign<T> for Pt
where
    T: Into<Pt>,
{
    fn mul_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}
impl<T> RemAssign<T> for Pt
where
    T: Into<Pt>,
{
    fn rem_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        self.x = self.x.rem_euclid(rhs.x);
        self.y = self.y.rem_euclid(rhs.y);
    }
}
impl<T> Sub<T> for Pt
where
    T: Into<Pt>,
{
    type Output = Self;
    fn sub(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        Pt(self.x - rhs.x, self.y - rhs.y)
    }
}
impl<T> SubAssign<T> for Pt
where
    T: Into<Pt>,
{
    fn sub_assign(&mut self, other: T) {
        let other = other.into();
        *self = Self {
            x: self.x - other.x,
            y: self.y - other.y,
        };
    }
}

impl Pt {
    /// A rotation operation, for rotating one point about another. Accepts a |by|
    /// argument in radians.
    pub fn rotate_inplace(&mut self, about: &Pt, by: f64) {
        *self -= *about;
        *self = Pt(
            (by.cos() * self.x) - (by.sin() * self.y),
            (by.sin() * self.x) + (by.cos() * self.y),
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
        (self.x * other.x) + (self.y * other.y)
    }

    /// Distance between two points.
    pub fn dist(&self, other: &Pt) -> f64 {
        Sg(*self, *other).abs()
    }

    /// Average of two points.
    pub fn avg(&self, other: &Pt) -> Pt {
        Pt((self.x + other.x) / 2.0, (self.y + other.y) / 2.0)
    }

    /// Flip x
    pub fn flip_x(&mut self) {
        self.x *= -1.0;
    }

    /// Flip y
    pub fn flip_y(&mut self) {
        self.y *= -1.0;
    }

    /// angle from here to there.
    pub fn angle_to(&self, other: &Pt) -> f64 {
        let o = self;
        let j = other;
        let i = Pt(other.x, self.y);
        abp(o, &i, j)
    }

    /// Iterator.
    pub fn iter(&self) -> impl Iterator<Item = &Pt> {
        std::iter::once(self)
    }

    /// Mutable iterator.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Pt> {
        std::iter::once(self)
    }
}

impl Bounded for Pt {
    fn bounds(&self) -> Result<Bounds> {
        Ok(Bounds {
            top_bound: self.y,
            bottom_bound: self.y,
            left_bound: self.x,
            right_bound: self.x,
        })
    }
}

impl Translatable for Pt {}
impl Scalable<Pt> for Pt {}
impl Scalable<f64> for Pt {}

impl Roundable for Pt {
    fn round_to_nearest(&mut self, f: f64) {
        self.x -= self.x % f;
        self.y -= self.y % f;
    }
}

impl Nullable for Pt {
    fn is_empty(&self) -> bool {
        false
    }
}

/// Returns true if all the points are colinear.
pub fn is_colinear_n(ch: &Vec<Pt>) -> bool {
    if ch.len() <= 2 {
        return false;
    }
    ch[2..].iter().all(|p| is_colinear_3(ch[0], ch[1], *p))
}

fn is_colinear_3(p1: Pt, p2: Pt, p3: Pt) -> bool {
    let a = p1.x;
    let b = p1.y;
    let m = p2.x;
    let n = p2.y;
    let x = p3.x;
    let y = p3.y;
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

        let origin = Pt(0, 0);
        let mut p = Pt(1, 0);

        p.rotate_inplace(/*about=*/ &origin, PI / 2.0);
        assert_float_eq!(p.x, 0.0, abs <= 0.000_1);
        assert_float_eq!(p.y, 1.0, abs <= 0.000_1);

        p.rotate_inplace(/*about=*/ &origin, PI / 2.0);
        assert_float_eq!(p.x, -1.0, abs <= 0.000_1);
        assert_float_eq!(p.y, 0.0, abs <= 0.000_1);

        p.rotate_inplace(/*about=*/ &origin, PI / 2.0);
        assert_float_eq!(p.x, 0.0, abs <= 0.000_1);
        assert_float_eq!(p.y, -1.0, abs <= 0.000_1);

        p.rotate_inplace(/*about=*/ &origin, PI / 2.0);
        assert_float_eq!(p.x, 1.0, abs <= 0.000_1);
        assert_float_eq!(p.y, 0.0, abs <= 0.000_1);
    }

    #[test]
    fn test_dot() {
        assert_float_eq!(Pt(1, 1).dot(&Pt(1, 0)), 1.0, abs <= 0.000_1);
        assert_float_eq!(Pt(7, 2).dot(&Pt(3, 6)), 33.0, abs <= 0.000_1);
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
        assert_eq!(Pt(1, 2) + (3, 4), Pt(4, 6));
    }

    #[test]
    fn test_add_assign() {
        let mut p = Pt(2, 4);
        p += (1, 2);
        assert_eq!(p, Pt(3, 6));
    }

    #[test]
    fn test_sub() {
        assert_eq!(Pt(1, 2) - (3, 4), Pt(-2, -2));
    }

    #[test]
    fn test_sub_assign() {
        let mut p = Pt(2, 4);
        p -= (1, 2);
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

    #[test_case(Pt(0,0), Pt(1,1), Pt(2,2), true; "colinear diagonal")]
    #[test_case(Pt(0,0), Pt(1,0), Pt(2,0), true; "colinear vert")]
    #[test_case(Pt(0,0), Pt(0,1), Pt(0,2), true; "colinear horz")]
    #[test_case(Pt(0,0), Pt(0,1), Pt(2,2), false; "not colinear")]
    #[test_case(Pt(0,0), Pt(0,1), Pt(0.1, 0.1), false; "not colinear small")]
    #[test_case(Pt(0,0), Pt(0,1), Pt(0.0001, 0.0001), false; "not colinear very small")]
    fn test_is_colinear_3(a: Pt, b: Pt, c: Pt, expectation: bool) {
        assert_eq!(is_colinear_3(a, b, c), expectation);
    }

    #[test_case(&[], false; "empty")]
    #[test_case(&[Pt(0,0)], false; "one")]
    #[test_case(&[Pt(0,0), Pt(0,1)], false; "two")]
    #[test_case(&[Pt(0,0), Pt(0,1), Pt(0,2)], true; "three colinear")]
    #[test_case(&[Pt(0,0), Pt(0,1), Pt(0,2), Pt(0,3)], true; "four colinear")]
    #[test_case(&[Pt(0,0), Pt(0,1), Pt(0,2), Pt(1,3)], false; "four not colinear")]
    #[test_case(&[Pt(0,0), Pt(0,1), Pt(0,2), Pt(0,3), Pt(0,4)], true; "five colinear")]
    #[test_case(&[Pt(0,0), Pt(0,1), Pt(0,2), Pt(0,3), Pt(1,4)], false; "five not colinear")]
    fn test_is_colinear_n(pts: &[Pt], expectation: bool) {
        assert_eq!(is_colinear_n(&pts.to_vec()), expectation);
    }
}
