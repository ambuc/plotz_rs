//! A 2D point.
#![allow(missing_docs)]

use crate::{
    bounded::{Bounded, Bounds},
    obj2::ObjType2d,
    shapes::{polygon::abp, segment::Segment},
    *,
};
use anyhow::Result;
use float_cmp::approx_eq;
use float_ord::FloatOrd;
use std::{convert::From, fmt::Debug, hash::Hash, ops::*};

#[derive(Copy, Clone)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Point {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        FloatOrd(self.x)
            .cmp(&FloatOrd(other.x))
            .then(FloatOrd(self.y).cmp(&FloatOrd(other.y)))
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        FloatOrd(self.x).eq(&FloatOrd(other.x)) && (FloatOrd(self.y).eq(&FloatOrd(other.y)))
    }
}

impl Eq for Point {}

impl Hash for Point {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        FloatOrd(self.x).hash(state);
        FloatOrd(self.y).hash(state);
    }
}

impl Debug for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Point { x, y } = self;
        write!(f, "Pt({:.1},{:.1})", x, y)
    }
}

/// An alternate constructor for points.
#[allow(non_snake_case)]
pub fn Point<T1, T2>(x: T1, y: T2) -> Point
where
    f64: From<T1>,
    f64: From<T2>,
{
    Point {
        y: y.into(),
        x: x.into(),
    }
}

impl From<f64> for Point {
    fn from(n: f64) -> Self {
        Point { x: n, y: n }
    }
}

impl<T1, T2> From<(T1, T2)> for Point
where
    f64: From<T1>,
    f64: From<T2>,
{
    fn from((x, y): (T1, T2)) -> Self {
        Point {
            x: x.into(),
            y: y.into(),
        }
    }
}

/// An alternate constructor for points which accepts an angle in radians.
#[allow(non_snake_case)]
pub fn PolarPt<T>(r: T, theta: T) -> Point
where
    f64: From<T>,
{
    let theta: f64 = theta.into();
    let r: f64 = r.into();
    Point {
        x: r * theta.cos(),
        y: r * theta.sin(),
    }
}

macro_rules! ops_trait {
    ($trait:ident, $fn:ident) => {
        impl<T> $trait<T> for Point
        where
            T: Into<Point>,
        {
            type Output = Self;
            fn $fn(self, rhs: T) -> Self::Output {
                let rhs = rhs.into();
                Point(self.x.$fn(rhs.x), self.y.$fn(rhs.y))
            }
        }
    };
}

macro_rules! ops_mut_trait {
    ($trait:ident, $fn:ident) => {
        impl<T> $trait<T> for Point
        where
            T: Into<Point>,
        {
            fn $fn(&mut self, rhs: T) {
                let rhs = rhs.into();
                self.x.$fn(rhs.x);
                self.y.$fn(rhs.y);
            }
        }
    };
}

ops_mut_trait!(AddAssign, add_assign);
ops_mut_trait!(DivAssign, div_assign);
ops_mut_trait!(MulAssign, mul_assign);
ops_mut_trait!(RemAssign, rem_assign);
ops_mut_trait!(SubAssign, sub_assign);
ops_trait!(Add, add);
ops_trait!(Div, div);
ops_trait!(Mul, mul);
ops_trait!(Rem, rem);
ops_trait!(Sub, sub);

impl Point {
    /// A rotation operation, for rotating one point about another. Accepts a |by|
    /// argument in radians.
    pub fn rotate_inplace(&mut self, about: &Point, by: f64) {
        *self -= *about;
        *self = Point(
            (by.cos() * self.x) - (by.sin() * self.y),
            (by.sin() * self.x) + (by.cos() * self.y),
        );
        *self += *about;
    }

    /// rotate
    #[must_use]
    pub fn rotate(&self, about: &Point, by: f64) -> Point {
        let mut n = *self;
        n.rotate_inplace(about, by);
        n
    }

    /// Dot prouduct of (origin, self) • (origin, other)
    pub fn dot(&self, other: &Point) -> f64 {
        (self.x * other.x) + (self.y * other.y)
    }

    /// Distance between two points.
    pub fn dist(&self, other: &Point) -> f64 {
        Segment(*self, *other).abs()
    }

    /// Average of two points.
    pub fn avg(&self, other: &Point) -> Point {
        Point((self.x + other.x) / 2.0, (self.y + other.y) / 2.0)
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
    pub fn angle_to(&self, other: &Point) -> f64 {
        let o = self;
        let j = other;
        let i = Point(other.x, self.y);
        abp(o, &i, j)
    }
}

impl Bounded for Point {
    fn bounds(&self) -> Result<Bounds> {
        Ok(Bounds {
            y_max: self.y,
            y_min: self.y,
            x_min: self.x,
            x_max: self.x,
        })
    }
}

impl Object for Point {
    fn objtype(&self) -> ObjType2d {
        ObjType2d::Point2d
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &Point> + '_> {
        Box::new(std::iter::once(self))
    }

    fn iter_mut(&mut self) -> Box<dyn Iterator<Item = &mut Point> + '_> {
        Box::new(std::iter::once(self))
    }
}

/// Returns true if all the points are colinear.
pub fn is_colinear_n(ch: &Vec<Point>) -> bool {
    if ch.len() <= 2 {
        return false;
    }
    ch[2..].iter().all(|p| is_colinear_3(ch[0], ch[1], *p))
}

fn is_colinear_3(p1: Point, p2: Point, p3: Point) -> bool {
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

        let origin = Point(0, 0);
        let mut p = Point(1, 0);

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
        assert_float_eq!(Point(1, 1).dot(&Point(1, 0)), 1.0, abs <= 0.000_1);
        assert_float_eq!(Point(7, 2).dot(&Point(3, 6)), 33.0, abs <= 0.000_1);
    }

    #[test]
    fn test_rem() {
        assert_eq!(Point(1.5, 1.5) % (1.0, 1.0), Point(0.5, 0.5));
    }

    #[test]
    fn test_div_assign() {
        let mut p = Point(1.5, 1.5);
        p /= 2.0;
        assert_eq!(p, Point(0.75, 0.75));
    }

    #[test]
    fn test_add() {
        assert_eq!(Point(1, 2) + (3, 4), Point(4, 6));
    }

    #[test]
    fn test_add_assign() {
        let mut p = Point(2, 4);
        p += (1, 2);
        assert_eq!(p, Point(3, 6));
    }

    #[test]
    fn test_sub() {
        assert_eq!(Point(1, 2) - (3, 4), Point(-2, -2));
    }

    #[test]
    fn test_sub_assign() {
        let mut p = Point(2, 4);
        p -= (1, 2);
        assert_eq!(p, Point(1, 2));
    }

    #[test]
    fn test_mul() {
        assert_eq!(Point(1.0, 2.0) * 2.0, Point(2.0, 4.0));
    }

    #[test]
    fn test_div() {
        assert_eq!(Point(1.0, 2.0) / 2.0, Point(0.5, 1.0)); // floats
    }

    #[test_case(Point(0,0), Point(1,1), Point(2,2), true; "colinear diagonal")]
    #[test_case(Point(0,0), Point(1,0), Point(2,0), true; "colinear vert")]
    #[test_case(Point(0,0), Point(0,1), Point(0,2), true; "colinear horz")]
    #[test_case(Point(0,0), Point(0,1), Point(2,2), false; "not colinear")]
    #[test_case(Point(0,0), Point(0,1), Point(0.1, 0.1), false; "not colinear small")]
    #[test_case(Point(0,0), Point(0,1), Point(0.0001, 0.0001), false; "not colinear very small")]
    fn test_is_colinear_3(a: Point, b: Point, c: Point, expectation: bool) {
        assert_eq!(is_colinear_3(a, b, c), expectation);
    }

    #[test_case(&[], false; "empty")]
    #[test_case(&[Point(0,0)], false; "one")]
    #[test_case(&[Point(0,0), Point(0,1)], false; "two")]
    #[test_case(&[Point(0,0), Point(0,1), Point(0,2)], true; "three colinear")]
    #[test_case(&[Point(0,0), Point(0,1), Point(0,2), Point(0,3)], true; "four colinear")]
    #[test_case(&[Point(0,0), Point(0,1), Point(0,2), Point(1,3)], false; "four not colinear")]
    #[test_case(&[Point(0,0), Point(0,1), Point(0,2), Point(0,3), Point(0,4)], true; "five colinear")]
    #[test_case(&[Point(0,0), Point(0,1), Point(0,2), Point(0,3), Point(1,4)], false; "five not colinear")]
    fn test_is_colinear_n(pts: &[Point], expectation: bool) {
        assert_eq!(is_colinear_n(&pts.to_vec()), expectation);
    }
}
