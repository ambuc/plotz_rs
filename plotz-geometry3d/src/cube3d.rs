//! A cube in 3d.

use crate::{face::Face, group::Group, p3, point3d::Pt3d, polygon3d::Polygon3d};

fn make_planar_face(origin: Pt3d, d1: Pt3d, d2: Pt3d) -> Face {
    Face::from(Polygon3d([
        origin,
        origin + d1,
        origin + d1 + d2,
        origin + d2,
        origin,
    ]))
}

/// make a cube of faces (no edges)
#[allow(non_snake_case)]
pub fn Cube(root: Pt3d, (dx, dy, dz): (f64, f64, f64)) -> Group<Face> {
    let dx = p3!(dx, 0.0, 0.0);
    let dy = p3!(0.0, dy, 0.0);
    let dz = p3!(0.0, 0.0, dz);

    Group(vec![
        make_planar_face(root, dx, dy),
        make_planar_face(root, dx, dz),
        make_planar_face(root, dy, dz),
        make_planar_face(root + dx, dy, dz),
        make_planar_face(root + dy, dx, dz),
        make_planar_face(root + dz, dx, dy),
    ])
}
