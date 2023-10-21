//! A cube in 3d.

use plotz_geometry::style::Style;

use crate::{
    group3::Group3,
    obj3::Obj3,
    shapes::{pg3::Pg3, pt3::Pt3},
};

fn make_planar_face(origin: Pt3, d1: Pt3, d2: Pt3) -> Pg3 {
    Pg3([origin, origin + d1, origin + d1 + d2, origin + d2, origin])
}

// make a cube of faces (no edges)
#[allow(non_snake_case)]
pub fn Cube(root: Pt3, (dx, dy, dz): (f64, f64, f64)) -> Group3<Style> {
    let dx = Pt3(dx, 0.0, 0.0);
    let dy = Pt3(0.0, dy, 0.0);
    let dz = Pt3(0.0, 0.0, dz);

    Group3::<Style>::new([
        (Obj3::Pg3(make_planar_face(root, dx, dy)), Style::default()),
        (Obj3::Pg3(make_planar_face(root, dx, dz)), Style::default()),
        (Obj3::Pg3(make_planar_face(root, dy, dz)), Style::default()),
        (
            Obj3::Pg3(make_planar_face(root + dx, dy, dz)),
            Style::default(),
        ),
        (
            Obj3::Pg3(make_planar_face(root + dy, dx, dz)),
            Style::default(),
        ),
        (
            Obj3::Pg3(make_planar_face(root + dz, dx, dy)),
            Style::default(),
        ),
    ])
}
