//! A 2D segment.
#![allow(missing_docs)]

use crate::{
    bounded::{Bounded, Bounds},
    crop::{CropType, Croppable, PointLocation},
    interpolate,
    interpolate::interpolate_2d_checked,
    intersection::{Intersection, IntersectionResult},
    obj2::ObjType2d,
    shapes::{point::Point, polygon::Polygon, ray::Ray},
    Object,
};
use anyhow::{anyhow, Result};
use float_cmp::approx_eq;
use float_ord::FloatOrd;
use std::{
    cmp::PartialOrd,
    f64::consts::{FRAC_PI_2, PI},
    fmt::Debug,
    ops::*,
};

#[derive(Debug, PartialEq, Eq)]
enum _Orientation {
    _Colinear,
    _Clockwise,
    _CounterClockwise,
}

/// Whether a line segment contains a point, and if so where.
#[derive(Debug, PartialEq, Eq)]
pub enum SegmentContainsPoint {
    /// A line segment contains a point along it.
    Within,
    /// A line segment contains a point at its head.
    AtStart,
    /// A line segment contains a point at its tail.
    AtEnd,
}
/// A segment in 2D space, with initial and final points.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Segment {
    /// The initial point of the segment.
    pub i: Point,
    /// The final point of the segment.
    pub f: Point,
}

/// An alternate constructor for segments.
#[allow(non_snake_case)]
pub fn Segment(i: impl Into<Point>, f: impl Into<Point>) -> Segment {
    Segment {
        i: i.into(),
        f: f.into(),
    }
}

impl Segment {
    // Internal helper function; see https://www.geeksforgeeks.org/check-if-two-given-line-segments-intersect/.
    fn _ccw(&self, other: &Point) -> Result<_Orientation> {
        use std::cmp::Ordering;
        match PartialOrd::partial_cmp(
            &((other.y - self.i.y) * (self.f.x - self.i.x)
                - (self.f.y - self.i.y) * (other.x - self.i.x)),
            &0_f64,
        ) {
            Some(Ordering::Equal) => Ok(_Orientation::_Colinear),
            Some(Ordering::Greater) => Ok(_Orientation::_Clockwise),
            Some(Ordering::Less) => Ok(_Orientation::_CounterClockwise),
            None => Err(anyhow!("!")),
        }
    }

    /// The slope of a line segment.
    /// NB: this is the "elementary school math slope"; i.e. rise over run.
    /// Not the same as the angle of the ray.
    pub fn slope(&self) -> f64 {
        (self.f.y - self.i.y) / (self.f.x - self.i.x)
    }

    /// The angle from sg.i to sg.f, in radians.
    pub fn ray_angle(&self) -> f64 {
        self.i.angle_to(&self.f)
    }

    /// A rotation operation, for rotating a line segment about a point. Accepts
    /// a |by| argument in radians.
    pub fn rotate(&mut self, about: &Point, by: f64) {
        self.i.rotate_inplace(about, by);
        self.f.rotate_inplace(about, by);
    }

    /// Returns true if this line segment has point |other| along it.
    pub fn contains_point(&self, other: &Point) -> Option<SegmentContainsPoint> {
        if *other == self.i {
            return Some(SegmentContainsPoint::AtStart);
        }
        if *other == self.f {
            return Some(SegmentContainsPoint::AtEnd);
        }
        // if the len of the (i -> f) is the same as the len of (i -> pt -> f), then pt is on i->f.
        let d1: f64 = self.abs();
        let d2: f64 = Segment(self.i, *other).abs() + Segment(self.f, *other).abs();
        if approx_eq!(f64, d1, d2) {
            return Some(SegmentContainsPoint::Within);
        }
        None
    }

    /// sometimes you just have to flip it.
    pub fn flip(&self) -> Segment {
        Segment {
            i: self.f,
            f: self.i,
        }
    }

    /// Returns true if one line segment intersects another.
    /// If two line segments share a point, returns false.
    /// If two line segments are parallel and overlapping, returns false.
    /// If two line segments are the same, returns false.
    pub fn intersects(&self, other: &Segment) -> Option<IntersectionResult> {
        if self == other {
            Some(IntersectionResult::ErrSegmentsAreTheSame)
        } else if *self == Segment(other.f, other.i) {
            Some(IntersectionResult::ErrSegmentsAreTheSameButReversed)
        } else if self.slope() == other.slope()
            && (self.f == other.i || other.f == self.i || self.i == other.i || self.f == other.f)
        {
            Some(IntersectionResult::ErrSegmentsAreColinear)
        } else if let Some(pt) = self.get_line_intersection_inner(
            (self.i.x, self.i.y),
            (self.f.x, self.f.y),
            (other.i.x, other.i.y),
            (other.f.x, other.f.y),
        ) {
            Some(IntersectionResult::Ok(Intersection::new(
                pt,
                interpolate_2d_checked(self.i, self.f, pt).ok()?,
                interpolate_2d_checked(other.i, other.f, pt).ok()?,
            )?))
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
    ) -> Option<Point> {
        let s1_x = p1_x - p0_x;
        let s1_y = p1_y - p0_y;
        let s2_x = p3_x - p2_x;
        let s2_y = p3_y - p2_y;

        let s = (-s1_y * (p0_x - p2_x) + s1_x * (p0_y - p2_y)) / (-s2_x * s1_y + s1_x * s2_y);
        let t = (s2_x * (p0_y - p2_y) - s2_y * (p0_x - p2_x)) / (-s2_x * s1_y + s1_x * s2_y);

        if (0_f64..=1_f64).contains(&s) && (0_f64..=1_f64).contains(&t) {
            let i_x = p0_x + (t * s1_x);
            let i_y = p0_y + (t * s1_y);
            return Some(Point(i_x, i_y));
        }
        None
    }

    /// Returns the absolute value of the length of this segment.
    pub fn abs(&self) -> f64 {
        let two = 2_f64;
        ((self.f.y - self.i.y).powf(two) + (self.f.x - self.i.x).powf(two)).sqrt()
    }

    /// Takes a lossy cross product of this with another segment (oriented tail-to-tail).
    pub fn cross_z(&self, other: &Segment) -> f64 {
        let d1 = self.f - self.i;
        let d2 = other.f - other.i;
        let x1 = d1.x;
        let x2 = d2.x;
        let y1 = d1.y;
        let y2 = d2.y;
        (x1 * y2) - (x2 * y1)
    }

    /// Midpoint of a segment.
    pub fn midpoint(&self) -> Point {
        (self.i + self.f) / 2.0
    }

    /// Generates a ray perpendicular to this segment and emitting from its
    /// midpoint. One of the two angles, dunno which.
    pub fn ray_perpendicular(&self) -> Ray {
        Ray(self.midpoint(), self.ray_angle() + FRAC_PI_2)
    }

    /// Generates both perpendicular rays which emit from the midpoint of this
    /// segment.
    pub fn rays_perpendicular_both(&self) -> (Ray, Ray) {
        let ray = self.ray_perpendicular();
        (ray.clone().rotate(PI), ray)
    }
}

crate::ops_defaults_t!(Segment, Point);

impl Bounded for Segment {
    fn bounds(&self) -> Result<Bounds> {
        Ok(Bounds {
            y_max: std::cmp::max(FloatOrd(self.i.y), FloatOrd(self.f.y)).0,
            y_min: std::cmp::min(FloatOrd(self.i.y), FloatOrd(self.f.y)).0,
            x_min: std::cmp::min(FloatOrd(self.i.x), FloatOrd(self.f.x)).0,
            x_max: std::cmp::max(FloatOrd(self.i.x), FloatOrd(self.f.x)).0,
        })
    }
}

impl Croppable for Segment {
    type Output = Segment;

    fn crop(&self, frame: &Polygon, crop_type: CropType) -> Result<Vec<Self::Output>>
    where
        Self: Sized,
    {
        assert_eq!(crop_type, CropType::Inclusive);

        let frame_segments = frame.to_segments();
        let mut resultants: Vec<Segment> = vec![];
        let mut curr_pt = self.i;
        let mut curr_pen_down = !matches!(frame.contains_pt(&self.i)?, PointLocation::Outside);

        if let (PointLocation::Inside, PointLocation::Inside) =
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
                    IntersectionResult::Ok(isxn) => Some(isxn),
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
                        return Ok(resultants);
                    }

                    if !matches!(frame.contains_pt(&new_pt)?, PointLocation::Outside)
                        && curr_pen_down
                    {
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

    fn crop_excluding(&self, _other: &Polygon) -> Result<Vec<Self::Output>>
    where
        Self: Sized,
    {
        unimplemented!("we haven't implemented segment crop excluding yet.");
    }
}

impl Object for Segment {
    fn objtype(&self) -> ObjType2d {
        ObjType2d::Segment2d
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &Point> + '_> {
        Box::new(std::iter::once(&self.i).chain(std::iter::once(&self.f)))
    }

    fn iter_mut(&mut self) -> Box<dyn Iterator<Item = &mut Point> + '_> {
        Box::new(std::iter::once(&mut self.i).chain(std::iter::once(&mut self.f)))
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
        let a = Point(0, 2);
        let b = Point(1, 2);
        let c = Point(2, 2);
        let d = Point(0, 1);
        let e = Point(1, 1);
        let f = Point(2, 1);
        let g = Point(0, 0);
        let h = Point(1, 0);
        let i = Point(2, 0);

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

        let origin = Point(0, 0);

        //      ^
        //      |
        //      |  F
        // <----+--I->
        //      |
        //      |
        //      v
        let mut s = Segment((1, 0), (1, 0.5));

        s.rotate(/*about=*/ &origin, PI / 2.0);
        //      ^
        //     FI
        //      |
        // <----+---->
        //      |
        //      |
        //      v
        assert_float_eq!(s.i.x, 0.0, abs <= 0.000_1);
        assert_float_eq!(s.i.y, 1.0, abs <= 0.000_1);
        assert_float_eq!(s.f.x, -0.5, abs <= 0.000_1);
        assert_float_eq!(s.f.y, 1.0, abs <= 0.000_1);

        s.rotate(/*about=*/ &origin, PI / 2.0);
        //      ^
        //      |
        //      |
        // <-I--+---->
        //   F  |
        //      |
        //      v
        assert_float_eq!(s.i.x, -1.0, abs <= 0.000_1);
        assert_float_eq!(s.i.y, 0.0, abs <= 0.000_1);
        assert_float_eq!(s.f.x, -1.0, abs <= 0.000_1);
        assert_float_eq!(s.f.y, -0.5, abs <= 0.000_1);

        s.rotate(/*about=*/ &origin, PI / 2.0);
        //      ^
        //      |
        //      |
        // <----+---->
        //      |
        //      IF
        //      v
        assert_float_eq!(s.i.x, 0.0, abs <= 0.000_1);
        assert_float_eq!(s.i.y, -1.0, abs <= 0.000_1);
        assert_float_eq!(s.f.x, 0.5, abs <= 0.000_1);
        assert_float_eq!(s.f.y, -1.0, abs <= 0.000_1);

        s.rotate(/*about=*/ &origin, PI / 2.0);
        //      ^
        //      |
        //      |  F
        // <----+--I->
        //      |
        //      |
        //      v
        assert_float_eq!(s.i.x, 1.0, abs <= 0.000_1);
        assert_float_eq!(s.i.y, 0.0, abs <= 0.000_1);
        assert_float_eq!(s.f.x, 1.0, abs <= 0.000_1);
        assert_float_eq!(s.f.y, 0.5, abs <= 0.000_1);
    }

    #[test]
    fn test_equality() {
        let a = Point(0, 2);
        let b = Point(1, 2);
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
        let a = Point(0, 2);
        let b = Point(1, 2);
        let c = Point(2, 2);
        let e = Point(1, 1);
        let g = Point(0, 0);
        let i = Point(2, 0);

        // colinear
        assert_eq!(
            Segment(a, c).intersects(&Segment(a, c)),
            Some(IntersectionResult::ErrSegmentsAreTheSame)
        );
        assert_eq!(
            Segment(a, c).intersects(&Segment(c, a)),
            Some(IntersectionResult::ErrSegmentsAreTheSameButReversed)
        );
        // induce colinear
        assert_eq!(
            Segment(a, b).intersects(&Segment(b, c)),
            Some(IntersectionResult::ErrSegmentsAreColinear)
        );
        assert_eq!(
            Segment(a, b).intersects(&Segment(c, b)),
            Some(IntersectionResult::ErrSegmentsAreColinear)
        );
        assert_eq!(
            Segment(b, a).intersects(&Segment(b, c)),
            Some(IntersectionResult::ErrSegmentsAreColinear)
        );
        assert_eq!(
            Segment(b, a).intersects(&Segment(c, b)),
            Some(IntersectionResult::ErrSegmentsAreColinear)
        );

        // (s,w), (e,w), (w,s), (w,e)
        assert_eq!(
            Segment(e, i).intersects(&Segment(c, g)),
            Some(IntersectionResult::Ok(
                Intersection::new(e, 0.0, 0.5).unwrap()
            ))
        );
        assert_eq!(
            Segment(a, e).intersects(&Segment(c, g)),
            Some(IntersectionResult::Ok(
                Intersection::new(e, 1.0, 0.5).unwrap()
            ))
        );
        assert_eq!(
            Segment(c, g).intersects(&Segment(e, i)),
            Some(IntersectionResult::Ok(
                Intersection::new(e, 0.5, 0.0).unwrap()
            ))
        );
        assert_eq!(
            Segment(c, g).intersects(&Segment(a, e)),
            Some(IntersectionResult::Ok(
                Intersection::new(e, 0.5, 1.0).unwrap()
            ))
        );

        // // (s,s), (s,e), (e,s), (e,e)
        assert_eq!(
            Segment(a, c).intersects(&Segment(c, i)),
            Some(IntersectionResult::Ok(
                Intersection::new(c, 1.0, -0.0).unwrap()
            ))
        );
        assert_eq!(
            Segment(a, c).intersects(&Segment(i, c)),
            Some(IntersectionResult::Ok(
                Intersection::new(c, 1.0, 1.0).unwrap()
            ))
        );
        assert_eq!(
            Segment(a, c).intersects(&Segment(g, a)),
            Some(IntersectionResult::Ok(
                Intersection::new(a, 0.0, 1.0).unwrap()
            )),
        );
        assert_eq!(
            Segment(a, c).intersects(&Segment(a, g)),
            Some(IntersectionResult::Ok(
                Intersection::new(a, 0.0, -0.0).unwrap()
            ))
        );

        // // (w,w)
        assert_eq!(
            Segment(a, i).intersects(&Segment(c, g)),
            Some(IntersectionResult::Ok(
                Intersection::new(e, 0.5, 0.5).unwrap()
            ))
        );
    }

    #[test]
    fn test_abs() {
        assert_eq!(Segment((0, 0), (0, 1)).abs(), 1.0);
        assert_eq!(Segment((0, 0), (1, 1)).abs(), 2.0_f64.sqrt());
        assert_eq!(Segment((1, 1), (1, 1)).abs(), 0.0);
        assert_eq!(Segment((-1, -1), (1, 1)).abs(), 2.0 * 2.0_f64.sqrt());
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
        let a = Point(0, 2);
        let c = Point(2, 2);

        assert_eq!(
            Segment(a, c).contains_point(&a).unwrap(),
            SegmentContainsPoint::AtStart
        );
    }
    #[test]
    fn test_segment() {
        assert_eq!(
            Segment {
                i: Point(0, 0),
                f: Point(0, 1)
            },
            Segment((0, 0), (0, 1))
        );
    }

    #[test]
    fn test_add() {
        assert_eq!(Segment((0, 0), (1, 1)) + (1, 0), Segment((1, 0), (2, 1)));
    }

    #[test]
    fn test_add_assign() {
        let mut s = Segment((0, 0), (1, 1));
        s += (1, 0);
        assert_eq!(s, Segment((1, 0), (2, 1)));
    }

    #[test]
    fn test_div() {
        assert_eq!(Segment((0, 0), (1, 1)) / 2.0, Segment((0, 0), (0.5, 0.5)));
    }

    #[test]
    fn test_div_assign() {
        let mut s = Segment((0, 0), (1, 1));
        s /= 2.0;
        assert_eq!(s, Segment((0, 0), (0.5, 0.5)));
    }

    #[test]
    fn test_mul() {
        assert_eq!(Segment((0, 0), (1, 1)) * 2.0, Segment((0, 0), (2, 2)));
    }

    #[test]
    fn test_mul_assign() {
        let mut s = Segment((0, 0), (1, 1));
        s *= 2.0;
        assert_eq!(s, Segment((0, 0), (2, 2)));
    }

    #[test]
    fn test_sub() {
        assert_eq!(
            Segment((0, 0), (1, 1)) - (1, 2),
            // --------
            Segment((-1, -2), (0, -1))
        );
    }

    #[test]
    fn test_sub_assign() {
        let mut s = Segment((0, 0), (1, 1));
        s -= (1, 2);
        assert_eq!(s, Segment((-1, -2), (0, -1)));
    }

    #[test]
    fn test_bounded_segment() -> Result<()> {
        let s = Segment((0, 1), (1, 2));
        let b = s.bounds()?;
        assert_eq!(b.y_min, 1.0);
        assert_eq!(b.y_max, 2.0);
        assert_eq!(b.x_min, 0.0);
        assert_eq!(b.x_max, 1.0);
        assert_eq!(b.x_min_y_min(), Point(0, 1));
        assert_eq!(b.x_min_y_max(), Point(0, 2));
        assert_eq!(b.x_max_y_min(), Point(1, 1));
        assert_eq!(b.x_max_y_max(), Point(1, 2));
        Ok(())
    }
}
