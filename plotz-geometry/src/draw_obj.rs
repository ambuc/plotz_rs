//! An annotated object with color and thickness.

use {
    crate::{
        bounded::Bounded,
        crop::{CropToPolygonError, Croppable},
        draw_obj_inner::DrawObjInner,
        point::Pt,
        polygon::Polygon,
        traits::*,
    },
    plotz_color::{ColorRGB, BLACK},
    std::ops::*,
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
            thickness: 0.1,
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
    fn bounds(&self) -> crate::bounded::Bounds {
        self.obj.bounds()
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

impl DivAssign<f64> for DrawObj {
    fn div_assign(&mut self, rhs: f64) {
        self.obj /= rhs;
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

impl Add<Pt> for DrawObj {
    type Output = Self;
    fn add(self, rhs: Pt) -> Self::Output {
        Self {
            obj: self.obj + rhs,
            ..self
        }
    }
}
impl Sub<Pt> for DrawObj {
    type Output = Self;
    fn sub(self, rhs: Pt) -> Self::Output {
        Self {
            obj: self.obj - rhs,
            ..self
        }
    }
}
impl Div<f64> for DrawObj {
    type Output = Self;
    fn div(self, rhs: f64) -> Self::Output {
        Self {
            obj: self.obj / rhs,
            ..self
        }
    }
}
impl Mul<f64> for DrawObj {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            obj: self.obj * rhs,
            ..self
        }
    }
}

impl Translatable for DrawObj {}
impl Scalable<f64> for DrawObj {}
impl ScalableAssign for DrawObj {}
impl TranslatableAssign for DrawObj {}

impl Croppable for DrawObj {
    type Output = DrawObj;
    fn crop_to(&self, frame: &Polygon) -> Result<Vec<Self::Output>, CropToPolygonError> {
        Ok(self
            .obj
            .crop_to(frame)?
            .into_iter()
            .map(|doi| DrawObj {
                obj: doi,
                ..(*self)
            })
            .collect())
    }
}
