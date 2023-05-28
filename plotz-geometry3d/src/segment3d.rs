//! A segment in 3d.

use float_ord::FloatOrd;
use {
    crate::{camera::Oblique, point3d::Pt3d},
    plotz_geometry::segment::Segment,
    std::{fmt::Debug, ops::*},
};

// A segment in 3d space, with initial and final points.
#[derive(Debug, Clone, Copy, Eq, Hash)]
pub struct Segment3d {
    pub i: Pt3d,
    pub f: Pt3d,
}

impl PartialEq for Segment3d {
    fn eq(&self, other: &Self) -> bool {
        self.i == other.i && self.f == other.f
    }
}

#[allow(non_snake_case)]
pub fn Segment3d(i: Pt3d, f: Pt3d) -> Segment3d {
    Segment3d { i, f }
}

impl Segment3d {
    // Returns the absolute value of the length of this segment.
    pub fn abs(&self) -> f64 {
        let two = 2_f64;
        (0_f64
            + (self.f.x.0 - self.i.x.0).powf(two)
            + (self.f.y.0 - self.i.y.0).powf(two)
            + (self.f.z.0 - self.i.z.0).powf(two))
        .sqrt()
    }

    pub fn project_oblique(&self, oblique_projection: &Oblique) -> Segment {
        Segment(
            oblique_projection.project(&self.i),
            oblique_projection.project(&self.f),
        )
    }

    // The average point of the polygon.
    pub fn average_pt(&self) -> Pt3d {
        self.i.avg(&self.f)
    }

    // The center of the object, projected along the view vector.
    pub fn dist_along(&self, view_vector: &Pt3d) -> f64 {
        self.average_pt().dot(view_vector)
    }
    // the maximum distance along a vector.
    pub fn max_dist_along(&self, view_vector: &Pt3d) -> f64 {
        [self.i, self.f]
            .iter()
            .map(|pt| FloatOrd(view_vector.dot(pt)))
            .max()
            .unwrap()
            .0
    }
    // the minimum distance along a vector.
    pub fn min_dist_along(&self, view_vector: &Pt3d) -> f64 {
        [self.i, self.f]
            .iter()
            .map(|pt| FloatOrd(view_vector.dot(pt)))
            .min()
            .unwrap()
            .0
    }
}

impl Add<Pt3d> for Segment3d {
    type Output = Segment3d;
    fn add(self, rhs: Pt3d) -> Self::Output {
        Segment3d(self.i + rhs, self.f + rhs)
    }
}
impl AddAssign<Pt3d> for Segment3d {
    fn add_assign(&mut self, rhs: Pt3d) {
        *self = Segment3d(self.i + rhs, self.f + rhs);
    }
}
impl Div<Pt3d> for Segment3d {
    type Output = Segment3d;
    fn div(self, rhs: Pt3d) -> Self::Output {
        Segment3d(self.i / rhs, self.f / rhs)
    }
}
impl Div<f64> for Segment3d {
    type Output = Segment3d;
    fn div(self, rhs: f64) -> Self::Output {
        Segment3d(self.i / rhs, self.f / rhs)
    }
}
impl DivAssign<Pt3d> for Segment3d {
    fn div_assign(&mut self, rhs: Pt3d) {
        *self = Segment3d(self.i / rhs, self.f / rhs);
    }
}
impl DivAssign<f64> for Segment3d {
    fn div_assign(&mut self, rhs: f64) {
        *self = Segment3d(self.i / rhs, self.f / rhs)
    }
}
impl Mul<Pt3d> for Segment3d {
    type Output = Segment3d;
    fn mul(self, rhs: Pt3d) -> Self::Output {
        Segment3d(self.i * rhs, self.f * rhs)
    }
}
impl Mul<f64> for Segment3d {
    type Output = Segment3d;
    fn mul(self, rhs: f64) -> Self::Output {
        Segment3d(self.i * rhs, self.f * rhs)
    }
}
impl MulAssign<Pt3d> for Segment3d {
    fn mul_assign(&mut self, rhs: Pt3d) {
        *self = Segment3d(self.i * rhs, self.f * rhs);
    }
}
impl MulAssign<f64> for Segment3d {
    fn mul_assign(&mut self, rhs: f64) {
        *self = Segment3d(self.i * rhs, self.f * rhs);
    }
}
impl Sub<Pt3d> for Segment3d {
    type Output = Segment3d;
    fn sub(self, rhs: Pt3d) -> Self::Output {
        Segment3d {
            i: self.i - rhs,
            f: self.f - rhs,
        }
    }
}
impl SubAssign<Pt3d> for Segment3d {
    fn sub_assign(&mut self, rhs: Pt3d) {
        *self = Segment3d(self.i - rhs, self.f - rhs);
    }
}
