//! An inner object.

use crate::shapes::{pg3::Pg3, pt3::Pt3, sg3::Sg3};
use derive_more::From;
use std::ops::*;

#[derive(Debug, Clone, From)]
pub enum Obj3 {
    Pg3(Pg3),
    Sg3(Sg3),
    // others?
}

impl Obj3 {
    // The center of the object, projected along the view vector.
    pub fn dist_along(&self, view_vector: &Pt3) -> f64 {
        match self {
            Obj3::Pg3(pg3d) => pg3d.dist_along(view_vector),
            Obj3::Sg3(sg3d) => sg3d.dist_along(view_vector),
        }
    }
    // The maximum distance of the object, projected along the view vector.
    pub fn max_dist_along(&self, view_vector: &Pt3) -> f64 {
        match self {
            Obj3::Pg3(pg3d) => pg3d.max_dist_along(view_vector),
            Obj3::Sg3(sg3d) => sg3d.max_dist_along(view_vector),
        }
    }
    // The minimum distance of the object, projected along the view vector.
    pub fn min_dist_along(&self, view_vector: &Pt3) -> f64 {
        match self {
            Obj3::Pg3(pg3d) => pg3d.min_dist_along(view_vector),
            Obj3::Sg3(sg3d) => sg3d.min_dist_along(view_vector),
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
        }
    }
}
