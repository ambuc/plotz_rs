//! A cuboid in 3d.

use crate::{
    group3::Group3,
    obj3::Obj3,
    shapes::{point3::Pt3, polygon3::Pg3},
};

fn make_planar_face(origin: Pt3, d1: Pt3, d2: Pt3) -> Pg3 {
    Pg3([origin, origin + d1, origin + d1 + d2, origin + d2, origin])
}

// make a cube of faces (no edges)
#[allow(non_snake_case)]
pub fn Cuboid<T1, T2, T3>(root: impl Into<Pt3>, (dx, dy, dz): (T1, T2, T3)) -> Group3<()>
where
    f64: From<T1>,
    f64: From<T2>,
    f64: From<T3>,
{
    let root: Pt3 = root.into();
    let dx: f64 = dx.into();
    let dy: f64 = dy.into();
    let dz: f64 = dz.into();

    let dx: Pt3 = Pt3(dx, 0.0, 0.0);
    let dy: Pt3 = Pt3(0.0, dy, 0.0);
    let dz: Pt3 = Pt3(0.0, 0.0, dz);

    Group3::<()>::new([
        (Obj3::Pg3(make_planar_face(root, dx, dy)), ()),
        (Obj3::Pg3(make_planar_face(root, dx, dz)), ()),
        (Obj3::Pg3(make_planar_face(root, dy, dz)), ()),
        (Obj3::Pg3(make_planar_face(root + dx, dy, dz)), ()),
        (Obj3::Pg3(make_planar_face(root + dy, dx, dz)), ()),
        (Obj3::Pg3(make_planar_face(root + dz, dx, dy)), ()),
    ])
}
