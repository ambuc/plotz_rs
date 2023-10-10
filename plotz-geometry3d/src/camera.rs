//! A camera.

use plotz_geometry::style::Style;

use crate::{
    obj3::Obj3,
    p3,
    shapes::{pg3::Pg3, pt3::Pt3, sg3::Sg3},
    styled_obj3::StyledObj3,
};
use plotz_geometry::{
    obj2::Obj2,
    shapes::{pg2::Pg2, pt2::Pt2, sg2::Sg2},
};

// Any oblique projection.  https://en.wikipedia.org/wiki/3D_projection#Oblique_projection
pub struct Oblique {
    u_src: Pt3,
    v_src: Pt3,
    w_src: Pt3,
    u_dst: Pt2,
    v_dst: Pt2,
    w_dst: Pt2,
}

impl Default for Oblique {
    // A standard oblique projection -- looking down at the origin from (1,1,1),
    // with x going down-and-to-the-left, y going down-and-to-the-right, and z
    // going straight up.
    fn default() -> Self {
        let spread = 1.0 / 2.0_f64.sqrt(); // 0.7071...
        Oblique {
            u_src: p3!(1, 0, 0),
            v_src: p3!(0, 1, 0),
            w_src: p3!(0, 0, 1),
            u_dst: Pt2(-1.0, spread),
            v_dst: Pt2(1.0, spread),
            w_dst: Pt2(0.0, -1.0),
        }
    }
}

impl Oblique {
    pub fn view_vector(&self) -> Pt3 {
        p3!(0, 0, 0) - self.u_src - self.v_src - self.w_src
    }

    pub fn project_pt3(&self, pt3d: &Pt3) -> Pt2 {
        (self.u_dst * pt3d.dot(&self.u_src))
            + (self.v_dst * pt3d.dot(&self.v_src))
            + (self.w_dst * pt3d.dot(&self.w_src))
    }
    pub fn project_sg3(&self, sg3: &Sg3) -> Sg2 {
        Sg2(self.project_pt3(&sg3.i), self.project_pt3(&sg3.f))
    }
    pub fn project_pg3(&self, pg3: &Pg3) -> Pg2 {
        Pg2(pg3.pts.iter().map(|pt3d| self.project_pt3(pt3d)))
    }
    pub fn project_obj3(&self, obj3: &Obj3) -> Obj2 {
        match obj3 {
            Obj3::Pg3(pg3d) => Obj2::Pg2(self.project_pg3(pg3d)),
            Obj3::Sg3(sg3d) => Obj2::Sg2(self.project_sg3(sg3d)),
        }
    }
    pub fn project_styled_obj3(&self, sobj3: &StyledObj3) -> (Obj2, Style) {
        (
            self.project_obj3(&sobj3.inner),
            sobj3.style.unwrap_or_default(),
        )
    }
}

pub enum Projection {
    /// https://en.wikipedia.org/wiki/Pohlke%27s_theorem
    Oblique(Oblique),
}

impl Default for Projection {
    fn default() -> Self {
        Projection::Oblique(Oblique::default())
    }
}

pub enum Occlusion {
    False,
    True,
}
