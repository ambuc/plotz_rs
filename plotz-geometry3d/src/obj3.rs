//! An inner object.

use crate::{
    bounded3::{Bounded3, Bounds3},
    group3::Group3,
    shapes::{point3::Point3, polygon3::Pg3, ray3::Ray3, segment3::Sg3},
    Object, Rotatable,
};
use anyhow::Result;
use enum_dispatch::enum_dispatch;
use std::ops::*;

pub enum ObjType3d {
    Point3d,
    Segment3d,
    Polygon3d,
    Group3d,
}

#[derive(Debug, Clone)]
#[enum_dispatch]
pub enum Obj3 {
    Pg3(Pg3),
    Sg3(Sg3),
    Group3(Group3<()>),
    // others?
}

impl Obj3 {
    // The center of the object, projected along the view vector.
    pub fn dist_along(&self, view_vector: &Point3) -> f64 {
        match self {
            Obj3::Pg3(pg3d) => pg3d.dist_along(view_vector),
            Obj3::Sg3(sg3d) => sg3d.dist_along(view_vector),
            Obj3::Group3(_g3d) => unimplemented!("?"),
        }
    }
    // The maximum distance of the object, projected along the view vector.
    pub fn max_dist_along(&self, view_vector: &Point3) -> f64 {
        match self {
            Obj3::Pg3(pg3d) => pg3d.max_dist_along(view_vector),
            Obj3::Sg3(sg3d) => sg3d.max_dist_along(view_vector),
            Obj3::Group3(_g3d) => unimplemented!("?"),
        }
    }
    // The minimum distance of the object, projected along the view vector.
    pub fn min_dist_along(&self, view_vector: &Point3) -> f64 {
        match self {
            Obj3::Pg3(pg3d) => pg3d.min_dist_along(view_vector),
            Obj3::Sg3(sg3d) => sg3d.min_dist_along(view_vector),
            Obj3::Group3(_g3d) => unimplemented!("?"),
        }
    }
}

plotz_geometry::ops_defaults_t!(Obj3, Point3);

impl Rotatable for Obj3 {
    fn rotate(&self, by: f64, about: Ray3) -> Result<Self> {
        match self {
            Obj3::Pg3(pg3) => Ok(pg3.rotate(by, about)?.into()),
            Obj3::Sg3(_) => {
                // TODO(https://github.com/ambuc/plotz_rs/issues/5): Support sg3 rotation.
                todo!("sg rotate? See https://github.com/ambuc/plotz_rs/issues/5.")
            }
            Obj3::Group3(g3) => Ok(g3.rotate(by, about)?.into()),
        }
    }
}
