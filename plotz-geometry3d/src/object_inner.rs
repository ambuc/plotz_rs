//! An inner object.

use crate::point3d::Pt3d;

use {
    crate::{camera::Oblique, face::Face, polygon3d::Polygon3d, segment3d::Segment3d},
    derive_more::From,
    plotz_geometry::draw_obj_inner::DrawObjInner,
};

/// Some 3d object which can be projected.
#[derive(Debug, Clone, From)]
pub enum ObjectInner {
    /// A 3d polygon.
    Polygon3d(Polygon3d),
    /// A 3d line segment.
    Segment3d(Segment3d),
    /// A face.
    Face(Face),
    // others?
}

impl ObjectInner {
    /// Project oblique.
    pub fn project_oblique(&self, oblique_projection: &Oblique) -> DrawObjInner {
        match self {
            ObjectInner::Polygon3d(pg3d) => {
                DrawObjInner::Polygon(pg3d.project_oblique(oblique_projection))
            }
            ObjectInner::Segment3d(sg3d) => {
                DrawObjInner::Segment(sg3d.project_oblique(oblique_projection))
            }
            ObjectInner::Face(face) => {
                DrawObjInner::Polygon(face.pg3d.project_oblique(oblique_projection))
            }
        }
    }

    /// The center of the object, projected along the view vector.
    pub fn dist_along(&self, view_vector: &Pt3d) -> f64 {
        match self {
            ObjectInner::Polygon3d(pg3d) => pg3d.dist_along(view_vector),
            ObjectInner::Segment3d(sg3d) => sg3d.dist_along(view_vector),
            ObjectInner::Face(face) => face.pg3d.dist_along(view_vector),
        }
    }
}
