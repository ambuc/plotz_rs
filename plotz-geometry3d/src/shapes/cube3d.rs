//! A cube in 3d.

use crate::{
    group3::Group3,
    shapes::{cuboid3d::Cuboid, pt3::Pt3},
};

// make a cube of faces (no edges)
#[allow(non_snake_case)]
pub fn Cube(root: Pt3, w: f64) -> Group3<()> {
    Cuboid(root, (w, w, w))
}
