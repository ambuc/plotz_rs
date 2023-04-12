//! A cube in 3d.

use crate::{face::Face, group::Group, point3d::Pt3d, polygon3d::Polygon3d};

/// make a cube of faces (no edges)
#[allow(non_snake_case)]
pub fn Cube(root: Pt3d, (dx, dy, dz): (f64, f64, f64)) -> Group<Face> {
    let dx = Pt3d(dx, 0.0, 0.0);
    let dy = Pt3d(0.0, dy, 0.0);
    let dz = Pt3d(0.0, 0.0, dz);

    let o = root;
    let ox = o + dx;
    let oxy = ox + dy;
    let oxyz = oxy + dz;
    let oxz = ox + dz;
    let oy = o + dy;
    let oyz = oy + dz;
    let oz = o + dz;

    Group(vec![
        Face::from(Polygon3d([o, ox, oxy, oy, o])),
        Face::from(Polygon3d([o, ox, oxz, oz, o])),
        Face::from(Polygon3d([o, oy, oyz, oz, o])),
        Face::from(Polygon3d([ox, oxy, oxyz, oxz, ox])),
        Face::from(Polygon3d([oy, oyz, oxyz, oxy, oy])),
        Face::from(Polygon3d([oz, oxz, oxyz, oyz, oz])),
    ])
}
