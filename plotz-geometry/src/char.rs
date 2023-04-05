//! A character at a point.

use {
    crate::{bounded::Bounded, point::Pt, traits::*},
    std::ops::*,
};

#[derive(Debug, PartialEq, Clone)]
/// A character laid out at a point.
pub struct Char {
    /// the point.
    pub pt: Pt,
    /// the character.
    pub chr: char,
}

impl Bounded for Char {
    fn bounds(&self) -> crate::bounded::Bounds {
        self.pt.bounds()
    }
}

impl YieldPoints for Char {
    fn yield_pts(&self) -> Option<Box<dyn Iterator<Item = &Pt> + '_>> {
        Some(Box::new(std::iter::once(&self.pt)))
    }
}

impl YieldPointsMut for Char {
    fn yield_pts_mut(&mut self) -> Option<Box<dyn Iterator<Item = &mut Pt> + '_>> {
        Some(Box::new(std::iter::once(&mut self.pt)))
    }
}

impl Add<Pt> for Char {
    type Output = Self;
    fn add(self, rhs: Pt) -> Self::Output {
        Self {
            pt: self.pt + rhs,
            ..self
        }
    }
}
impl Sub<Pt> for Char {
    type Output = Self;
    fn sub(self, rhs: Pt) -> Self::Output {
        Self {
            pt: self.pt - rhs,
            ..self
        }
    }
}

impl AddAssign<Pt> for Char {
    fn add_assign(&mut self, rhs: Pt) {
        self.pt += rhs;
    }
}

impl SubAssign<Pt> for Char {
    fn sub_assign(&mut self, rhs: Pt) {
        self.pt -= rhs;
    }
}

impl Mul<Pt> for Char {
    type Output = Self;
    fn mul(self, rhs: Pt) -> Self::Output {
        Self {
            pt: self.pt * rhs,
            ..self
        }
    }
}
impl Mul<f64> for Char {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            pt: self.pt * rhs,
            ..self
        }
    }
}
impl Div<Pt> for Char {
    type Output = Self;
    fn div(self, rhs: Pt) -> Self::Output {
        Self {
            pt: self.pt / rhs,
            ..self
        }
    }
}
impl Div<f64> for Char {
    type Output = Self;
    fn div(self, rhs: f64) -> Self::Output {
        Self {
            pt: self.pt / rhs,
            ..self
        }
    }
}

impl MulAssign<f64> for Char {
    fn mul_assign(&mut self, rhs: f64) {
        self.pt *= rhs;
    }
}
impl MulAssign<Pt> for Char {
    fn mul_assign(&mut self, rhs: Pt) {
        self.pt *= rhs;
    }
}
impl DivAssign<f64> for Char {
    fn div_assign(&mut self, rhs: f64) {
        self.pt /= rhs;
    }
}
impl DivAssign<Pt> for Char {
    fn div_assign(&mut self, rhs: Pt) {
        self.pt /= rhs;
    }
}
impl RemAssign<Pt> for Char {
    fn rem_assign(&mut self, rhs: Pt) {
        self.pt.x.0 %= rhs.x.0;
        self.pt.y.0 %= rhs.y.0;
    }
}

impl Mutable for Char {}
impl Translatable for Char {}
impl Scalable<Pt> for Char {}
impl Scalable<f64> for Char {}
