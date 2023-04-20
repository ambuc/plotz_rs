//! A polygon in 3d.

use std::fmt::Debug;

use float_ord::FloatOrd;
use plotz_geometry::polygon::TryPolygon;

use {
    crate::{camera::Oblique, point3d::Pt3d},
    plotz_geometry::polygon::Polygon,
    std::ops::*,
};

/// A multiline is a list of points rendered with connecting line segments.
#[derive(Clone)]
pub struct Polygon3d {
    /// The points which describe a polygon or multiline.
    pub pts: Vec<Pt3d>,
}

impl Debug for Polygon3d {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Polygon3d { pts } = self;
        write!(f, "Polygon3d({:?})", pts)
    }
}

impl Polygon3d {
    /// Project oblique
    pub fn project_oblique(&self, oblique_projection: &Oblique) -> Polygon {
        Polygon(
            self.pts
                .iter()
                .map(|pt3d| oblique_projection.project(&pt3d))
                .collect::<Vec<_>>(),
        )
    }

    /// The average point of the polygon.
    pub fn average_pt(&self) -> Pt3d {
        let num: f64 = self.pts.len() as f64;
        let sum_x: f64 = self.pts.iter().map(|pt| pt.x.0).sum();
        let sum_y: f64 = self.pts.iter().map(|pt| pt.y.0).sum();
        let sum_z: f64 = self.pts.iter().map(|pt| pt.z.0).sum();
        Pt3d(sum_x / num, sum_y / num, sum_z / num)
    }

    /// The center of the object, projected along the view vector.
    pub fn dist_along(&self, view_vector: &Pt3d) -> f64 {
        view_vector.dot(&self.average_pt())
    }
    /// the minimum distance along a vector.
    pub fn min_dist_along(&self, view_vector: &Pt3d) -> f64 {
        self.pts
            .iter()
            .map(|pt| FloatOrd(view_vector.dot(&pt)))
            .min()
            .unwrap()
            .0
    }
    /// the maximum distance along a vector.
    pub fn max_dist_along(&self, view_vector: &Pt3d) -> f64 {
        self.pts
            .iter()
            .map(|pt| FloatOrd(view_vector.dot(&pt)))
            .max()
            .unwrap()
            .0
    }
}

/// Constructor for multilines, which are by definition open. The first and last
/// points must not be the same.
#[allow(non_snake_case)]
pub fn Multiline3d(a: impl IntoIterator<Item = Pt3d>) -> Polygon3d {
    let pts: Vec<Pt3d> = a.into_iter().collect();
    assert_ne!(pts[0], pts[pts.len() - 1]);
    Polygon3d { pts }
}

/// Constructor for polygons which are closed. The first and last points must be the same.
#[allow(non_snake_case)]
pub fn Polygon3d(a: impl IntoIterator<Item = Pt3d>) -> Polygon3d {
    let pts: Vec<Pt3d> = a.into_iter().collect();
    assert_eq!(pts[0], pts[pts.len() - 1]);
    Polygon3d { pts }
}

impl Add<Pt3d> for &Polygon3d {
    type Output = Polygon3d;
    fn add(self, rhs: Pt3d) -> Self::Output {
        Polygon3d {
            pts: self.pts.iter().map(|p| *p + rhs).collect(),
            ..self.clone()
        }
    }
}
impl Add<Pt3d> for Polygon3d {
    type Output = Polygon3d;
    fn add(self, rhs: Pt3d) -> Self::Output {
        &self + rhs
    }
}
impl AddAssign<Pt3d> for Polygon3d {
    fn add_assign(&mut self, rhs: Pt3d) {
        self.pts.iter_mut().for_each(|p| *p += rhs);
    }
}
impl Div<Pt3d> for Polygon3d {
    type Output = Polygon3d;
    fn div(self, rhs: Pt3d) -> Self::Output {
        Polygon3d {
            pts: self.pts.iter().map(|p| *p / rhs).collect(),
            ..self
        }
    }
}
impl Div<f64> for Polygon3d {
    type Output = Polygon3d;
    fn div(self, rhs: f64) -> Self::Output {
        Polygon3d {
            pts: self.pts.iter().map(|p| *p / rhs).collect(),
            ..self
        }
    }
}
impl DivAssign<Pt3d> for Polygon3d {
    fn div_assign(&mut self, rhs: Pt3d) {
        self.pts.iter_mut().for_each(|p| *p /= rhs);
    }
}
impl DivAssign<f64> for Polygon3d {
    fn div_assign(&mut self, rhs: f64) {
        self.pts.iter_mut().for_each(|p| *p /= rhs);
    }
}
impl Mul<Pt3d> for Polygon3d {
    type Output = Polygon3d;
    fn mul(self, rhs: Pt3d) -> Polygon3d {
        Polygon3d {
            pts: self.pts.iter().map(|p| *p * rhs).collect(),
            ..self
        }
    }
}
impl Mul<f64> for Polygon3d {
    type Output = Polygon3d;
    fn mul(mut self, rhs: f64) -> Polygon3d {
        self *= rhs;
        self
    }
}
impl MulAssign<Pt3d> for Polygon3d {
    fn mul_assign(&mut self, rhs: Pt3d) {
        self.pts.iter_mut().for_each(|p| *p *= rhs);
    }
}
impl MulAssign<f64> for Polygon3d {
    fn mul_assign(&mut self, rhs: f64) {
        self.pts.iter_mut().for_each(|p| *p *= rhs);
    }
}
impl Sub<Pt3d> for &Polygon3d {
    type Output = Polygon3d;
    fn sub(self, rhs: Pt3d) -> Self::Output {
        Polygon3d {
            pts: self.pts.iter().map(|p| *p - rhs).collect(),
            ..self.clone()
        }
    }
}
impl Sub<Pt3d> for Polygon3d {
    type Output = Polygon3d;
    fn sub(self, rhs: Pt3d) -> Self::Output {
        Polygon3d {
            pts: self.pts.iter().map(|p| *p - rhs).collect(),
            ..self
        }
    }
}
impl SubAssign<Pt3d> for Polygon3d {
    fn sub_assign(&mut self, rhs: Pt3d) {
        self.pts.iter_mut().for_each(|p| *p -= rhs);
    }
}
