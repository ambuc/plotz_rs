//! An inner object.

use {
    crate::{
        camera::Oblique, face::Face, group::Group, polygon3d::Polygon3d, segment3d::Segment3d,
    },
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
    /// A group of like objects.
    GroupOfFaces(Group<Face>),
    // others?
}

impl ObjectInner {
    /// Project oblique.
    pub fn project_oblique(&self, oblique_projection: &Oblique) -> Vec<DrawObjInner> {
        match self {
            ObjectInner::Polygon3d(pg3d) => {
                vec![DrawObjInner::Polygon(
                    pg3d.project_oblique(oblique_projection),
                )]
            }
            ObjectInner::Segment3d(sg3d) => {
                vec![DrawObjInner::Segment(
                    sg3d.project_oblique(oblique_projection),
                )]
            }
            ObjectInner::GroupOfFaces(group_of_faces) => group_of_faces
                .items
                .iter()
                .map(|face| DrawObjInner::Polygon(face.project_oblique(oblique_projection)))
                .collect(),
        }
    }
}
