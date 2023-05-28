//! An inner object.

use {
    crate::{
        camera::Oblique,
        shapes::{face::Face, point3d::Pt3d, polygon3d::Polygon3d, segment3d::Segment3d},
    },
    derive_more::From,
    plotz_geometry::object2d_inner::Object2dInner,
};

#[derive(Debug, Clone, From)]
pub enum Object3dInner {
    Polygon3d(Polygon3d),
    Segment3d(Segment3d),
    Face(Face),
    // others?
}

impl Object3dInner {
    pub fn project_oblique(&self, oblique_projection: &Oblique) -> Object2dInner {
        match self {
            Object3dInner::Polygon3d(pg3d) => {
                Object2dInner::Polygon(pg3d.project_oblique(oblique_projection))
            }
            Object3dInner::Segment3d(sg3d) => {
                Object2dInner::Segment(sg3d.project_oblique(oblique_projection))
            }
            Object3dInner::Face(face) => {
                Object2dInner::Polygon(face.pg3d.project_oblique(oblique_projection))
            }
        }
    }

    // The center of the object, projected along the view vector.
    pub fn dist_along(&self, view_vector: &Pt3d) -> f64 {
        match self {
            Object3dInner::Polygon3d(pg3d) => pg3d.dist_along(view_vector),
            Object3dInner::Segment3d(sg3d) => sg3d.dist_along(view_vector),
            Object3dInner::Face(face) => face.pg3d.dist_along(view_vector),
        }
    }
    // The maximum distance of the object, projected along the view vector.
    pub fn max_dist_along(&self, view_vector: &Pt3d) -> f64 {
        match self {
            Object3dInner::Polygon3d(pg3d) => pg3d.max_dist_along(view_vector),
            Object3dInner::Segment3d(sg3d) => sg3d.max_dist_along(view_vector),
            Object3dInner::Face(face) => face.pg3d.max_dist_along(view_vector),
        }
    }
    // The minimum distance of the object, projected along the view vector.
    pub fn min_dist_along(&self, view_vector: &Pt3d) -> f64 {
        match self {
            Object3dInner::Polygon3d(pg3d) => pg3d.min_dist_along(view_vector),
            Object3dInner::Segment3d(sg3d) => sg3d.min_dist_along(view_vector),
            Object3dInner::Face(face) => face.pg3d.min_dist_along(view_vector),
        }
    }
}
