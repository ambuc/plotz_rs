//! A cube in 3d.

use crate::{
    group3::Group3,
    shapes::{cuboid3d::Cuboid, point3::Point3},
};

// make a cube of faces (no edges)
#[allow(non_snake_case)]
pub fn Cube(root: Point3, w: f64) -> Group3<()> {
    Cuboid(root, (w, w, w))
}
