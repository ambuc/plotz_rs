//! A polygon in 3d.

use crate::{
    bounded3::{streaming_bbox, Bounded3, Bounds3},
    shapes::pt3::Pt3,
    Rotatable,
};
use anyhow::Result;
use float_ord::FloatOrd;
use std::{fmt::Debug, ops::*};

use super::ry3::Ry3;

// A multiline is a list of points rendered with connecting line segments.
#[derive(Clone)]
pub struct Pg3 {
    pub pts: Vec<Pt3>,
}

impl Debug for Pg3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Pg3 { pts } = self;
        write!(f, "Polygon3d({:?})", pts)
    }
}

impl Pg3 {
    // The average point of the polygon.
    pub fn average_pt(&self) -> Pt3 {
        let num: f64 = self.pts.len() as f64;
        let sum_x: f64 = self.pts.iter().map(|pt| pt.x.0).sum();
        let sum_y: f64 = self.pts.iter().map(|pt| pt.y.0).sum();
        let sum_z: f64 = self.pts.iter().map(|pt| pt.z.0).sum();
        Pt3(sum_x, sum_y, sum_z) / num
    }

    // The center of the object, projected along the view vector.
    pub fn dist_along(&self, view_vector: &Pt3) -> f64 {
        view_vector.dot(&self.average_pt())
    }
    // the minimum distance along a vector.
    pub fn min_dist_along(&self, view_vector: &Pt3) -> f64 {
        self.pts
            .iter()
            .map(|pt| FloatOrd(view_vector.dot(pt)))
            .min()
            .unwrap()
            .0
    }
    // the maximum distance along a vector.
    pub fn max_dist_along(&self, view_vector: &Pt3) -> f64 {
        self.pts
            .iter()
            .map(|pt| FloatOrd(view_vector.dot(pt)))
            .max()
            .unwrap()
            .0
    }
}

// Constructor for multilines, which are by definition open. The first and last
// points must not be the same.
#[allow(non_snake_case)]
pub fn Multiline3d(a: impl IntoIterator<Item = Pt3>) -> Pg3 {
    let pts: Vec<Pt3> = a.into_iter().collect();
    assert_ne!(pts[0], pts[pts.len() - 1]);
    Pg3 { pts }
}

// Constructor for polygons which are closed. The first and last points must be the same.
#[allow(non_snake_case)]
pub fn Pg3(a: impl IntoIterator<Item = Pt3>) -> Pg3 {
    let pts: Vec<Pt3> = a.into_iter().collect();
    assert_eq!(pts[0], pts[pts.len() - 1]);
    Pg3 { pts }
}

impl Add<Pt3> for &Pg3 {
    type Output = Pg3;
    fn add(self, rhs: Pt3) -> Self::Output {
        Pg3 {
            pts: self.pts.iter().map(|p| *p + rhs).collect(),
        }
    }
}
impl Add<Pt3> for Pg3 {
    type Output = Pg3;
    fn add(self, rhs: Pt3) -> Self::Output {
        &self + rhs
    }
}
impl AddAssign<Pt3> for Pg3 {
    fn add_assign(&mut self, rhs: Pt3) {
        self.pts.iter_mut().for_each(|p| *p += rhs);
    }
}
impl Div<Pt3> for Pg3 {
    type Output = Pg3;
    fn div(self, rhs: Pt3) -> Self::Output {
        Pg3 {
            pts: self.pts.iter().map(|p| *p / rhs).collect(),
        }
    }
}
impl Div<f64> for Pg3 {
    type Output = Pg3;
    fn div(self, rhs: f64) -> Self::Output {
        Pg3 {
            pts: self.pts.iter().map(|p| *p / rhs).collect(),
        }
    }
}
impl DivAssign<Pt3> for Pg3 {
    fn div_assign(&mut self, rhs: Pt3) {
        self.pts.iter_mut().for_each(|p| *p /= rhs);
    }
}
impl DivAssign<f64> for Pg3 {
    fn div_assign(&mut self, rhs: f64) {
        self.pts.iter_mut().for_each(|p| *p /= rhs);
    }
}
impl Mul<Pt3> for Pg3 {
    type Output = Pg3;
    fn mul(self, rhs: Pt3) -> Pg3 {
        Pg3 {
            pts: self.pts.iter().map(|p| *p * rhs).collect(),
        }
    }
}
impl Mul<f64> for Pg3 {
    type Output = Pg3;
    fn mul(mut self, rhs: f64) -> Pg3 {
        self *= rhs;
        self
    }
}
impl MulAssign<Pt3> for Pg3 {
    fn mul_assign(&mut self, rhs: Pt3) {
        self.pts.iter_mut().for_each(|p| *p *= rhs);
    }
}
impl MulAssign<f64> for Pg3 {
    fn mul_assign(&mut self, rhs: f64) {
        self.pts.iter_mut().for_each(|p| *p *= rhs);
    }
}
impl Sub<Pt3> for &Pg3 {
    type Output = Pg3;
    fn sub(self, rhs: Pt3) -> Self::Output {
        Pg3 {
            pts: self.pts.iter().map(|p| *p - rhs).collect(),
        }
    }
}
impl Sub<Pt3> for Pg3 {
    type Output = Pg3;
    fn sub(self, rhs: Pt3) -> Self::Output {
        Pg3 {
            pts: self.pts.iter().map(|p| *p - rhs).collect(),
        }
    }
}
impl SubAssign<Pt3> for Pg3 {
    fn sub_assign(&mut self, rhs: Pt3) {
        self.pts.iter_mut().for_each(|p| *p -= rhs);
    }
}

impl Rotatable for Pg3 {
    fn rotate(&self, by: f64, about: Ry3) -> Result<Self> {
        let mut v = vec![];
        for p in self.pts.iter() {
            v.push(p.rotate(by, about)?);
        }
        Ok(Pg3(v))
    }
}

impl Bounded3 for Pg3 {
    fn bounds3(&self) -> Result<Bounds3> {
        streaming_bbox(self.pts.iter())
    }
}
