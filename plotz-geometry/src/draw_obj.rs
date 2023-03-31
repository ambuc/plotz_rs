//! An annotated object with color and thickness.

use std::ops::{AddAssign, MulAssign, RemAssign, SubAssign};

use {
    crate::draw_obj_inner::DrawObjInner,
    crate::{
        bounded::Bounded,
        point::Pt,
        traits::{Mutable, YieldPoints, YieldPointsMut},
    },
    plotz_color::{ColorRGB, BLACK},
};

/// An object with a color and thickness.
#[derive(Debug, PartialEq, Clone)]
pub struct DrawObj {
    /// The object.
    pub obj: DrawObjInner,

    /// The color.
    pub color: &'static ColorRGB,

    /// The thickness.
    pub thickness: f64,
}

impl DrawObj {
    /// from an object.
    pub fn new(obj: impl Into<DrawObjInner>) -> DrawObj {
        DrawObj {
            obj: obj.into(),
            color: &BLACK,
            thickness: 1.0,
        }
    }

    /// with a color.
    pub fn with_color(self, color: &'static ColorRGB) -> DrawObj {
        DrawObj { color, ..self }
    }

    /// with a thickness.
    pub fn with_thickness(self, thickness: f64) -> DrawObj {
        DrawObj { thickness, ..self }
    }
}

impl YieldPoints for DrawObj {
    fn yield_pts(&self) -> Option<Box<dyn Iterator<Item = &Pt> + '_>> {
        self.obj.inner_impl_yield_points().yield_pts()
    }
}

impl YieldPointsMut for DrawObj {
    fn yield_pts_mut(&mut self) -> Option<Box<dyn Iterator<Item = &mut Pt> + '_>> {
        self.obj.inner_impl_yield_points_mut().yield_pts_mut()
    }
}

impl Mutable for DrawObj {}

impl Bounded for DrawObj {
    fn right_bound(&self) -> f64 {
        self.obj.right_bound()
    }
    fn left_bound(&self) -> f64 {
        self.obj.left_bound()
    }
    fn top_bound(&self) -> f64 {
        self.obj.top_bound()
    }
    fn bottom_bound(&self) -> f64 {
        self.obj.bottom_bound()
    }
}

impl RemAssign<Pt> for DrawObj {
    fn rem_assign(&mut self, rhs: Pt) {
        self.obj %= rhs;
    }
}

impl MulAssign<f64> for DrawObj {
    fn mul_assign(&mut self, rhs: f64) {
        self.obj *= rhs;
    }
}

impl AddAssign<Pt> for DrawObj {
    fn add_assign(&mut self, rhs: Pt) {
        self.obj += rhs;
    }
}

impl SubAssign<Pt> for DrawObj {
    fn sub_assign(&mut self, rhs: Pt) {
        self.obj -= rhs;
    }
}
