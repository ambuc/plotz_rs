//! A 2D segment.
#![allow(missing_docs)]

use crate::{
    bounded::{Bounded, Bounds},
    crop::{CropType, Croppable, PointLocation},
    interpolate,
    intersection::{Intersection, IntersectionResult},
    intersects::{
        opinion::{Opinion, SegmentOpinion},
        segment_intersects_segment,
        specialcase::{General, TwoSegments},
        Isxn,
    },
    obj2::ObjType2d,
    shapes::{point::Point, polygon::Polygon, ray::Ray},
    Object,
};
use anyhow::Result;
use float_ord::FloatOrd;
use std::{
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

#[derive(Debug, PartialEq, Eq)]
pub enum SegmentContainsPoint {
    Within,
    AtStart,
    AtEnd,
}
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Segment {
    pub i: Point,
    pub f: Point,
}

#[allow(non_snake_case)]
pub fn Segment(i: impl Into<Point>, f: impl Into<Point>) -> Segment {
    Segment {
        i: i.into(),
        f: f.into(),
    }
}

impl Segment {
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

    pub fn rotate(&mut self, about: &Point, by: f64) {
        self.i.rotate_inplace(about, by);
        self.f.rotate_inplace(about, by);
    }

    pub fn flip(&self) -> Segment {
        Segment {
            i: self.f,
            f: self.i,
        }
    }

    pub fn intersects(&self, other: &Segment) -> Option<IntersectionResult> {
        // TODO(ambuc): remove Segment::intersects entirely.
        match segment_intersects_segment(self, other) {
            Err(_) => None,
            Ok(Isxn::None) => None,
            Ok(Isxn::SpecialCase(General::TwoSegments(case))) => Some(match case {
                TwoSegments::Same => IntersectionResult::ErrSegmentsAreTheSame,
                TwoSegments::SameButReversed => {
                    IntersectionResult::ErrSegmentsAreTheSameButReversed
                }
                TwoSegments::Colinear => IntersectionResult::ErrSegmentsAreColinear,
            }),
            Ok(Isxn::SpecialCase(_)) => panic!("?"),
            Ok(Isxn::Some(Opinion::Segment(sg_ops_a), Opinion::Segment(sg_ops_b))) => {
                assert_eq!(sg_ops_a.len(), 1);
                assert_eq!(sg_ops_b.len(), 1);

                match (sg_ops_a.head, sg_ops_b.head) {
                    (
                        SegmentOpinion::AlongSegment {
                            at_point: pt,
                            percent_along: a_pct,
                        },
                        SegmentOpinion::AlongSegment {
                            percent_along: b_pct,
                            ..
                        },
                    ) => Some(IntersectionResult::Ok(Intersection { pt, a_pct, b_pct })),
                }
            }
            _ => None,
        }
    }

    pub fn length(&self) -> f64 {
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
                    <= interpolate::interpolate_2d_checked(self.i, self.f, curr_pt)
                        .unwrap_or_else(|_| {
                            panic!(
                                "interpolate failed: a: {:?}, b: {:?}, i: {:?}",
                                self.i, self.f, curr_pt,
                            )
                        })
                        .as_f64()
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
    fn test_abs() {
        assert_eq!(Segment((0, 0), (0, 1)).length(), 1.0);
        assert_eq!(Segment((0, 0), (1, 1)).length(), 2.0_f64.sqrt());
        assert_eq!(Segment((1, 1), (1, 1)).length(), 0.0);
        assert_eq!(Segment((-1, -1), (1, 1)).length(), 2.0 * 2.0_f64.sqrt());
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
