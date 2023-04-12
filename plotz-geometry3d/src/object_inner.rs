//! An inner object.

use crate::{group::Group, face::Face};

use {
    crate::{polygon3d::Polygon3d, segment3d::Segment3d},
    derive_more::From,
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
