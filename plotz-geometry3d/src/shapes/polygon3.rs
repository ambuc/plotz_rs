//! A polygon in 3d.

use crate::{
    bounded3::{streaming_bbox, Bounded3, Bounds3},
    obj3::ObjType3d,
    shapes::point3::Point3,
    Object, Rotatable,
};
use anyhow::Result;
use float_ord::FloatOrd;
use std::{fmt::Debug, ops::*};

use super::ray3::Ray3;

// A multiline is a list of points rendered with connecting line segments.
#[derive(Clone)]
pub struct Polygon3 {
    pub pts: Vec<Point3>,
}

impl Debug for Polygon3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Polygon3 { pts } = self;
        write!(f, "Polygon3d({:?})", pts)
    }
}

impl Polygon3 {
    // The average point of the polygon.
    pub fn average_pt(&self) -> Point3 {
        let num: f64 = self.pts.len() as f64;
        let sum_x: f64 = self.pts.iter().map(|pt| pt.x).sum();
        let sum_y: f64 = self.pts.iter().map(|pt| pt.y).sum();
        let sum_z: f64 = self.pts.iter().map(|pt| pt.z).sum();
        Point3(sum_x, sum_y, sum_z) / num
    }

    // The center of the object, projected along the view vector.
    pub fn dist_along(&self, view_vector: &Point3) -> f64 {
        view_vector.dot(&self.average_pt())
    }
    // the minimum distance along a vector.
    pub fn min_dist_along(&self, view_vector: &Point3) -> f64 {
        self.pts
            .iter()
            .map(|pt| FloatOrd(view_vector.dot(pt)))
            .min()
            .unwrap()
            .0
    }
    // the maximum distance along a vector.
    pub fn max_dist_along(&self, view_vector: &Point3) -> f64 {
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
pub fn Multiline3d(a: impl IntoIterator<Item = Point3>) -> Polygon3 {
    let pts: Vec<Point3> = a.into_iter().collect();
    assert_ne!(pts[0], pts[pts.len() - 1]);
    Polygon3 { pts }
}

// Constructor for polygons which are closed. The first and last points must be the same.
#[allow(non_snake_case)]
pub fn Polygon3(a: impl IntoIterator<Item = Point3>) -> Polygon3 {
    let pts: Vec<Point3> = a.into_iter().collect();
    assert_eq!(pts[0], pts[pts.len() - 1]);
    Polygon3 { pts }
}

plotz_geometry::ops_defaults_t!(Polygon3, Point3);

impl Rotatable for Polygon3 {
    fn rotate(&self, by: f64, about: Ray3) -> Result<Self> {
        let mut v = vec![];
        for p in self.pts.iter() {
            v.push(p.rotate(by, about)?);
        }
        Ok(Polygon3(v))
    }
}

impl Bounded3 for Polygon3 {
    fn bounds3(&self) -> Result<Bounds3> {
        streaming_bbox(self.pts.iter())
    }
}

impl Object for Polygon3 {
    fn objtype(&self) -> ObjType3d {
        ObjType3d::Polygon3d
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &Point3> + '_> {
        Box::new(self.pts.iter())
    }

    fn iter_mut(&mut self) -> Box<dyn Iterator<Item = &mut Point3> + '_> {
        Box::new(self.pts.iter_mut())
    }
}
