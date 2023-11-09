//! A segment in 3d.

use crate::{
    bounded3::{streaming_bbox, Bounded3, Bounds3},
    obj3::ObjType3d,
    shapes::point3::Point3,
    Object,
};
use anyhow::Result;
use float_ord::FloatOrd;
use std::{fmt::Debug, ops::*};

// A segment in 3d space, with initial and final points.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Segment3 {
    pub i: Point3,
    pub f: Point3,
}

#[allow(non_snake_case)]
pub fn Segment3(i: Point3, f: Point3) -> Segment3 {
    Segment3 { i, f }
}

impl Segment3 {
    // Returns the absolute value of the length of this segment.
    pub fn abs(&self) -> f64 {
        let two = 2_f64;
        (0_f64
            + (self.f.x.0 - self.i.x.0).powf(two)
            + (self.f.y.0 - self.i.y.0).powf(two)
            + (self.f.z.0 - self.i.z.0).powf(two))
        .sqrt()
    }

    // The average point of the polygon.
    pub fn average_pt(&self) -> Point3 {
        self.i.avg(&self.f)
    }

    // The center of the object, projected along the view vector.
    pub fn dist_along(&self, view_vector: &Point3) -> f64 {
        self.average_pt().dot(view_vector)
    }
    // the maximum distance along a vector.
    pub fn max_dist_along(&self, view_vector: &Point3) -> f64 {
        [self.i, self.f]
            .iter()
            .map(|pt| FloatOrd(view_vector.dot(pt)))
            .max()
            .unwrap()
            .0
    }
    // the minimum distance along a vector.
    pub fn min_dist_along(&self, view_vector: &Point3) -> f64 {
        [self.i, self.f]
            .iter()
            .map(|pt| FloatOrd(view_vector.dot(pt)))
            .min()
            .unwrap()
            .0
    }
}

plotz_geometry::ops_defaults_t!(Segment3, Point3);

impl Bounded3 for Segment3 {
    fn bounds3(&self) -> Result<Bounds3> {
        streaming_bbox(self.iter())
    }
}

impl Object for Segment3 {
    fn objtype(&self) -> ObjType3d {
        ObjType3d::Segment3d
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &Point3> + '_> {
        Box::new(std::iter::once(&self.i).chain(std::iter::once(&self.f)))
    }

    fn iter_mut(&mut self) -> Box<dyn Iterator<Item = &mut Point3> + '_> {
        Box::new(std::iter::once(&mut self.i).chain(std::iter::once(&mut self.f)))
    }
}
