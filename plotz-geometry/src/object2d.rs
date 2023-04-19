//! An annotated object with color and thickness.

use crate::crop::CropType;

use {
    crate::{
        bounded::Bounded,
        crop::{CropToPolygonError, Croppable},
        object2d_inner::Object2dInner,
        point::Pt,
        polygon::Polygon,
        traits::*,
    },
    plotz_color::{ColorRGB, BLACK},
    std::ops::*,
};

/// An object with a color and thickness.
#[derive(Debug, PartialEq, Clone)]
pub struct Object2d {
    /// The object.
    pub inner: Object2dInner,

    /// The color.
    pub color: &'static ColorRGB,

    /// The thickness.
    pub thickness: f64,
}

impl Object2d {
    /// from an object.
    pub fn new(obj: impl Into<Object2dInner>) -> Object2d {
        Object2d {
            inner: obj.into(),
            color: &BLACK,
            thickness: 0.1,
        }
    }

    /// with a color.
    pub fn with_color(self, color: &'static ColorRGB) -> Object2d {
        Object2d { color, ..self }
    }

    /// with a thickness.
    pub fn with_thickness(self, thickness: f64) -> Object2d {
        Object2d { thickness, ..self }
    }

    /// Casts each inner value to something which implements Bounded.
    pub fn inner_impl_bounded(&self) -> &dyn Bounded {
        self.inner.inner_impl_bounded()
    }

    /// Casts each inner value to something which implements YieldPoints.
    pub fn inner_impl_yield_points(&self) -> &dyn YieldPoints {
        self.inner.inner_impl_yield_points()
    }

    /// Casts each inner value to something which implements YieldPointsMut.
    pub fn inner_impl_yield_points_mut(&mut self) -> &mut dyn YieldPointsMut {
        self.inner.inner_impl_yield_points_mut()
    }
}

impl YieldPoints for Object2d {
    fn yield_pts(&self) -> Option<Box<dyn Iterator<Item = &Pt> + '_>> {
        self.inner.inner_impl_yield_points().yield_pts()
    }
}

impl YieldPointsMut for Object2d {
    fn yield_pts_mut(&mut self) -> Option<Box<dyn Iterator<Item = &mut Pt> + '_>> {
        self.inner.inner_impl_yield_points_mut().yield_pts_mut()
    }
}

impl Mutable for Object2d {}

impl Bounded for Object2d {
    fn bounds(&self) -> crate::bounded::Bounds {
        self.inner.bounds()
    }
}

impl RemAssign<Pt> for Object2d {
    fn rem_assign(&mut self, rhs: Pt) {
        self.inner %= rhs;
    }
}

impl MulAssign<f64> for Object2d {
    fn mul_assign(&mut self, rhs: f64) {
        self.inner *= rhs;
    }
}

impl DivAssign<f64> for Object2d {
    fn div_assign(&mut self, rhs: f64) {
        self.inner /= rhs;
    }
}

impl AddAssign<Pt> for Object2d {
    fn add_assign(&mut self, rhs: Pt) {
        self.inner += rhs;
    }
}

impl SubAssign<Pt> for Object2d {
    fn sub_assign(&mut self, rhs: Pt) {
        self.inner -= rhs;
    }
}

impl Add<Pt> for Object2d {
    type Output = Self;
    fn add(self, rhs: Pt) -> Self::Output {
        Self {
            inner: self.inner + rhs,
            ..self
        }
    }
}
impl Sub<Pt> for Object2d {
    type Output = Self;
    fn sub(self, rhs: Pt) -> Self::Output {
        Self {
            inner: self.inner - rhs,
            ..self
        }
    }
}
impl Div<f64> for Object2d {
    type Output = Self;
    fn div(self, rhs: f64) -> Self::Output {
        Self {
            inner: self.inner / rhs,
            ..self
        }
    }
}
impl Mul<f64> for Object2d {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            inner: self.inner * rhs,
            ..self
        }
    }
}

impl Translatable for Object2d {}
impl Scalable<f64> for Object2d {}
impl ScalableAssign for Object2d {}
impl TranslatableAssign for Object2d {}

impl Croppable for Object2d {
    type Output = Object2d;

    fn crop(
        &self,
        other: &Polygon,
        crop_type: CropType,
    ) -> Result<Vec<Self::Output>, CropToPolygonError>
    where
        Self: Sized,
    {
        match crop_type {
            CropType::Inclusive => Ok(self
                .inner
                .crop_to(other)?
                .into_iter()
                .map(|doi| Object2d {
                    inner: doi,
                    ..(*self)
                })
                .collect()),
            CropType::Exclusive => Ok(self
                .inner
                .crop_excluding(other)?
                .into_iter()
                .map(|doi| Object2d {
                    inner: doi,
                    ..(*self)
                })
                .collect()),
        }
    }
}

impl Annotatable for Object2d {
    fn annotate(&self) -> Vec<Object2d> {
        self.inner
            .annotate()
            .into_iter()
            .map(|o| o.with_color(self.color))
            .collect()
    }
}
