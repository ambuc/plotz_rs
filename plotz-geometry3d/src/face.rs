//! A filled polygon without edges.

use crate::point3d::Pt3d;
use derive_more::From;

use {crate::polygon3d::Polygon3d, std::ops::*};

/// A Face is a polygon which is opaque, i.e. the face of the polygon rather
/// than its edges.
#[derive(Debug, Clone, From)]
pub struct Face {
    /// The polygon.
    pub pg3d: Polygon3d,
}

impl Add<Pt3d> for &Face {
    type Output = Face;
    fn add(self, rhs: Pt3d) -> Self::Output {
        Face {
            pg3d: self.pg3d.clone() + rhs,
            ..self.clone()
        }
    }
}
impl Add<Pt3d> for Face {
    type Output = Face;
    fn add(self, rhs: Pt3d) -> Self::Output {
        &self + rhs
    }
}
impl AddAssign<Pt3d> for Face {
    fn add_assign(&mut self, rhs: Pt3d) {
        self.pg3d += rhs;
    }
}
impl Div<Pt3d> for Face {
    type Output = Face;
    fn div(self, rhs: Pt3d) -> Self::Output {
        Face {
            pg3d: self.pg3d / rhs,
            ..self
        }
    }
}
impl Div<f64> for Face {
    type Output = Face;
    fn div(self, rhs: f64) -> Self::Output {
        Face {
            pg3d: self.pg3d / rhs,
            ..self
        }
    }
}
impl DivAssign<Pt3d> for Face {
    fn div_assign(&mut self, rhs: Pt3d) {
        self.pg3d /= rhs;
    }
}
impl DivAssign<f64> for Face {
    fn div_assign(&mut self, rhs: f64) {
        self.pg3d /= rhs;
    }
}
impl Mul<Pt3d> for Face {
    type Output = Face;
    fn mul(self, rhs: Pt3d) -> Face {
        Face {
            pg3d: self.pg3d * rhs,
            ..self
        }
    }
}
impl Mul<f64> for Face {
    type Output = Face;
    fn mul(self, rhs: f64) -> Face {
        Face {
            pg3d: self.pg3d * rhs,
            ..self
        }
    }
}
impl MulAssign<Pt3d> for Face {
    fn mul_assign(&mut self, rhs: Pt3d) {
        self.pg3d *= rhs;
    }
}
impl MulAssign<f64> for Face {
    fn mul_assign(&mut self, rhs: f64) {
        self.pg3d *= rhs;
    }
}
impl Sub<Pt3d> for &Face {
    type Output = Face;
    fn sub(self, rhs: Pt3d) -> Self::Output {
        Face {
            pg3d: self.pg3d.clone() - rhs,
            ..self.clone()
        }
    }
}
impl Sub<Pt3d> for Face {
    type Output = Face;
    fn sub(self, rhs: Pt3d) -> Self::Output {
        Face {
            pg3d: self.pg3d.clone() - rhs,
            ..self.clone()
        }
    }
}
impl SubAssign<Pt3d> for Face {
    fn sub_assign(&mut self, rhs: Pt3d) {
        self.pg3d -= rhs;
    }
}
