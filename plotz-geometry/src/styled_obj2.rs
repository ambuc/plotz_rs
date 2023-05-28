//! An annotated object with color and thickness.

use {
    crate::{
        bounded::Bounded,
        crop::{CropType, Croppable},
        obj2::Obj2,
        shapes::{pg2::Pg2, pt2::Pt2},
        traits::*,
    },
    plotz_color::{ColorRGB, BLACK},
    std::{fmt::Debug, ops::*},
};

/// An object with a color and thickness.
#[derive(PartialEq, Clone)]
pub struct StyledObj2 {
    /// The object.
    pub inner: Obj2,

    /// The color.
    pub color: &'static ColorRGB,

    /// The thickness.
    pub thickness: f64,
}

impl Debug for StyledObj2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let StyledObj2 {
            inner,
            color,
            thickness,
        } = self;
        let inner_fmt = match inner {
            Obj2::Pt(p) => format!("{:?}", p),
            Obj2::Pg2(pg) => format!("{:?}", pg),
            Obj2::Sg2(sg) => format!("{:?}", sg),
            Obj2::CurveArc(ca) => format!("{:?}", ca),
            Obj2::Txt(ch) => format!("{:?}", ch),
            Obj2::Group(g) => format!("{:?}", g),
        };
        write!(
            f,
            "Object2d::new({}).with_color({:?}).with_thickness({:?})",
            inner_fmt, color, thickness
        )
    }
}

impl StyledObj2 {
    /// from an object.
    pub fn new(obj: impl Into<Obj2>) -> StyledObj2 {
        StyledObj2 {
            inner: obj.into(),
            color: &BLACK,
            thickness: 0.1,
        }
    }

    /// with a color.
    pub fn with_color(self, color: &'static ColorRGB) -> StyledObj2 {
        StyledObj2 { color, ..self }
    }

    /// with a thickness.
    pub fn with_thickness(self, thickness: f64) -> StyledObj2 {
        StyledObj2 { thickness, ..self }
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

impl YieldPoints for StyledObj2 {
    fn yield_pts(&self) -> Box<dyn Iterator<Item = &Pt2> + '_> {
        self.inner.inner_impl_yield_points().yield_pts()
    }
}

impl YieldPointsMut for StyledObj2 {
    fn yield_pts_mut(&mut self) -> Box<dyn Iterator<Item = &mut Pt2> + '_> {
        self.inner.inner_impl_yield_points_mut().yield_pts_mut()
    }
}

impl Mutable for StyledObj2 {}

impl Bounded for StyledObj2 {
    fn bounds(&self) -> crate::bounded::Bounds {
        self.inner.bounds()
    }
}

impl RemAssign<Pt2> for StyledObj2 {
    fn rem_assign(&mut self, rhs: Pt2) {
        self.inner %= rhs;
    }
}

impl MulAssign<f64> for StyledObj2 {
    fn mul_assign(&mut self, rhs: f64) {
        self.inner *= rhs;
    }
}

impl DivAssign<f64> for StyledObj2 {
    fn div_assign(&mut self, rhs: f64) {
        self.inner /= rhs;
    }
}

impl AddAssign<Pt2> for StyledObj2 {
    fn add_assign(&mut self, rhs: Pt2) {
        self.inner += rhs;
    }
}

impl SubAssign<Pt2> for StyledObj2 {
    fn sub_assign(&mut self, rhs: Pt2) {
        self.inner -= rhs;
    }
}

impl Add<Pt2> for StyledObj2 {
    type Output = Self;
    fn add(self, rhs: Pt2) -> Self::Output {
        Self {
            inner: self.inner + rhs,
            ..self
        }
    }
}
impl Sub<Pt2> for StyledObj2 {
    type Output = Self;
    fn sub(self, rhs: Pt2) -> Self::Output {
        Self {
            inner: self.inner - rhs,
            ..self
        }
    }
}
impl Div<f64> for StyledObj2 {
    type Output = Self;
    fn div(self, rhs: f64) -> Self::Output {
        Self {
            inner: self.inner / rhs,
            ..self
        }
    }
}
impl Mul<f64> for StyledObj2 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            inner: self.inner * rhs,
            ..self
        }
    }
}

impl Translatable for StyledObj2 {}
impl Scalable<f64> for StyledObj2 {}
impl ScalableAssign for StyledObj2 {}
impl TranslatableAssign for StyledObj2 {}

impl Croppable for StyledObj2 {
    type Output = StyledObj2;

    fn crop(&self, other: &Pg2, crop_type: CropType) -> Vec<Self::Output>
    where
        Self: Sized,
    {
        match crop_type {
            CropType::Inclusive => self
                .inner
                .crop_to(other)
                .into_iter()
                .map(|doi| StyledObj2 {
                    inner: doi,
                    ..(*self)
                })
                .collect(),
            CropType::Exclusive => self
                .inner
                .crop_excluding(other)
                .into_iter()
                .map(|doi| StyledObj2 {
                    inner: doi,
                    ..(*self)
                })
                .collect(),
        }
    }
}

impl Annotatable for StyledObj2 {
    fn annotate(&self, settings: &AnnotationSettings) -> Vec<StyledObj2> {
        self.inner
            .annotate(settings)
            .into_iter()
            .map(|o| o.with_color(self.color))
            .collect()
    }
}