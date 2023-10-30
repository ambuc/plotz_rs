//! An inner object.

use crate::{
    bounded3::{Bounded3, Bounds3},
    group3::Group3,
    shapes::{pg3::Pg3, pt3::Pt3, ry3::Ry3, sg3::Sg3},
    Rotatable,
};
use anyhow::Result;
use derive_more::From;
use std::ops::*;

#[derive(Debug, Clone, From)]
pub enum Obj3 {
    Pg3(Pg3),
    Sg3(Sg3),
    Group3(Group3<()>),
    // others?
}

impl Obj3 {
    // The center of the object, projected along the view vector.
    pub fn dist_along(&self, view_vector: &Pt3) -> f64 {
        match self {
            Obj3::Pg3(pg3d) => pg3d.dist_along(view_vector),
            Obj3::Sg3(sg3d) => sg3d.dist_along(view_vector),
            Obj3::Group3(_g3d) => unimplemented!("?"),
        }
    }
    // The maximum distance of the object, projected along the view vector.
    pub fn max_dist_along(&self, view_vector: &Pt3) -> f64 {
        match self {
            Obj3::Pg3(pg3d) => pg3d.max_dist_along(view_vector),
            Obj3::Sg3(sg3d) => sg3d.max_dist_along(view_vector),
            Obj3::Group3(_g3d) => unimplemented!("?"),
        }
    }
    // The minimum distance of the object, projected along the view vector.
    pub fn min_dist_along(&self, view_vector: &Pt3) -> f64 {
        match self {
            Obj3::Pg3(pg3d) => pg3d.min_dist_along(view_vector),
            Obj3::Sg3(sg3d) => sg3d.min_dist_along(view_vector),
            Obj3::Group3(_g3d) => unimplemented!("?"),
        }
    }
}

impl Obj3 {
    pub fn iter(&self) -> Box<dyn Iterator<Item = &Pt3> + '_> {
        match self {
            Obj3::Pg3(pg3) => Box::new(pg3.iter()),
            Obj3::Sg3(sg3) => Box::new(sg3.iter()),
            Obj3::Group3(g3) => Box::new(g3.iter()),
        }
    }
}

impl<T> Add<T> for Obj3
where
    T: Into<Pt3>,
{
    type Output = Obj3;
    fn add(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        match self {
            Obj3::Pg3(pg) => Obj3::from(pg + rhs),
            Obj3::Sg3(sg) => Obj3::from(sg + rhs),
            Obj3::Group3(g) => Obj3::from(g + rhs),
        }
    }
}

impl<T> AddAssign<T> for Obj3
where
    T: Into<Pt3>,
{
    fn add_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        match self {
            Obj3::Pg3(p) => {
                *p += rhs;
            }
            Obj3::Sg3(sg) => {
                *sg += rhs;
            }
            Obj3::Group3(g) => {
                *g += rhs;
            }
        }
    }
}

impl Rotatable for Obj3 {
    fn rotate(&self, by: f64, about: Ry3) -> Result<Self> {
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

impl Bounded3 for Obj3 {
    fn bounds3(&self) -> Result<Bounds3> {
        match self {
            Obj3::Pg3(pg3) => pg3.bounds3(),
            Obj3::Sg3(sg3) => sg3.bounds3(),
            Obj3::Group3(g3) => g3.bounds3(),
        }
    }
}
