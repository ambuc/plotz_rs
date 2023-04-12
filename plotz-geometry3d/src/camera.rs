//! A camera.

#![allow(unused)]

use plotz_geometry::point::Pt;
use {crate::point3d::Pt3d, typed_builder::TypedBuilder};

#[derive(Debug, Clone, TypedBuilder)]
/// A camera.
pub struct Camera {
    /// At a location.
    at: Pt3d,
    /// Facing a direction.
    towards: Pt3d,
    /// Which direction is 'up' for the camera: controls the roll. This is
    /// really a Vec3d relative to [at].
    up: Pt3d,
}

impl Camera {
    // For now, no frustrum culling: if the thing is in the scene, project it
    // and render it.
    //
}

/// Any oblique projection.
/// https://en.wikipedia.org/wiki/3D_projection#Oblique_projection
pub struct Oblique {
    u_src: Pt3d,
    v_src: Pt3d,
    w_src: Pt3d,
    u_dst: Pt,
    v_dst: Pt,
    w_dst: Pt,
}

impl Oblique {
    /// A standard oblique projection -- looking down at the origin from
    /// (1,1,1), with x going down-and-to-the-left, y going
    /// down-and-to-the-right, and z going straight up.
    pub fn standard() -> Oblique {
        Oblique {
            u_src: Pt3d(1.0, 0.0, 0.0),
            v_src: Pt3d(0.0, 1.0, 0.0),
            w_src: Pt3d(0.0, 0.0, 1.0),
            u_dst: Pt(-1.0, 1.0),
            v_dst: Pt(1.0, 1.0),
            w_dst: Pt(0.0, -1.0),
        }
    }

    /// Projects a 3d point down to a 3d point.
    pub fn project(&self, pt3d: &Pt3d) -> Pt {
        (self.u_dst * pt3d.dot(&self.u_src))
            + (self.v_dst * pt3d.dot(&self.v_src))
            + (self.w_dst * pt3d.dot(&self.w_src))
    }
}

/// https://en.wikipedia.org/wiki/3D_projection
pub enum Projection {
    /// https://en.wikipedia.org/wiki/Pohlke%27s_theorem
    Oblique(Oblique),
    // More to come
}
