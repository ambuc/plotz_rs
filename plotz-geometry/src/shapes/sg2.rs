//! A 2D segment.
use crate::{
    bounded::{Bounded, Bounds},
    crop::{CropType, Croppable, PointLoc},
    interpolate,
    interpolate::interpolate_2d_checked,
    isxn::{Intersection, IsxnResult, MultipleIntersections},
    p2,
    shapes::{pg2::Pg2, pt2::Pt2},
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
pub struct Sg2 {
    /// The initial point of the segment.
    pub i: Pt2,
    /// The final point of the segment.
    pub f: Pt2,
}

impl PartialEq for Sg2 {
    fn eq(&self, other: &Self) -> bool {
        self.i == other.i && self.f == other.f
    }
}

/// An alternate constructor for segments.
#[allow(non_snake_case)]
pub fn Sg2(i: Pt2, f: Pt2) -> Sg2 {
    Sg2 { i, f }
}

impl Sg2 {
    // Internal helper function; see https://www.geeksforgeeks.org/check-if-two-given-line-segments-intersect/.
    fn _ccw(&self, other: &Pt2) -> _Orientation {
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
    pub fn rotate(&mut self, about: &Pt2, by: f64) {
        self.i.rotate_inplace(about, by);
        self.f.rotate_inplace(about, by);
    }

    /// Returns true if this line segment has point |other| along it.
    pub fn line_segment_contains_pt(&self, other: &Pt2) -> Option<Contains> {
        if *other == self.i {
            return Some(Contains::AtStart);
        }
        if *other == self.f {
            return Some(Contains::AtEnd);
        }
        let d1: f64 = self.abs();
        let d2: f64 = Sg2(self.i, *other).abs() + Sg2(self.f, *other).abs();
        if approx_eq!(f64, d1, d2) {
            return Some(Contains::Within);
        }
        None
    }

    /// sometimes you just have to flip it.
    pub fn flip(&self) -> Sg2 {
        Sg2 {
            i: self.f,
            f: self.i,
        }
    }

    /// Returns true if one line segment intersects another.
    /// If two line segments share a point, returns false.
    /// If two line segments are parallel and overlapping, returns false.
    /// If two line segments are the same, returns false.
    pub fn intersects(&self, other: &Sg2) -> Option<IsxnResult> {
        if self == other {
            Some(IsxnResult::MultipleIntersections(
                MultipleIntersections::LineSegmentsAreTheSame,
            ))
        } else if *self == Sg2(other.f, other.i) {
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
    ) -> Option<Pt2> {
        let s1_x = p1_x - p0_x;
        let s1_y = p1_y - p0_y;
        let s2_x = p3_x - p2_x;
        let s2_y = p3_y - p2_y;

        let s = (-s1_y * (p0_x - p2_x) + s1_x * (p0_y - p2_y)) / (-s2_x * s1_y + s1_x * s2_y);
        let t = (s2_x * (p0_y - p2_y) - s2_y * (p0_x - p2_x)) / (-s2_x * s1_y + s1_x * s2_y);

        if (0_f64..=1_f64).contains(&s) && (0_f64..=1_f64).contains(&t) {
            let i_x = p0_x + (t * s1_x);
            let i_y = p0_y + (t * s1_y);
            return Some(p2!(i_x, i_y));
        }
        None
    }

    /// Returns the absolute value of the length of this segment.
    pub fn abs(&self) -> f64 {
        let two = 2_f64;
        ((self.f.y.0 - self.i.y.0).powf(two) + (self.f.x.0 - self.i.x.0).powf(two)).sqrt()
    }

    /// Takes a lossy cross product of this with another segment (oriented tail-to-tail).
    pub fn cross_z(&self, other: &Sg2) -> f64 {
        let d1 = self.f - self.i;
        let d2 = other.f - other.i;
        let x1 = d1.x.0;
        let x2 = d2.x.0;
        let y1 = d1.y.0;
        let y2 = d2.y.0;
        (x1 * y2) - (x2 * y1)
    }
}

impl Add<Pt2> for Sg2 {
    type Output = Sg2;
    fn add(self, rhs: Pt2) -> Self::Output {
        Sg2(self.i + rhs, self.f + rhs)
    }
}
impl AddAssign<Pt2> for Sg2 {
    fn add_assign(&mut self, rhs: Pt2) {
        *self = Sg2(self.i + rhs, self.f + rhs);
    }
}
impl Div<Pt2> for Sg2 {
    type Output = Sg2;
    fn div(self, rhs: Pt2) -> Self::Output {
        Sg2(self.i / rhs, self.f / rhs)
    }
}
impl Div<f64> for Sg2 {
    type Output = Sg2;
    fn div(self, rhs: f64) -> Self::Output {
        Sg2(self.i / rhs, self.f / rhs)
    }
}
impl DivAssign<Pt2> for Sg2 {
    fn div_assign(&mut self, rhs: Pt2) {
        *self = Sg2(self.i / rhs, self.f / rhs);
    }
}
impl DivAssign<f64> for Sg2 {
    fn div_assign(&mut self, rhs: f64) {
        *self = Sg2(self.i / rhs, self.f / rhs)
    }
}
impl Mul<Pt2> for Sg2 {
    type Output = Sg2;
    fn mul(self, rhs: Pt2) -> Self::Output {
        Sg2(self.i * rhs, self.f * rhs)
    }
}
impl Mul<f64> for Sg2 {
    type Output = Sg2;
    fn mul(self, rhs: f64) -> Self::Output {
        Sg2(self.i * rhs, self.f * rhs)
    }
}
impl MulAssign<Pt2> for Sg2 {
    fn mul_assign(&mut self, rhs: Pt2) {
        *self = Sg2(self.i * rhs, self.f * rhs);
    }
}
impl MulAssign<f64> for Sg2 {
    fn mul_assign(&mut self, rhs: f64) {
        *self = Sg2(self.i * rhs, self.f * rhs);
    }
}
impl Sub<Pt2> for Sg2 {
    type Output = Sg2;
    fn sub(self, rhs: Pt2) -> Self::Output {
        Sg2 {
            i: self.i - rhs,
            f: self.f - rhs,
        }
    }
}
impl SubAssign<Pt2> for Sg2 {
    fn sub_assign(&mut self, rhs: Pt2) {
        *self = Sg2(self.i - rhs, self.f - rhs);
    }
}
impl RemAssign<Pt2> for Sg2 {
    fn rem_assign(&mut self, rhs: Pt2) {
        self.i %= rhs;
        self.f %= rhs;
    }
}

impl Bounded for Sg2 {
    fn bounds(&self) -> Bounds {
        Bounds {
            top_bound: std::cmp::max(self.i.y, self.f.y).0,
            bottom_bound: std::cmp::min(self.i.y, self.f.y).0,
            left_bound: std::cmp::min(self.i.x, self.f.x).0,
            right_bound: std::cmp::max(self.i.x, self.f.x).0,
        }
    }
}

impl YieldPoints for Sg2 {
    fn yield_pts(&self) -> Box<dyn Iterator<Item = &Pt2> + '_> {
        Box::new([&self.i, &self.f].into_iter())
    }
}
impl YieldPointsMut for Sg2 {
    fn yield_pts_mut(&mut self) -> Box<dyn Iterator<Item = &mut Pt2> + '_> {
        Box::new([&mut self.i, &mut self.f].into_iter())
    }
}
impl Mutable for Sg2 {}

impl Croppable for Sg2 {
    type Output = Sg2;

    fn crop(&self, frame: &Pg2, crop_type: CropType) -> Vec<Self::Output>
    where
        Self: Sized,
    {
        assert_eq!(crop_type, CropType::Inclusive);

        let frame_segments = frame.to_segments();
        let mut resultants: Vec<Sg2> = vec![];
        let mut curr_pt = self.i;
        let mut curr_pen_down = !matches!(frame.contains_pt(&self.i), PointLoc::Outside);

        if let (PointLoc::Inside, PointLoc::Inside) =
            (frame.contains_pt(&self.i), frame.contains_pt(&self.f))
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
            isxns.sort_by_key(|i| i.percent_along_a());
            let (_, vs) = isxns.into_iter().partition(|i| {
                i.percent_along_a().0
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
                        intersection.percent_along_a().0,
                    );

                    // Not sure why. But escape the loop.
                    if new_pt == curr_pt {
                        return resultants;
                    }

                    if !matches!(frame.contains_pt(&new_pt), PointLoc::Outside) && curr_pen_down {
                        resultants.push(Sg2(curr_pt, new_pt));
                    }
                    curr_pt = new_pt;
                    curr_pen_down = !curr_pen_down;
                }
                None => {
                    return resultants;
                }
            }
        }

        resultants
    }

    fn crop_excluding(&self, _other: &Pg2) -> Vec<Self::Output>
    where
        Self: Sized,
    {
        unimplemented!("we haven't implemented segment crop excluding yet.");
    }
}

impl Translatable for Sg2 {}
impl Scalable<Pt2> for Sg2 {}
impl Scalable<f64> for Sg2 {}

impl Roundable for Sg2 {
    fn round_to_nearest(&mut self, f: f64) {
        self.i.round_to_nearest(f);
        self.f.round_to_nearest(f);
    }
}

impl Nullable for Sg2 {
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
        let a = p2!(0.0, 2.0);
        let b = p2!(1.0, 2.0);
        let c = p2!(2.0, 2.0);
        let d = p2!(0.0, 1.0);
        let e = p2!(1.0, 1.0);
        let f = p2!(2.0, 1.0);
        let g = p2!(0.0, 0.0);
        let h = p2!(1.0, 0.0);
        let i = p2!(2.0, 0.0);

        // m=0
        assert_eq!(Sg2(g, h).slope(), 0.0);
        assert_eq!(Sg2(g, i).slope(), 0.0);

        // m=0.5
        assert_eq!(Sg2(g, f).slope(), 0.5);
        assert_eq!(Sg2(d, c).slope(), 0.5);

        // m=1
        assert_eq!(Sg2(g, e).slope(), 1.0);
        assert_eq!(Sg2(g, c).slope(), 1.0);

        // m=2.0
        assert_eq!(Sg2(h, c).slope(), 2.0);
        assert_eq!(Sg2(g, b).slope(), 2.0);

        // m=inf
        assert_eq!(Sg2(g, a).slope(), std::f64::INFINITY);
        assert_eq!(Sg2(g, d).slope(), std::f64::INFINITY);

        // m=-0.5
        assert_eq!(Sg2(a, f).slope(), -0.5);
        assert_eq!(Sg2(d, i).slope(), -0.5);

        // m=-1.0
        assert_eq!(Sg2(a, e).slope(), -1.0);
        assert_eq!(Sg2(a, i).slope(), -1.0);

        // m=-2.0
        assert_eq!(Sg2(b, i).slope(), -2.0);
        assert_eq!(Sg2(a, h).slope(), -2.0);

        // m=-inf
        assert_eq!(Sg2(a, g).slope(), -1.0 * std::f64::INFINITY);
        assert_eq!(Sg2(d, g).slope(), -1.0 * std::f64::INFINITY);

        // slope is independent of start/end
        assert_eq!(Sg2(a, c).slope(), Sg2(c, a).slope());
        assert_eq!(Sg2(a, f).slope(), Sg2(f, a).slope());
        assert_eq!(Sg2(a, i).slope(), Sg2(i, a).slope());
        assert_eq!(Sg2(a, h).slope(), Sg2(h, a).slope());
    }

    #[test]
    fn test_rotate() {
        use float_eq::assert_float_eq;
        use std::f64::consts::PI;

        let origin = p2!(0.0, 0.0);

        //      ^
        //      |
        //      |  F
        // <----+--I->
        //      |
        //      |
        //      v
        let mut s = Sg2(p2!(1.0, 0.0), p2!(1.0, 0.5));

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
        let a = p2!(0.0, 2.0);
        let b = p2!(1.0, 2.0);
        assert!(Sg2(a, b) == Sg2(a, b));
        assert!(Sg2(a, b) != Sg2(b, a));
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
        let a = p2!(0.0, 2.0);
        let b = p2!(1.0, 2.0);
        let c = p2!(2.0, 2.0);
        let e = p2!(1.0, 1.0);
        let g = p2!(0.0, 0.0);
        let i = p2!(2.0, 0.0);

        // colinear
        assert_eq!(
            Sg2(a, c).intersects(&Sg2(a, c)),
            Some(IsxnResult::MultipleIntersections(
                MultipleIntersections::LineSegmentsAreTheSame
            ))
        );
        assert_eq!(
            Sg2(a, c).intersects(&Sg2(c, a)),
            Some(IsxnResult::MultipleIntersections(
                MultipleIntersections::LineSegmentsAreTheSameButReversed
            ))
        );
        // induce colinear
        assert_eq!(
            Sg2(a, b).intersects(&Sg2(b, c)),
            Some(IsxnResult::MultipleIntersections(
                MultipleIntersections::LineSegmentsAreColinear
            ))
        );
        assert_eq!(
            Sg2(a, b).intersects(&Sg2(c, b)),
            Some(IsxnResult::MultipleIntersections(
                MultipleIntersections::LineSegmentsAreColinear
            ))
        );
        assert_eq!(
            Sg2(b, a).intersects(&Sg2(b, c)),
            Some(IsxnResult::MultipleIntersections(
                MultipleIntersections::LineSegmentsAreColinear
            ))
        );
        assert_eq!(
            Sg2(b, a).intersects(&Sg2(c, b)),
            Some(IsxnResult::MultipleIntersections(
                MultipleIntersections::LineSegmentsAreColinear
            ))
        );

        // (s,w), (e,w), (w,s), (w,e)
        assert_eq!(
            Sg2(e, i).intersects(&Sg2(c, g)),
            Some(IsxnResult::OneIntersection(
                Intersection::new(e, 0.0, 0.5).unwrap()
            ))
        );
        assert_eq!(
            Sg2(a, e).intersects(&Sg2(c, g)),
            Some(IsxnResult::OneIntersection(
                Intersection::new(e, 1.0, 0.5).unwrap()
            ))
        );
        assert_eq!(
            Sg2(c, g).intersects(&Sg2(e, i)),
            Some(IsxnResult::OneIntersection(
                Intersection::new(e, 0.5, 0.0).unwrap()
            ))
        );
        assert_eq!(
            Sg2(c, g).intersects(&Sg2(a, e)),
            Some(IsxnResult::OneIntersection(
                Intersection::new(e, 0.5, 1.0).unwrap()
            ))
        );

        // // (s,s), (s,e), (e,s), (e,e)
        assert_eq!(
            Sg2(a, c).intersects(&Sg2(c, i)),
            Some(IsxnResult::OneIntersection(
                Intersection::new(c, 1.0, -0.0).unwrap()
            ))
        );
        assert_eq!(
            Sg2(a, c).intersects(&Sg2(i, c)),
            Some(IsxnResult::OneIntersection(
                Intersection::new(c, 1.0, 1.0).unwrap()
            ))
        );
        assert_eq!(
            Sg2(a, c).intersects(&Sg2(g, a)),
            Some(IsxnResult::OneIntersection(
                Intersection::new(a, 0.0, 1.0).unwrap()
            )),
        );
        assert_eq!(
            Sg2(a, c).intersects(&Sg2(a, g)),
            Some(IsxnResult::OneIntersection(
                Intersection::new(a, 0.0, -0.0).unwrap()
            ))
        );

        // // (w,w)
        assert_eq!(
            Sg2(a, i).intersects(&Sg2(c, g)),
            Some(IsxnResult::OneIntersection(
                Intersection::new(e, 0.5, 0.5).unwrap()
            ))
        );
    }

    #[test]
    fn test_abs() {
        assert_eq!(Sg2(p2!(0.0, 0.0), p2!(0.0, 1.0)).abs(), 1.0);
        assert_eq!(Sg2(p2!(0.0, 0.0), p2!(1.0, 1.0)).abs(), 2.0_f64.sqrt());
        assert_eq!(Sg2(p2!(1.0, 1.0), p2!(1.0, 1.0)).abs(), 0.0);
        assert_eq!(
            Sg2(p2!(-1.0, -1.0), p2!(1.0, 1.0)).abs(),
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
        let a = p2!(0.0, 2.0);
        let c = p2!(2.0, 2.0);

        assert_eq!(
            Sg2(a, c).line_segment_contains_pt(&a).unwrap(),
            Contains::AtStart
        );
    }
    #[test]
    fn test_segment() {
        assert_eq!(
            Sg2 {
                i: p2!(0, 0),
                f: p2!(0, 1)
            },
            Sg2(p2!(0, 0), p2!(0, 1))
        );
    }

    #[test]
    fn test_add() {
        assert_eq!(
            Sg2(p2!(0, 0), p2!(1, 1)) + p2!(1, 0),
            Sg2(p2!(1, 0), p2!(2, 1))
        );
    }

    #[test]
    fn test_add_assign() {
        let mut s = Sg2(p2!(0, 0), p2!(1, 1));
        s += p2!(1, 0);
        assert_eq!(s, Sg2(p2!(1, 0), p2!(2, 1)));
    }

    #[test]
    fn test_div() {
        assert_eq!(
            Sg2(p2!(0.0, 0.0), p2!(1.0, 1.0)) / 2.0,
            Sg2(p2!(0.0, 0.0), p2!(0.5, 0.5))
        );
    }

    #[test]
    fn test_div_assign() {
        let mut s = Sg2(p2!(0.0, 0.0), p2!(1.0, 1.0));
        s /= 2.0;
        assert_eq!(s, Sg2(p2!(0.0, 0.0), p2!(0.5, 0.5)));
    }

    #[test]
    fn test_mul() {
        assert_eq!(
            Sg2(p2!(0.0, 0.0), p2!(1.0, 1.0)) * 2.0,
            Sg2(p2!(0.0, 0.0), p2!(2.0, 2.0))
        );
    }

    #[test]
    fn test_mul_assign() {
        let mut s = Sg2(p2!(0.0, 0.0), p2!(1.0, 1.0));
        s *= 2.0;
        assert_eq!(s, Sg2(p2!(0.0, 0.0), p2!(2.0, 2.0)));
    }

    #[test]
    fn test_sub() {
        assert_eq!(
            Sg2(p2!(0.0, 0.0), p2!(1.0, 1.0)) - p2!(1.0, 2.0),
            // --------
            Sg2(p2!(-1.0, -2.0), p2!(0.0, -1.0))
        );
    }

    #[test]
    fn test_sub_assign() {
        let mut s = Sg2(p2!(0.0, 0.0), p2!(1.0, 1.0));
        s -= p2!(1.0, 2.0);
        assert_eq!(s, Sg2(p2!(-1.0, -2.0), p2!(0.0, -1.0)));
    }

    #[test]
    fn test_bounded_segment() {
        let s = Sg2(p2!(0, 1), p2!(1, 2));
        assert_eq!(s.bottom_bound(), 1.0);
        assert_eq!(s.top_bound(), 2.0);
        assert_eq!(s.left_bound(), 0.0);
        assert_eq!(s.right_bound(), 1.0);
        assert_eq!(s.bl_bound(), p2!(0, 1));
        assert_eq!(s.tl_bound(), p2!(0, 2));
        assert_eq!(s.br_bound(), p2!(1, 1));
        assert_eq!(s.tr_bound(), p2!(1, 2));
    }
}