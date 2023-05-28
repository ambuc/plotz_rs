//! A camera.

use {
    crate::{p3, shapes::pt3::Pt3},
    plotz_geometry::{p2, shapes::pt2::Pt2},
};

// Any oblique projection.  https://en.wikipedia.org/wiki/3D_projection#Oblique_projection
pub struct Oblique {
    u_src: Pt3,
    v_src: Pt3,
    w_src: Pt3,
    u_dst: Pt2,
    v_dst: Pt2,
    w_dst: Pt2,
    /// the angle from which to view 3d objects (for dist along projection)
    pub view_vector: Pt3,
}

impl Oblique {
    // A standard oblique projection -- looking down at the origin from (1,1,1),
    // with x going down-and-to-the-left, y going down-and-to-the-right, and z
    // going straight up.
    pub fn standard() -> Oblique {
        let spread = 0.7;
        Oblique {
            u_src: p3!(1, 0, 0),
            v_src: p3!(0, 1, 0),
            w_src: p3!(0, 0, 1),
            u_dst: p2!(-1.0, spread),
            v_dst: p2!(1.0, spread),
            w_dst: p2!(0.0, -1.0),
            view_vector: p3!(-1.0, -1.0, -1.3),
        }
    }

    pub fn project(&self, pt3d: &Pt3) -> Pt2 {
        (self.u_dst * pt3d.dot(&self.u_src))
            + (self.v_dst * pt3d.dot(&self.v_src))
            + (self.w_dst * pt3d.dot(&self.w_src))
    }
}

pub enum Projection {
    /// https://en.wikipedia.org/wiki/Pohlke%27s_theorem
    Oblique(Oblique),
}

pub enum Occlusion {
    False,
    True,
}
