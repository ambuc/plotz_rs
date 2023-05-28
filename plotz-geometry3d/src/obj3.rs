//! An inner object.

use {
    crate::{
        camera::Oblique,
        shapes::{pg3::Pg3, pt3::Pt3, sg3::Sg3},
    },
    derive_more::From,
    plotz_geometry::obj2::Obj2,
};

#[derive(Debug, Clone, From)]
pub enum Obj3 {
    Pg3(Pg3),
    Sg3(Sg3),
    // others?
}

impl Obj3 {
    pub fn project_oblique(&self, oblique_projection: &Oblique) -> Obj2 {
        match self {
            Obj3::Pg3(pg3d) => Obj2::Pg2(pg3d.project_oblique(oblique_projection)),
            Obj3::Sg3(sg3d) => Obj2::Sg2(sg3d.project_oblique(oblique_projection)),
        }
    }

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
