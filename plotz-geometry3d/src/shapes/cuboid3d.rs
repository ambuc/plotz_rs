//! A cuboid in 3d.

use crate::{
    group3::Group3,
    obj3::Obj3,
    shapes::{point3::Point3, polygon3::Polygon3},
};

fn make_planar_face(origin: Point3, d1: Point3, d2: Point3) -> Polygon3 {
    Polygon3([origin, origin + d1, origin + d1 + d2, origin + d2, origin])
}

// make a cube of faces (no edges)
#[allow(non_snake_case)]
pub fn Cuboid<T1, T2, T3>(root: impl Into<Point3>, (dx, dy, dz): (T1, T2, T3)) -> Group3<()>
where
    f64: From<T1>,
    f64: From<T2>,
    f64: From<T3>,
{
    let root: Point3 = root.into();
    let dx: f64 = dx.into();
    let dy: f64 = dy.into();
    let dz: f64 = dz.into();

    let dx: Point3 = Point3(dx, 0.0, 0.0);
    let dy: Point3 = Point3(0.0, dy, 0.0);
    let dz: Point3 = Point3(0.0, 0.0, dz);

    Group3::<()>::new([
        (Obj3::Polygon3(make_planar_face(root, dx, dy)), ()),
        (Obj3::Polygon3(make_planar_face(root, dx, dz)), ()),
        (Obj3::Polygon3(make_planar_face(root, dy, dz)), ()),
        (Obj3::Polygon3(make_planar_face(root + dx, dy, dz)), ()),
        (Obj3::Polygon3(make_planar_face(root + dy, dx, dz)), ()),
        (Obj3::Polygon3(make_planar_face(root + dz, dx, dy)), ()),
    ])
}
