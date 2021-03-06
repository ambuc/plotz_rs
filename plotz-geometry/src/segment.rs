use crate::interpolate::interpolate_2d_checked;
use crate::point::Pt;
use float_cmp::approx_eq;
use std::{
    cmp::PartialOrd,
    fmt::Debug,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
enum Orientation {
    Colinear,
    Clockwise,
    CounterClockwise,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Contains {
    Within,
    AtStart,
    AtEnd,
}

/// A struct representing an intersection between two line segments.
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Intersection {
    /// At which % of the way from self_i to self_f the intersection occurs. Will always be between 0.0 and 1.0.
    /// If this value is 0.0, the intersection is at self_i.
    /// If this value is 1.0, the intersection is at self_f.
    pub percent_along_self: f64,
    /// At which % of the way from other_i to other_f the intersection occurs. Will always be between 0.0 and 1.0.
    /// If this value is 0.0, the intersection is at other_i.
    /// If this value is 1.0, the intersection is at other_f.
    pub percent_along_other: f64,
}

impl Intersection {
    fn on_points_of_self(&self) -> bool {
        self.percent_along_self == 0.0 || self.percent_along_self == 1.0
    }
    fn on_points_of_other(&self) -> bool {
        self.percent_along_other == 0.0 || self.percent_along_other == 1.0
    }
    pub fn on_points_of_either_polygon(&self) -> bool {
        self.on_points_of_self() || self.on_points_of_other()
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum IntersectionOutcome {
    LineSegmentsAreTheSame,
    LineSegmentsAreTheSameButReversed,
    LineSegmentsAreColinear,
    Yes(Intersection),
}

/// A segment in 2D space, with initial and final points.
#[derive(Debug, Clone, Copy)]
pub struct Segment {
    /// The initial point of the segment.
    pub i: Pt,
    /// The final point of the segment.
    pub f: Pt,
}

impl PartialEq for Segment {
    fn eq(&self, other: &Self) -> bool {
        self.i == other.i && self.f == other.f
    }
}

/// An alternate constructor for segments.
#[allow(non_snake_case)]
pub fn Segment(i: Pt, f: Pt) -> Segment {
    Segment { i, f }
}

impl Segment {
    // Internal helper function; see https://www.geeksforgeeks.org/check-if-two-given-line-segments-intersect/.
    fn _ccw(&self, other: &Pt) -> Orientation {
        use std::cmp::Ordering;
        match PartialOrd::partial_cmp(
            &((other.y.0 - self.i.y.0) * (self.f.x.0 - self.i.x.0)
                - (self.f.y.0 - self.i.y.0) * (other.x.0 - self.i.x.0)),
            &0_f64,
        ) {
            Some(Ordering::Equal) => Orientation::Colinear,
            Some(Ordering::Greater) => Orientation::Clockwise,
            Some(Ordering::Less) => Orientation::CounterClockwise,
            None => panic!("!"),
        }
    }

    /// The slope of a line segment.
    pub fn slope(&self) -> f64 {
        (self.f.y.0 - self.i.y.0) / (self.f.x.0 - self.i.x.0)
    }

    /// A rotation operation, for rotating a line segment about a point. Accepts
    /// a |by| argument in radians.
    pub fn rotate(&mut self, about: &Pt, by: f64) {
        self.i.rotate(about, by);
        self.f.rotate(about, by);
    }

    // Returns true if this line segment has point |other| along it.
    pub fn line_segment_contains_pt(&self, other: &Pt) -> Option<Contains> {
        if *other == self.i {
            return Some(Contains::AtStart);
        }
        if *other == self.f {
            return Some(Contains::AtEnd);
        }
        let d1: f64 = self.abs();
        let d2: f64 = Segment(self.i, *other).abs() + Segment(self.f, *other).abs();
        if approx_eq!(f64, d1, d2) {
            return Some(Contains::Within);
        }
        None
    }

    /// Returns true if one line segment intersects another.
    /// If two line segments share a point, returns false.
    /// If two line segments are parallel and overlapping, returns false.
    /// If two line segments are the same, returns false.
    pub fn intersects(&self, other: &Segment) -> Option<IntersectionOutcome> {
        if self == other {
            return Some(IntersectionOutcome::LineSegmentsAreTheSame);
        }
        if *self == Segment(other.f, other.i) {
            return Some(IntersectionOutcome::LineSegmentsAreTheSameButReversed);
        }
        if self.slope() == other.slope()
            && (self.f == other.i || other.f == self.i || self.i == other.i || self.f == other.f)
        {
            return Some(IntersectionOutcome::LineSegmentsAreColinear);
        }

        if let Some(pt) = self.get_line_intersection_inner(
            (self.i.x.0, self.i.y.0),
            (self.f.x.0, self.f.y.0),
            (other.i.x.0, other.i.y.0),
            (other.f.x.0, other.f.y.0),
        ) {
            return Some(IntersectionOutcome::Yes(Intersection {
                percent_along_self: interpolate_2d_checked(self.i, self.f, pt).ok()?,
                percent_along_other: interpolate_2d_checked(other.i, other.f, pt).ok()?,
            }));
        }
        None
    }

    /// If two line segments are parallel and overlapping, returns None.
    /// If two line segments are the same, returns None.
    fn get_line_intersection_inner(
        &self,
        (p0_x, p0_y): (f64, f64),
        (p1_x, p1_y): (f64, f64),
        (p2_x, p2_y): (f64, f64),
        (p3_x, p3_y): (f64, f64),
    ) -> Option<Pt> {
        let s1_x = p1_x - p0_x;
        let s1_y = p1_y - p0_y;
        let s2_x = p3_x - p2_x;
        let s2_y = p3_y - p2_y;

        let s = (-s1_y * (p0_x - p2_x) + s1_x * (p0_y - p2_y)) / (-s2_x * s1_y + s1_x * s2_y);
        let t = (s2_x * (p0_y - p2_y) - s2_y * (p0_x - p2_x)) / (-s2_x * s1_y + s1_x * s2_y);

        if (0_f64..=1_f64).contains(&s) && (0_f64..=1_f64).contains(&t) {
            let i_x = p0_x + (t * s1_x);
            let i_y = p0_y + (t * s1_y);
            return Some(Pt(i_x, i_y));
        }
        None
    }

    pub fn abs(&self) -> f64 {
        let two = 2_f64;
        ((self.f.y.0 - self.i.y.0).powf(two) + (self.f.x.0 - self.i.x.0).powf(two)).sqrt()
    }
}

/// An add operation between a segment and a point. This can be seen as
/// transposition by |rhs|.
impl Add<Pt> for Segment {
    type Output = Segment;
    fn add(self, rhs: Pt) -> Self::Output {
        Segment(self.i + rhs, self.f + rhs)
    }
}

/// An add-assign operation between a segment and a point. This can be seen as a
/// transposition by |rhs|.
impl AddAssign<Pt> for Segment {
    fn add_assign(&mut self, rhs: Pt) {
        *self = Segment(self.i + rhs, self.f + rhs);
    }
}

/// A division operation between a segment and a point. This can be seen as
/// scaling by |rhs|.
impl Div<f64> for Segment {
    type Output = Segment;
    fn div(self, rhs: f64) -> Self::Output {
        Segment(self.i / rhs, self.f / rhs)
    }
}

/// An division-assign operation between a segment and a point. This can be seen
/// as a scaling by |rhs|.
impl DivAssign<f64> for Segment {
    fn div_assign(&mut self, rhs: f64) {
        *self = Segment(self.i / rhs, self.f / rhs)
    }
}

/// A multiplication operation between a segment and a point. This can be seen
/// as scaling by |rhs|.
impl Mul<f64> for Segment {
    type Output = Segment;
    fn mul(self, rhs: f64) -> Self::Output {
        Segment(self.i * rhs, self.f * rhs)
    }
}

/// An multiplication-assign operation between a segment and a point. This can
/// be seen as a scaling by |rhs|.
impl MulAssign<f64> for Segment {
    fn mul_assign(&mut self, rhs: f64) {
        *self = Segment(self.i * rhs, self.f * rhs);
    }
}

/// A subtraction operation between a segment and a point. This can be seen
/// as translation by |rhs|.
impl Sub<Pt> for Segment {
    type Output = Segment;
    fn sub(self, rhs: Pt) -> Self::Output {
        Segment {
            i: self.i - rhs,
            f: self.f - rhs,
        }
    }
}

/// An subtraction-assign operation between a segment and a point. This can
/// be seen as translation by |rhs|.
impl SubAssign<Pt> for Segment {
    fn sub_assign(&mut self, rhs: Pt) {
        *self = Segment(self.i - rhs, self.f - rhs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slope() {
        //   ^
        //   |
        //   A  B  C
        //   |
        //   D  E  F
        //   |
        // --G--H--I->
        //   |
        let a = Pt(0.0, 2.0);
        let b = Pt(1.0, 2.0);
        let c = Pt(2.0, 2.0);
        let d = Pt(0.0, 1.0);
        let e = Pt(1.0, 1.0);
        let f = Pt(2.0, 1.0);
        let g = Pt(0.0, 0.0);
        let h = Pt(1.0, 0.0);
        let i = Pt(2.0, 0.0);

        // m=0
        assert_eq!(Segment(g, h).slope(), 0.0);
        assert_eq!(Segment(g, i).slope(), 0.0);

        // m=0.5
        assert_eq!(Segment(g, f).slope(), 0.5);
        assert_eq!(Segment(d, c).slope(), 0.5);

        // m=1
        assert_eq!(Segment(g, e).slope(), 1.0);
        assert_eq!(Segment(g, c).slope(), 1.0);

        // m=2.0
        assert_eq!(Segment(h, c).slope(), 2.0);
        assert_eq!(Segment(g, b).slope(), 2.0);

        // m=inf
        assert_eq!(Segment(g, a).slope(), std::f64::INFINITY);
        assert_eq!(Segment(g, d).slope(), std::f64::INFINITY);

        // m=-0.5
        assert_eq!(Segment(a, f).slope(), -0.5);
        assert_eq!(Segment(d, i).slope(), -0.5);

        // m=-1.0
        assert_eq!(Segment(a, e).slope(), -1.0);
        assert_eq!(Segment(a, i).slope(), -1.0);

        // m=-2.0
        assert_eq!(Segment(b, i).slope(), -2.0);
        assert_eq!(Segment(a, h).slope(), -2.0);

        // m=-inf
        assert_eq!(Segment(a, g).slope(), -1.0 * std::f64::INFINITY);
        assert_eq!(Segment(d, g).slope(), -1.0 * std::f64::INFINITY);

        // slope is independent of start/end
        assert_eq!(Segment(a, c).slope(), Segment(c, a).slope());
        assert_eq!(Segment(a, f).slope(), Segment(f, a).slope());
        assert_eq!(Segment(a, i).slope(), Segment(i, a).slope());
        assert_eq!(Segment(a, h).slope(), Segment(h, a).slope());
    }

    #[test]
    fn test_rotate() {
        use float_eq::assert_float_eq;
        use std::f64::consts::PI;

        let origin = Pt(0.0, 0.0);

        //      ^
        //      |
        //      |  F
        // <----+--I->
        //      |
        //      |
        //      v
        let mut s = Segment(Pt(1.0, 0.0), Pt(1.0, 0.5));

        s.rotate(/*about=*/ &origin, PI / 2.0);
        //      ^
        //     FI
        //      |
        // <----+---->
        //      |
        //      |
        //      v
        assert_float_eq!(s.i.x.0, 0.0, abs <= 0.000_1);
        assert_float_eq!(s.i.y.0, 1.0, abs <= 0.000_1);
        assert_float_eq!(s.f.x.0, -0.5, abs <= 0.000_1);
        assert_float_eq!(s.f.y.0, 1.0, abs <= 0.000_1);

        s.rotate(/*about=*/ &origin, PI / 2.0);
        //      ^
        //      |
        //      |
        // <-I--+---->
        //   F  |
        //      |
        //      v
        assert_float_eq!(s.i.x.0, -1.0, abs <= 0.000_1);
        assert_float_eq!(s.i.y.0, 0.0, abs <= 0.000_1);
        assert_float_eq!(s.f.x.0, -1.0, abs <= 0.000_1);
        assert_float_eq!(s.f.y.0, -0.5, abs <= 0.000_1);

        s.rotate(/*about=*/ &origin, PI / 2.0);
        //      ^
        //      |
        //      |
        // <----+---->
        //      |
        //      IF
        //      v
        assert_float_eq!(s.i.x.0, 0.0, abs <= 0.000_1);
        assert_float_eq!(s.i.y.0, -1.0, abs <= 0.000_1);
        assert_float_eq!(s.f.x.0, 0.5, abs <= 0.000_1);
        assert_float_eq!(s.f.y.0, -1.0, abs <= 0.000_1);

        s.rotate(/*about=*/ &origin, PI / 2.0);
        //      ^
        //      |
        //      |  F
        // <----+--I->
        //      |
        //      |
        //      v
        assert_float_eq!(s.i.x.0, 1.0, abs <= 0.000_1);
        assert_float_eq!(s.i.y.0, 0.0, abs <= 0.000_1);
        assert_float_eq!(s.f.x.0, 1.0, abs <= 0.000_1);
        assert_float_eq!(s.f.y.0, 0.5, abs <= 0.000_1);
    }

    #[test]
    fn test_equality() {
        let a = Pt(0.0, 2.0);
        let b = Pt(1.0, 2.0);
        assert!(Segment(a, b) == Segment(a, b));
        assert!(Segment(a, b) != Segment(b, a));
    }

    #[test]
    fn test_intersects() {
        //   ^
        //   |
        //   A  B  C
        //   |
        //   D  E  F
        //   |
        // --G--H--I->
        //   |
        let a = Pt(0.0, 2.0);
        let b = Pt(1.0, 2.0);
        let c = Pt(2.0, 2.0);
        let e = Pt(1.0, 1.0);
        let g = Pt(0.0, 0.0);
        let i = Pt(2.0, 0.0);

        // colinear
        assert_eq!(
            Segment(a, c).intersects(&Segment(a, c)),
            Some(IntersectionOutcome::LineSegmentsAreTheSame)
        );
        assert_eq!(
            Segment(a, c).intersects(&Segment(c, a)),
            Some(IntersectionOutcome::LineSegmentsAreTheSameButReversed)
        );
        // induce colinear
        assert_eq!(
            Segment(a, b).intersects(&Segment(b, c)),
            Some(IntersectionOutcome::LineSegmentsAreColinear)
        );
        assert_eq!(
            Segment(a, b).intersects(&Segment(c, b)),
            Some(IntersectionOutcome::LineSegmentsAreColinear)
        );
        assert_eq!(
            Segment(b, a).intersects(&Segment(b, c)),
            Some(IntersectionOutcome::LineSegmentsAreColinear)
        );
        assert_eq!(
            Segment(b, a).intersects(&Segment(c, b)),
            Some(IntersectionOutcome::LineSegmentsAreColinear)
        );

        // (s,w), (e,w), (w,s), (w,e)
        assert_eq!(
            Segment(e, i).intersects(&Segment(c, g)),
            Some(IntersectionOutcome::Yes(Intersection {
                percent_along_self: 0.0,
                percent_along_other: 0.5
            }))
        );
        assert_eq!(
            Segment(a, e).intersects(&Segment(c, g)),
            Some(IntersectionOutcome::Yes(Intersection {
                percent_along_self: 1.0,
                percent_along_other: 0.5
            }))
        );
        assert_eq!(
            Segment(c, g).intersects(&Segment(e, i)),
            Some(IntersectionOutcome::Yes(Intersection {
                percent_along_self: 0.5,
                percent_along_other: 0.0
            }))
        );
        assert_eq!(
            Segment(c, g).intersects(&Segment(a, e)),
            Some(IntersectionOutcome::Yes(Intersection {
                percent_along_self: 0.5,
                percent_along_other: 1.0
            }))
        );

        // // (s,s), (s,e), (e,s), (e,e)
        assert_eq!(
            Segment(a, c).intersects(&Segment(c, i)),
            Some(IntersectionOutcome::Yes(Intersection {
                percent_along_self: 1.0,
                percent_along_other: 0.0
            }))
        );
        assert_eq!(
            Segment(a, c).intersects(&Segment(i, c)),
            Some(IntersectionOutcome::Yes(Intersection {
                percent_along_self: 1.0,
                percent_along_other: 1.0
            }))
        );
        assert_eq!(
            Segment(a, c).intersects(&Segment(g, a)),
            Some(IntersectionOutcome::Yes(Intersection {
                percent_along_self: 0.0,
                percent_along_other: 1.0
            }))
        );
        assert_eq!(
            Segment(a, c).intersects(&Segment(a, g)),
            Some(IntersectionOutcome::Yes(Intersection {
                percent_along_self: 0.0,
                percent_along_other: 0.0
            }))
        );

        // // (w,w)
        assert_eq!(
            Segment(a, i).intersects(&Segment(c, g)),
            Some(IntersectionOutcome::Yes(Intersection {
                percent_along_self: 0.5,
                percent_along_other: 0.5
            }))
        );
    }

    #[test]
    fn test_abs() {
        assert_eq!(Segment(Pt(0.0, 0.0), Pt(0.0, 1.0)).abs(), 1.0);
        assert_eq!(Segment(Pt(0.0, 0.0), Pt(1.0, 1.0)).abs(), 2.0_f64.sqrt());
        assert_eq!(Segment(Pt(1.0, 1.0), Pt(1.0, 1.0)).abs(), 0.0);
        assert_eq!(
            Segment(Pt(-1.0, -1.0), Pt(1.0, 1.0)).abs(),
            2.0 * 2.0_f64.sqrt()
        );
    }

    #[test]
    fn test_line_segment_contains_pt() {
        //   ^
        //   |
        //   A  B  C
        //   |
        //   D  E  F
        //   |
        // --G--H--I->
        //   |
        let a = Pt(0.0, 2.0);
        let c = Pt(2.0, 2.0);

        assert_eq!(
            Segment(a, c).line_segment_contains_pt(&a).unwrap(),
            Contains::AtStart
        );
    }
    #[test]
    fn test_segment() {
        assert_eq!(
            Segment {
                i: Pt(0, 0),
                f: Pt(0, 1)
            },
            Segment(Pt(0, 0), Pt(0, 1))
        );
    }

    #[test]
    fn test_add() {
        assert_eq!(
            Segment(Pt(0, 0), Pt(1, 1)) + Pt(1, 0),
            Segment(Pt(1, 0), Pt(2, 1))
        );
    }

    #[test]
    fn test_add_assign() {
        let mut s = Segment(Pt(0, 0), Pt(1, 1));
        s += Pt(1, 0);
        assert_eq!(s, Segment(Pt(1, 0), Pt(2, 1)));
    }

    #[test]
    fn test_div() {
        assert_eq!(
            Segment(Pt(0.0, 0.0), Pt(1.0, 1.0)) / 2.0,
            Segment(Pt(0.0, 0.0), Pt(0.5, 0.5))
        );
    }

    #[test]
    fn test_div_assign() {
        let mut s = Segment(Pt(0.0, 0.0), Pt(1.0, 1.0));
        s /= 2.0;
        assert_eq!(s, Segment(Pt(0.0, 0.0), Pt(0.5, 0.5)));
    }

    #[test]
    fn test_mul() {
        assert_eq!(
            Segment(Pt(0.0, 0.0), Pt(1.0, 1.0)) * 2.0,
            Segment(Pt(0.0, 0.0), Pt(2.0, 2.0))
        );
    }

    #[test]
    fn test_mul_assign() {
        let mut s = Segment(Pt(0.0, 0.0), Pt(1.0, 1.0));
        s *= 2.0;
        assert_eq!(s, Segment(Pt(0.0, 0.0), Pt(2.0, 2.0)));
    }

    #[test]
    fn test_sub() {
        assert_eq!(
            Segment(Pt(0.0, 0.0), Pt(1.0, 1.0)) - Pt(1.0, 2.0),
            // --------
            Segment(Pt(-1.0, -2.0), Pt(0.0, -1.0))
        );
    }

    #[test]
    fn test_sub_assign() {
        let mut s = Segment(Pt(0.0, 0.0), Pt(1.0, 1.0));
        s -= Pt(1.0, 2.0);
        assert_eq!(s, Segment(Pt(-1.0, -2.0), Pt(0.0, -1.0)));
    }
}
