//! An inner object.

use {
    crate::{
        camera::Oblique,
        shapes::{point3d::Pt3d, polygon3d::Polygon3d, segment3d::Segment3d},
    },
    derive_more::From,
    plotz_geometry::obj2::Obj2,
};

#[derive(Debug, Clone, From)]
pub enum Obj3 {
    Polygon3d(Polygon3d),
    Segment3d(Segment3d),
    // others?
}

impl Obj3 {
    pub fn project_oblique(&self, oblique_projection: &Oblique) -> Obj2 {
        match self {
            Obj3::Polygon3d(pg3d) => Obj2::Polygon(pg3d.project_oblique(oblique_projection)),
            Obj3::Segment3d(sg3d) => Obj2::Segment(sg3d.project_oblique(oblique_projection)),
        }
    }

    // The center of the object, projected along the view vector.
    pub fn dist_along(&self, view_vector: &Pt3d) -> f64 {
        match self {
            Obj3::Polygon3d(pg3d) => pg3d.dist_along(view_vector),
            Obj3::Segment3d(sg3d) => sg3d.dist_along(view_vector),
        }
    }
    // The maximum distance of the object, projected along the view vector.
    pub fn max_dist_along(&self, view_vector: &Pt3d) -> f64 {
        match self {
            Obj3::Polygon3d(pg3d) => pg3d.max_dist_along(view_vector),
            Obj3::Segment3d(sg3d) => sg3d.max_dist_along(view_vector),
        }
    }
    // The minimum distance of the object, projected along the view vector.
    pub fn min_dist_along(&self, view_vector: &Pt3d) -> f64 {
        match self {
            Obj3::Polygon3d(pg3d) => pg3d.min_dist_along(view_vector),
            Obj3::Segment3d(sg3d) => sg3d.min_dist_along(view_vector),
        }
    }
}
