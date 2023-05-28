//! A character at a point.

use {
    crate::{bounded::Bounded, shapes::pt2::Pt2, traits::*},
    std::ops::*,
};

#[derive(Debug, PartialEq, Clone)]
/// A character laid out at a point.
pub struct Txt {
    /// the point.
    pub pt: Pt2,
    /// the text.
    pub inner: String,
    /// The font size.
    pub font_size: f64,
}

impl Bounded for Txt {
    fn bounds(&self) -> crate::bounded::Bounds {
        self.pt.bounds()
    }
}

impl YieldPoints for Txt {
    fn yield_pts(&self) -> Box<dyn Iterator<Item = &Pt2> + '_> {
        Box::new(std::iter::once(&self.pt))
    }
}

impl YieldPointsMut for Txt {
    fn yield_pts_mut(&mut self) -> Box<dyn Iterator<Item = &mut Pt2> + '_> {
        Box::new(std::iter::once(&mut self.pt))
    }
}

impl Add<Pt2> for Txt {
    type Output = Self;
    fn add(self, rhs: Pt2) -> Self::Output {
        Self {
            pt: self.pt + rhs,
            ..self
        }
    }
}
impl Sub<Pt2> for Txt {
    type Output = Self;
    fn sub(self, rhs: Pt2) -> Self::Output {
        Self {
            pt: self.pt - rhs,
            ..self
        }
    }
}

impl AddAssign<Pt2> for Txt {
    fn add_assign(&mut self, rhs: Pt2) {
        self.pt += rhs;
    }
}

impl SubAssign<Pt2> for Txt {
    fn sub_assign(&mut self, rhs: Pt2) {
        self.pt -= rhs;
    }
}

impl Mul<Pt2> for Txt {
    type Output = Self;
    fn mul(self, rhs: Pt2) -> Self::Output {
        Self {
            pt: self.pt * rhs,
            ..self
        }
    }
}
impl Mul<f64> for Txt {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            pt: self.pt * rhs,
            ..self
        }
    }
}
impl Div<Pt2> for Txt {
    type Output = Self;
    fn div(self, rhs: Pt2) -> Self::Output {
        Self {
            pt: self.pt / rhs,
            ..self
        }
    }
}
impl Div<f64> for Txt {
    type Output = Self;
    fn div(self, rhs: f64) -> Self::Output {
        Self {
            pt: self.pt / rhs,
            ..self
        }
    }
}

impl MulAssign<f64> for Txt {
    fn mul_assign(&mut self, rhs: f64) {
        self.pt *= rhs;
    }
}
impl MulAssign<Pt2> for Txt {
    fn mul_assign(&mut self, rhs: Pt2) {
        self.pt *= rhs;
    }
}
impl DivAssign<f64> for Txt {
    fn div_assign(&mut self, rhs: f64) {
        self.pt /= rhs;
    }
}
impl DivAssign<Pt2> for Txt {
    fn div_assign(&mut self, rhs: Pt2) {
        self.pt /= rhs;
    }
}
impl RemAssign<Pt2> for Txt {
    fn rem_assign(&mut self, rhs: Pt2) {
        self.pt.x.0 %= rhs.x.0;
        self.pt.y.0 %= rhs.y.0;
    }
}

impl Mutable for Txt {}
impl Translatable for Txt {}
impl Scalable<Pt2> for Txt {}
impl Scalable<f64> for Txt {}

impl Nullable for Txt {
    fn is_empty(&self) -> bool {
        false
    }
}
