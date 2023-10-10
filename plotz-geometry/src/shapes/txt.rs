//! A character at a point.

use crate::{bounded::Bounded, shapes::pt::Pt, *};
use std::ops::*;

#[derive(Debug, PartialEq, Clone)]
/// A character laid out at a point.
pub struct Txt {
    /// the point.
    pub pt: Pt,
    /// the text.
    pub inner: String,
    /// The font size.
    pub font_size: f64,
}

impl Txt {
    /// Iterator.
    pub fn iter(&self) -> impl Iterator<Item = &Pt> {
        std::iter::once(&self.pt)
    }

    /// Mutable iterator.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Pt> {
        std::iter::once(&mut self.pt)
    }
}

impl Bounded for Txt {
    fn bounds(&self) -> crate::bounded::Bounds {
        self.pt.bounds()
    }
}

impl Add<Pt> for Txt {
    type Output = Self;
    fn add(self, rhs: Pt) -> Self::Output {
        Self {
            pt: self.pt + rhs,
            ..self
        }
    }
}
impl Sub<Pt> for Txt {
    type Output = Self;
    fn sub(self, rhs: Pt) -> Self::Output {
        Self {
            pt: self.pt - rhs,
            ..self
        }
    }
}

impl AddAssign<Pt> for Txt {
    fn add_assign(&mut self, rhs: Pt) {
        self.pt += rhs;
    }
}

impl SubAssign<Pt> for Txt {
    fn sub_assign(&mut self, rhs: Pt) {
        self.pt -= rhs;
    }
}

impl Mul<Pt> for Txt {
    type Output = Self;
    fn mul(self, rhs: Pt) -> Self::Output {
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
impl Div<Pt> for Txt {
    type Output = Self;
    fn div(self, rhs: Pt) -> Self::Output {
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
impl MulAssign<Pt> for Txt {
    fn mul_assign(&mut self, rhs: Pt) {
        self.pt *= rhs;
    }
}
impl DivAssign<f64> for Txt {
    fn div_assign(&mut self, rhs: f64) {
        self.pt /= rhs;
    }
}
impl DivAssign<Pt> for Txt {
    fn div_assign(&mut self, rhs: Pt) {
        self.pt /= rhs;
    }
}
impl RemAssign<Pt> for Txt {
    fn rem_assign(&mut self, rhs: Pt) {
        self.pt.x %= rhs.x;
        self.pt.y %= rhs.y;
    }
}

impl Translatable for Txt {}
impl Scalable<Pt> for Txt {}
impl Scalable<f64> for Txt {}

impl Nullable for Txt {
    fn is_empty(&self) -> bool {
        false
    }
}
