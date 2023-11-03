//! A 2D segment.
#![allow(missing_docs)]

use crate::{
    bounded::{Bounded, Bounds},
    crop::{CropType, Croppable, PointLoc},
    interpolate,
    interpolate::interpolate_2d_checked,
    intersection::{Intersection, IntersectionResult, MultipleIntersections},
    obj::{Obj, ObjType},
    shapes::{pg::Pg, pt::Pt, ry::Ry},
    style::Style,
    AnnotationSettings, Object, Roundable,
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

use super::txt::Txt;

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
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Sg {
    /// The initial point of the segment.
    pub i: Pt,
    /// The final point of the segment.
    pub f: Pt,
}

/// An alternate constructor for segments.
#[allow(non_snake_case)]
pub fn Sg(i: impl Into<Pt>, f: impl Into<Pt>) -> Sg {
    Sg {
        i: i.into(),
        f: f.into(),
    }
}

impl Sg {
    // Internal helper function; see https://www.geeksforgeeks.org/check-if-two-given-line-segments-intersect/.
    fn _ccw(&self, other: &Pt) -> Result<_Orientation> {
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
        let d2: f64 = Sg(self.i, *other).abs() + Sg(self.f, *other).abs();
        if approx_eq!(f64, d1, d2) {
            return Some(Contains::Within);
        }
        None
    }

    /// sometimes you just have to flip it.
    pub fn flip(&self) -> Sg {
        Sg {
            i: self.f,
            f: self.i,
        }
    }

    /// Returns true if one line segment intersects another.
    /// If two line segments share a point, returns false.
    /// If two line segments are parallel and overlapping, returns false.
    /// If two line segments are the same, returns false.
    pub fn intersects(&self, other: &Sg) -> Option<IntersectionResult> {
        if self == other {
            Some(IntersectionResult::MultipleIntersections(
                MultipleIntersections::LineSegmentsAreTheSame,
            ))
        } else if *self == Sg(other.f, other.i) {
            Some(IntersectionResult::MultipleIntersections(
                MultipleIntersections::LineSegmentsAreTheSameButReversed,
            ))
        } else if self.slope() == other.slope()
            && (self.f == other.i || other.f == self.i || self.i == other.i || self.f == other.f)
        {
            Some(IntersectionResult::MultipleIntersections(
                MultipleIntersections::LineSegmentsAreColinear,
            ))
        } else if let Some(pt) = self.get_line_intersection_inner(
            (self.i.x, self.i.y),
            (self.f.x, self.f.y),
            (other.i.x, other.i.y),
            (other.f.x, other.f.y),
        ) {
            Some(IntersectionResult::OneIntersection(Intersection::new(
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
        ((self.f.y - self.i.y).powf(two) + (self.f.x - self.i.x).powf(two)).sqrt()
    }

    /// Takes a lossy cross product of this with another segment (oriented tail-to-tail).
    pub fn cross_z(&self, other: &Sg) -> f64 {
        let d1 = self.f - self.i;
        let d2 = other.f - other.i;
        let x1 = d1.x;
        let x2 = d2.x;
        let y1 = d1.y;
        let y2 = d2.y;
        (x1 * y2) - (x2 * y1)
    }

    /// Midpoint of a segment.
    pub fn midpoint(&self) -> Pt {
        (self.i + self.f) / 2.0
    }

    /// Generates a ray perpendicular to this segment and emitting from its
    /// midpoint. One of the two angles, dunno which.
    pub fn ray_perpendicular(&self) -> Ry {
        Ry(self.midpoint(), self.ray_angle() + FRAC_PI_2)
    }

    /// Generates both perpendicular rays which emit from the midpoint of this
    /// segment.
    pub fn rays_perpendicular_both(&self) -> (Ry, Ry) {
        let ray = self.ray_perpendicular();
        (ray.clone().rotate(PI), ray)
    }
}

crate::ops_defaults_t!(Sg, Pt);

impl Bounded for Sg {
    fn bounds(&self) -> Result<Bounds> {
        Ok(Bounds {
            top_bound: std::cmp::max(FloatOrd(self.i.y), FloatOrd(self.f.y)).0,
            bottom_bound: std::cmp::min(FloatOrd(self.i.y), FloatOrd(self.f.y)).0,
            left_bound: std::cmp::min(FloatOrd(self.i.x), FloatOrd(self.f.x)).0,
            right_bound: std::cmp::max(FloatOrd(self.i.x), FloatOrd(self.f.x)).0,
        })
    }
}

impl Croppable for Sg {
    type Output = Sg;

    fn crop(&self, frame: &Pg, crop_type: CropType) -> Result<Vec<Self::Output>>
    where
        Self: Sized,
    {
        assert_eq!(crop_type, CropType::Inclusive);

        let frame_segments = frame.to_segments();
        let mut resultants: Vec<Sg> = vec![];
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
                    IntersectionResult::OneIntersection(isxn) => Some(isxn),
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

                    if !matches!(frame.contains_pt(&new_pt)?, PointLoc::Outside) && curr_pen_down {
                        resultants.push(Sg(curr_pt, new_pt));
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

    fn crop_excluding(&self, _other: &Pg) -> Result<Vec<Self::Output>>
    where
        Self: Sized,
    {
        unimplemented!("we haven't implemented segment crop excluding yet.");
    }
}

impl Roundable for Sg {
    fn round_to_nearest(&mut self, f: f64) {
        self.i.round_to_nearest(f);
        self.f.round_to_nearest(f);
    }
}

impl Object for Sg {
    fn is_empty(&self) -> bool {
        false
    }

    fn annotate(&self, settings: &AnnotationSettings) -> Vec<(Obj, Style)> {
        let mut a = vec![];

        let AnnotationSettings {
            font_size,
            precision,
        } = settings;
        for (_idx, pt) in self.iter().enumerate() {
            let x = format!("{:.1$}", pt.x, precision);
            let y = format!("{:.1$}", pt.y, precision);
            a.push((
                Txt {
                    pt: *pt,
                    inner: format!("({}, {})", x, y),
                    font_size: *font_size,
                }
                .into(),
                Style::default(),
            ));
        }

        a
    }

    fn objtype(&self) -> ObjType {
        ObjType::Segment
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &Pt> + '_> {
        Box::new(std::iter::once(&self.i).chain(std::iter::once(&self.f)))
    }

    fn iter_mut(&mut self) -> Box<dyn Iterator<Item = &mut Pt> + '_> {
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
        let a = Pt(0, 2);
        let b = Pt(1, 2);
        let c = Pt(2, 2);
        let d = Pt(0, 1);
        let e = Pt(1, 1);
        let f = Pt(2, 1);
        let g = Pt(0, 0);
        let h = Pt(1, 0);
        let i = Pt(2, 0);

        // m=0
        assert_eq!(Sg(g, h).slope(), 0.0);
        assert_eq!(Sg(g, i).slope(), 0.0);

        // m=0.5
        assert_eq!(Sg(g, f).slope(), 0.5);
        assert_eq!(Sg(d, c).slope(), 0.5);

        // m=1
        assert_eq!(Sg(g, e).slope(), 1.0);
        assert_eq!(Sg(g, c).slope(), 1.0);

        // m=2.0
        assert_eq!(Sg(h, c).slope(), 2.0);
        assert_eq!(Sg(g, b).slope(), 2.0);

        // m=inf
        assert_eq!(Sg(g, a).slope(), std::f64::INFINITY);
        assert_eq!(Sg(g, d).slope(), std::f64::INFINITY);

        // m=-0.5
        assert_eq!(Sg(a, f).slope(), -0.5);
        assert_eq!(Sg(d, i).slope(), -0.5);

        // m=-1.0
        assert_eq!(Sg(a, e).slope(), -1.0);
        assert_eq!(Sg(a, i).slope(), -1.0);

        // m=-2.0
        assert_eq!(Sg(b, i).slope(), -2.0);
        assert_eq!(Sg(a, h).slope(), -2.0);

        // m=-inf
        assert_eq!(Sg(a, g).slope(), -1.0 * std::f64::INFINITY);
        assert_eq!(Sg(d, g).slope(), -1.0 * std::f64::INFINITY);

        // slope is independent of start/end
        assert_eq!(Sg(a, c).slope(), Sg(c, a).slope());
        assert_eq!(Sg(a, f).slope(), Sg(f, a).slope());
        assert_eq!(Sg(a, i).slope(), Sg(i, a).slope());
        assert_eq!(Sg(a, h).slope(), Sg(h, a).slope());
    }

    #[test]
    fn test_rotate() {
        use float_eq::assert_float_eq;
        use std::f64::consts::PI;

        let origin = Pt(0, 0);

        //      ^
        //      |
        //      |  F
        // <----+--I->
        //      |
        //      |
        //      v
        let mut s = Sg((1, 0), (1, 0.5));

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
        let a = Pt(0, 2);
        let b = Pt(1, 2);
        assert!(Sg(a, b) == Sg(a, b));
        assert!(Sg(a, b) != Sg(b, a));
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
        let a = Pt(0, 2);
        let b = Pt(1, 2);
        let c = Pt(2, 2);
        let e = Pt(1, 1);
        let g = Pt(0, 0);
        let i = Pt(2, 0);

        // colinear
        assert_eq!(
            Sg(a, c).intersects(&Sg(a, c)),
            Some(IntersectionResult::MultipleIntersections(
                MultipleIntersections::LineSegmentsAreTheSame
            ))
        );
        assert_eq!(
            Sg(a, c).intersects(&Sg(c, a)),
            Some(IntersectionResult::MultipleIntersections(
                MultipleIntersections::LineSegmentsAreTheSameButReversed
            ))
        );
        // induce colinear
        assert_eq!(
            Sg(a, b).intersects(&Sg(b, c)),
            Some(IntersectionResult::MultipleIntersections(
                MultipleIntersections::LineSegmentsAreColinear
            ))
        );
        assert_eq!(
            Sg(a, b).intersects(&Sg(c, b)),
            Some(IntersectionResult::MultipleIntersections(
                MultipleIntersections::LineSegmentsAreColinear
            ))
        );
        assert_eq!(
            Sg(b, a).intersects(&Sg(b, c)),
            Some(IntersectionResult::MultipleIntersections(
                MultipleIntersections::LineSegmentsAreColinear
            ))
        );
        assert_eq!(
            Sg(b, a).intersects(&Sg(c, b)),
            Some(IntersectionResult::MultipleIntersections(
                MultipleIntersections::LineSegmentsAreColinear
            ))
        );

        // (s,w), (e,w), (w,s), (w,e)
        assert_eq!(
            Sg(e, i).intersects(&Sg(c, g)),
            Some(IntersectionResult::OneIntersection(
                Intersection::new(e, 0.0, 0.5).unwrap()
            ))
        );
        assert_eq!(
            Sg(a, e).intersects(&Sg(c, g)),
            Some(IntersectionResult::OneIntersection(
                Intersection::new(e, 1.0, 0.5).unwrap()
            ))
        );
        assert_eq!(
            Sg(c, g).intersects(&Sg(e, i)),
            Some(IntersectionResult::OneIntersection(
                Intersection::new(e, 0.5, 0.0).unwrap()
            ))
        );
        assert_eq!(
            Sg(c, g).intersects(&Sg(a, e)),
            Some(IntersectionResult::OneIntersection(
                Intersection::new(e, 0.5, 1.0).unwrap()
            ))
        );

        // // (s,s), (s,e), (e,s), (e,e)
        assert_eq!(
            Sg(a, c).intersects(&Sg(c, i)),
            Some(IntersectionResult::OneIntersection(
                Intersection::new(c, 1.0, -0.0).unwrap()
            ))
        );
        assert_eq!(
            Sg(a, c).intersects(&Sg(i, c)),
            Some(IntersectionResult::OneIntersection(
                Intersection::new(c, 1.0, 1.0).unwrap()
            ))
        );
        assert_eq!(
            Sg(a, c).intersects(&Sg(g, a)),
            Some(IntersectionResult::OneIntersection(
                Intersection::new(a, 0.0, 1.0).unwrap()
            )),
        );
        assert_eq!(
            Sg(a, c).intersects(&Sg(a, g)),
            Some(IntersectionResult::OneIntersection(
                Intersection::new(a, 0.0, -0.0).unwrap()
            ))
        );

        // // (w,w)
        assert_eq!(
            Sg(a, i).intersects(&Sg(c, g)),
            Some(IntersectionResult::OneIntersection(
                Intersection::new(e, 0.5, 0.5).unwrap()
            ))
        );
    }

    #[test]
    fn test_abs() {
        assert_eq!(Sg((0, 0), (0, 1)).abs(), 1.0);
        assert_eq!(Sg((0, 0), (1, 1)).abs(), 2.0_f64.sqrt());
        assert_eq!(Sg((1, 1), (1, 1)).abs(), 0.0);
        assert_eq!(Sg((-1, -1), (1, 1)).abs(), 2.0 * 2.0_f64.sqrt());
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
        let a = Pt(0, 2);
        let c = Pt(2, 2);

        assert_eq!(
            Sg(a, c).line_segment_contains_pt(&a).unwrap(),
            Contains::AtStart
        );
    }
    #[test]
    fn test_segment() {
        assert_eq!(
            Sg {
                i: Pt(0, 0),
                f: Pt(0, 1)
            },
            Sg((0, 0), (0, 1))
        );
    }

    #[test]
    fn test_add() {
        assert_eq!(Sg((0, 0), (1, 1)) + (1, 0), Sg((1, 0), (2, 1)));
    }

    #[test]
    fn test_add_assign() {
        let mut s = Sg((0, 0), (1, 1));
        s += (1, 0);
        assert_eq!(s, Sg((1, 0), (2, 1)));
    }

    #[test]
    fn test_div() {
        assert_eq!(Sg((0, 0), (1, 1)) / 2.0, Sg((0, 0), (0.5, 0.5)));
    }

    #[test]
    fn test_div_assign() {
        let mut s = Sg((0, 0), (1, 1));
        s /= 2.0;
        assert_eq!(s, Sg((0, 0), (0.5, 0.5)));
    }

    #[test]
    fn test_mul() {
        assert_eq!(Sg((0, 0), (1, 1)) * 2.0, Sg((0, 0), (2, 2)));
    }

    #[test]
    fn test_mul_assign() {
        let mut s = Sg((0, 0), (1, 1));
        s *= 2.0;
        assert_eq!(s, Sg((0, 0), (2, 2)));
    }

    #[test]
    fn test_sub() {
        assert_eq!(
            Sg((0, 0), (1, 1)) - (1, 2),
            // --------
            Sg((-1, -2), (0, -1))
        );
    }

    #[test]
    fn test_sub_assign() {
        let mut s = Sg((0, 0), (1, 1));
        s -= (1, 2);
        assert_eq!(s, Sg((-1, -2), (0, -1)));
    }

    #[test]
    fn test_bounded_segment() -> Result<()> {
        let s = Sg((0, 1), (1, 2));
        let b = s.bounds()?;
        assert_eq!(b.b(), 1.0);
        assert_eq!(b.t(), 2.0);
        assert_eq!(b.l(), 0.0);
        assert_eq!(b.r(), 1.0);
        assert_eq!(b.bl(), Pt(0, 1));
        assert_eq!(b.tl(), Pt(0, 2));
        assert_eq!(b.br(), Pt(1, 1));
        assert_eq!(b.tr(), Pt(1, 2));
        Ok(())
    }
}
