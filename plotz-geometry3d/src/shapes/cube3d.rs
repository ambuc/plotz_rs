//! A cube in 3d.

use crate::{
    group::Group,
    p3,
    shapes::{pg3::Pg3, pt3::Pt3},
};

fn make_planar_face(origin: Pt3, d1: Pt3, d2: Pt3) -> Pg3 {
    Pg3([origin, origin + d1, origin + d1 + d2, origin + d2, origin])
}

// make a cube of faces (no edges)
#[allow(non_snake_case)]
pub fn Cube(root: Pt3, (dx, dy, dz): (f64, f64, f64)) -> Group<Pg3> {
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
