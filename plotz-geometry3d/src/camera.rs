//! A camera.

use plotz_geometry::{group::Group, style::Style};

use crate::{
    group3::Group3,
    obj3::Obj3,
    shapes::{point3::Point3, polygon3::Polygon3, segment3::Segment3},
};
use plotz_geometry::{
    obj::Obj2,
    shapes::{point::Point, polygon::Polygon, segment::Segment},
};

// Any oblique projection.  https://en.wikipedia.org/wiki/3D_projection#Oblique_projection
#[derive(Debug, Clone)]
pub struct Oblique {
    u_src: Point3,
    v_src: Point3,
    w_src: Point3,
    u_dst: Point,
    v_dst: Point,
    w_dst: Point,
}

impl Default for Oblique {
    // A standard oblique projection -- looking down at the origin from (1,1,1),
    // with x going down-and-to-the-left, y going down-and-to-the-right, and z
    // going straight up.
    fn default() -> Self {
        let spread = 1.0 / 2.0_f64.sqrt(); // 0.7071...
        Oblique {
            u_src: Point3(1, 0, 0),
            v_src: Point3(0, 1, 0),
            w_src: Point3(0, 0, 1),
            u_dst: Point(-1, spread),
            v_dst: Point(1, spread),
            w_dst: Point(0, -1),
        }
    }
}

impl Oblique {
    pub fn view_vector(&self) -> Point3 {
        Point3(0, 0, 0) - self.u_src - self.v_src - self.w_src
    }

    pub fn project_pt3(&self, pt3d: &Point3) -> Point {
        (self.u_dst * pt3d.dot(&self.u_src))
            + (self.v_dst * pt3d.dot(&self.v_src))
            + (self.w_dst * pt3d.dot(&self.w_src))
    }
    pub fn project_sg3(&self, sg3: &Segment3) -> Segment {
        Segment(self.project_pt3(&sg3.i), self.project_pt3(&sg3.f))
    }
    pub fn project_pg3(&self, pg3: &Polygon3) -> Polygon {
        Polygon(pg3.pts.iter().map(|pt3d| self.project_pt3(pt3d))).unwrap()
    }
    pub fn project_group3(&self, _: &Group3<()>) -> Group<Style> {
        todo!("https://github.com/ambuc/plotz_rs/issues/6")
    }
    pub fn project_obj3(&self, obj3: &Obj3) -> Obj2 {
        match obj3 {
            Obj3::Polygon3(pg3d) => Obj2::Polygon(self.project_pg3(pg3d)),
            Obj3::Segment3(sg3d) => Obj2::Segment(self.project_sg3(sg3d)),
            Obj3::Group3(g3d) => Obj2::Group(self.project_group3(g3d)),
        }
    }
    pub fn project_styled_obj3(&self, (obj3, style): &(Obj3, Style)) -> (Obj2, Style) {
        (self.project_obj3(obj3), *style)
    }
}

#[derive(Debug, Clone)]
pub enum Projection {
    /// https://en.wikipedia.org/wiki/Pohlke%27s_theorem
    Oblique(Oblique),
}

impl Default for Projection {
    fn default() -> Self {
        Projection::Oblique(Oblique::default())
    }
}

#[derive(Debug, Clone, Default)]
pub enum Occlusion {
    False,
    #[default]
    True,
}
