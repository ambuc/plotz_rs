//! A 2D segment.
use crate::{
    bounded::{Bounded, Bounds},
    crop::{CropToPolygonError, Croppable, PointLoc},
    interpolate,
    interpolate::interpolate_2d_checked,
    isxn::{Intersection, IsxnResult, MultipleIntersections},
    point::Pt,
    polygon::Polygon,
    traits::*,
};
use float_cmp::approx_eq;
use std::{cmp::PartialOrd, fmt::Debug, ops::*};

#[derive(Debug, PartialEq, Eq)]
enum _Orientation {
    _Colinear,
    _Clockwise,
    _CounterClockwise,
}

/// Whether a line segment contains a point, and if so where.
#[derive(Debug, PartialEq, Eq)]
pub enum Contains {
    /// A line segment contains a point along it.
    Within,
    /// A line segment contains a point at its head.
    AtStart,
    /// A line segment contains a point at its tail.
    AtEnd,
}
/// A segment in 2D space, with initial and final points.
#[derive(Debug, Clone, Copy, Eq, Hash)]
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
    fn _ccw(&self, other: &Pt) -> _Orientation {
        use std::cmp::Ordering;
        match PartialOrd::partial_cmp(
            &((other.y.0 - self.i.y.0) * (self.f.x.0 - self.i.x.0)
                - (self.f.y.0 - self.i.y.0) * (other.x.0 - self.i.x.0)),
            &0_f64,
        ) {
            Some(Ordering::Equal) => _Orientation::_Colinear,
            Some(Ordering::Greater) => _Orientation::_Clockwise,
            Some(Ordering::Less) => _Orientation::_CounterClockwise,
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
        self.i.rotate_inplace(about, by);
        self.f.rotate_inplace(about, by);
    }

    /// Returns true if this line segment has point |other| along it.
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
    pub fn intersects(&self, other: &Segment) -> Option<IsxnResult> {
        if self == other {
            Some(IsxnResult::MultipleIntersections(
                MultipleIntersections::LineSegmentsAreTheSame,
            ))
        } else if *self == Segment(other.f, other.i) {
            Some(IsxnResult::MultipleIntersections(
                MultipleIntersections::LineSegmentsAreTheSameButReversed,
            ))
        } else if self.slope() == other.slope()
            && (self.f == other.i || other.f == self.i || self.i == other.i || self.f == other.f)
        {
            Some(IsxnResult::MultipleIntersections(
                MultipleIntersections::LineSegmentsAreColinear,
            ))
        } else if let Some(pt) = self.get_line_intersection_inner(
            (self.i.x.0, self.i.y.0),
            (self.f.x.0, self.f.y.0),
            (other.i.x.0, other.i.y.0),
            (other.f.x.0, other.f.y.0),
        ) {
            Some(IsxnResult::OneIntersection(
                Intersection::new(
                    pt,
                    interpolate_2d_checked(self.i, self.f, pt).ok()?,
                    interpolate_2d_checked(other.i, other.f, pt).ok()?,
                )
                .expect("valid intersection"),
            ))
        } else {
            None
        }
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

    /// Returns the absolute value of the length of this segment.
    pub fn abs(&self) -> f64 {
        let two = 2_f64;
        ((self.f.y.0 - self.i.y.0).powf(two) + (self.f.x.0 - self.i.x.0).powf(two)).sqrt()
    }

    /// Takes a lossy cross product of this with another segment (oriented tail-to-tail).
    pub fn cross_z(&self, other: &Segment) -> f64 {
        let d1 = self.f - self.i;
        let d2 = other.f - other.i;
        let x1 = d1.x.0;
        let x2 = d2.x.0;
        let y1 = d1.y.0;
        let y2 = d2.y.0;
        (x1 * y2) - (x2 * y1)
    }
}

impl Add<Pt> for Segment {
    type Output = Segment;
    fn add(self, rhs: Pt) -> Self::Output {
        Segment(self.i + rhs, self.f + rhs)
    }
}
impl AddAssign<Pt> for Segment {
    fn add_assign(&mut self, rhs: Pt) {
        *self = Segment(self.i + rhs, self.f + rhs);
    }
}
impl Div<Pt> for Segment {
    type Output = Segment;
    fn div(self, rhs: Pt) -> Self::Output {
        Segment(self.i / rhs, self.f / rhs)
    }
}
impl Div<f64> for Segment {
    type Output = Segment;
    fn div(self, rhs: f64) -> Self::Output {
        Segment(self.i / rhs, self.f / rhs)
    }
}
impl DivAssign<Pt> for Segment {
    fn div_assign(&mut self, rhs: Pt) {
        *self = Segment(self.i / rhs, self.f / rhs);
    }
}
impl DivAssign<f64> for Segment {
    fn div_assign(&mut self, rhs: f64) {
        *self = Segment(self.i / rhs, self.f / rhs)
    }
}
impl Mul<Pt> for Segment {
    type Output = Segment;
    fn mul(self, rhs: Pt) -> Self::Output {
        Segment(self.i * rhs, self.f * rhs)
    }
}
impl Mul<f64> for Segment {
    type Output = Segment;
    fn mul(self, rhs: f64) -> Self::Output {
        Segment(self.i * rhs, self.f * rhs)
    }
}
impl MulAssign<Pt> for Segment {
    fn mul_assign(&mut self, rhs: Pt) {
        *self = Segment(self.i * rhs, self.f * rhs);
    }
}
impl MulAssign<f64> for Segment {
    fn mul_assign(&mut self, rhs: f64) {
        *self = Segment(self.i * rhs, self.f * rhs);
    }
}
impl Sub<Pt> for Segment {
    type Output = Segment;
    fn sub(self, rhs: Pt) -> Self::Output {
        Segment {
            i: self.i - rhs,
            f: self.f - rhs,
        }
    }
}
impl SubAssign<Pt> for Segment {
    fn sub_assign(&mut self, rhs: Pt) {
        *self = Segment(self.i - rhs, self.f - rhs);
    }
}
impl RemAssign<Pt> for Segment {
    fn rem_assign(&mut self, rhs: Pt) {
        self.i %= rhs;
        self.f %= rhs;
    }
}

impl Bounded for Segment {
    fn bounds(&self) -> Bounds {
        Bounds {
            top_bound: std::cmp::max(self.i.y, self.f.y).0,
            bottom_bound: std::cmp::min(self.i.y, self.f.y).0,
            left_bound: std::cmp::min(self.i.x, self.f.x).0,
            right_bound: std::cmp::max(self.i.x, self.f.x).0,
        }
    }
}

impl YieldPoints for Segment {
    fn yield_pts(&self) -> Option<Box<dyn Iterator<Item = &Pt> + '_>> {
        Some(Box::new([&self.i, &self.f].into_iter()))
    }
}
impl YieldPointsMut for Segment {
    fn yield_pts_mut(&mut self) -> Option<Box<dyn Iterator<Item = &mut Pt> + '_>> {
        Some(Box::new([&mut self.i, &mut self.f].into_iter()))
    }
}
impl Mutable for Segment {}

impl Croppable for Segment {
    type Output = Segment;

    fn crop_to(&self, frame: &Polygon) -> Result<Vec<Self::Output>, CropToPolygonError>
    where
        Self: Sized,
    {
        let frame_segments = frame.to_segments();
        let mut resultants: Vec<Segment> = vec![];
        let mut curr_pt = self.i;
        let mut curr_pen_down = !matches!(frame.contains_pt(&self.i)?, PointLoc::Outside);

        if let (PointLoc::Inside, PointLoc::Inside) =
            (frame.contains_pt(&self.i)?, frame.contains_pt(&self.f)?)
        {
            resultants.push(*self);
        }

        loop {
            if curr_pt == self.f {
                break;
            }

            let mut isxns = frame_segments
                .iter()
                .filter_map(|f| self.intersects(f))
                .filter_map(|isxn_outcome| match isxn_outcome {
                    IsxnResult::OneIntersection(isxn) => Some(isxn),
                    _ => None,
                })
                .collect::<Vec<Intersection>>();
            isxns.sort_by_key(|i| i.percent_along_inner());
            let (_, vs) = isxns.into_iter().partition(|i| {
                i.percent_along_inner().0
                    <= interpolate::interpolate_2d_checked(self.i, self.f, curr_pt).unwrap_or_else(
                        |_| {
                            panic!(
                                "interpolate failed: a: {:?}, b: {:?}, i: {:?}",
                                self.i, self.f, curr_pt,
                            )
                        },
                    )
            });
            isxns = vs;

            match isxns.get(0) {
                Some(intersection) => {
                    let new_pt = interpolate::extrapolate_2d(
                        self.i,
                        self.f,
                        intersection.percent_along_inner().0,
                    );

                    // Not sure why. But escape the loop.
                    if new_pt == curr_pt {
                        return Ok(resultants);
                    }

                    if !matches!(frame.contains_pt(&new_pt)?, PointLoc::Outside) && curr_pen_down {
                        resultants.push(Segment(curr_pt, new_pt));
                    }
                    curr_pt = new_pt;
                    curr_pen_down = !curr_pen_down;
                }
                None => {
                    return Ok(resultants);
                }
            }
        }

        Ok(resultants)
    }
}

impl Translatable for Segment {}
impl Scalable<Pt> for Segment {}
impl Scalable<f64> for Segment {}

impl Roundable for Segment {
    fn round_to_nearest(&mut self, f: f64) {
        self.i.round_to_nearest(f);
        self.f.round_to_nearest(f);
    }
}

impl Nullable for Segment {
    fn is_empty(&self) -> bool {
        false
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
            Some(IsxnResult::MultipleIntersections(
                MultipleIntersections::LineSegmentsAreTheSame
            ))
        );
        assert_eq!(
            Segment(a, c).intersects(&Segment(c, a)),
            Some(IsxnResult::MultipleIntersections(
                MultipleIntersections::LineSegmentsAreTheSameButReversed
            ))
        );
        // induce colinear
        assert_eq!(
            Segment(a, b).intersects(&Segment(b, c)),
            Some(IsxnResult::MultipleIntersections(
                MultipleIntersections::LineSegmentsAreColinear
            ))
        );
        assert_eq!(
            Segment(a, b).intersects(&Segment(c, b)),
            Some(IsxnResult::MultipleIntersections(
                MultipleIntersections::LineSegmentsAreColinear
            ))
        );
        assert_eq!(
            Segment(b, a).intersects(&Segment(b, c)),
            Some(IsxnResult::MultipleIntersections(
                MultipleIntersections::LineSegmentsAreColinear
            ))
        );
        assert_eq!(
            Segment(b, a).intersects(&Segment(c, b)),
            Some(IsxnResult::MultipleIntersections(
                MultipleIntersections::LineSegmentsAreColinear
            ))
        );

        // (s,w), (e,w), (w,s), (w,e)
        assert_eq!(
            Segment(e, i).intersects(&Segment(c, g)),
            Some(IsxnResult::OneIntersection(
                Intersection::new(e, 0.0, 0.5).unwrap()
            ))
        );
        assert_eq!(
            Segment(a, e).intersects(&Segment(c, g)),
            Some(IsxnResult::OneIntersection(
                Intersection::new(e, 1.0, 0.5).unwrap()
            ))
        );
        assert_eq!(
            Segment(c, g).intersects(&Segment(e, i)),
            Some(IsxnResult::OneIntersection(
                Intersection::new(e, 0.5, 0.0).unwrap()
            ))
        );
        assert_eq!(
            Segment(c, g).intersects(&Segment(a, e)),
            Some(IsxnResult::OneIntersection(
                Intersection::new(e, 0.5, 1.0).unwrap()
            ))
        );

        // // (s,s), (s,e), (e,s), (e,e)
        assert_eq!(
            Segment(a, c).intersects(&Segment(c, i)),
            Some(IsxnResult::OneIntersection(
                Intersection::new(c, 1.0, -0.0).unwrap()
            ))
        );
        assert_eq!(
            Segment(a, c).intersects(&Segment(i, c)),
            Some(IsxnResult::OneIntersection(
                Intersection::new(c, 1.0, 1.0).unwrap()
            ))
        );
        assert_eq!(
            Segment(a, c).intersects(&Segment(g, a)),
            Some(IsxnResult::OneIntersection(
                Intersection::new(a, 0.0, 1.0).unwrap()
            )),
        );
        assert_eq!(
            Segment(a, c).intersects(&Segment(a, g)),
            Some(IsxnResult::OneIntersection(
                Intersection::new(a, 0.0, -0.0).unwrap()
            ))
        );

        // // (w,w)
        assert_eq!(
            Segment(a, i).intersects(&Segment(c, g)),
            Some(IsxnResult::OneIntersection(
                Intersection::new(e, 0.5, 0.5).unwrap()
            ))
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

    #[test]
    fn test_bounded_segment() {
        let s = Segment(Pt(0, 1), Pt(1, 2));
        assert_eq!(s.bottom_bound(), 1.0);
        assert_eq!(s.top_bound(), 2.0);
        assert_eq!(s.left_bound(), 0.0);
        assert_eq!(s.right_bound(), 1.0);
        assert_eq!(s.bl_bound(), Pt(0, 1));
        assert_eq!(s.tl_bound(), Pt(0, 2));
        assert_eq!(s.br_bound(), Pt(1, 1));
        assert_eq!(s.tr_bound(), Pt(1, 2));
    }
}
