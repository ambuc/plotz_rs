//! A polygon in 3d.

use {
    crate::{point3d::Pt3d, segment3d::Segment3d},
    itertools::zip,
    std::ops::*,
};

/// Whether this polygon is open or closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A polygon is open, i.e. a multiline. No fill is possible.
    Open,
    /// A polygon is closed.
    Closed(Fill),
}

/// Whether this polygon should be opaque or transparent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fill {
    /// A polygon is opaque.
    Opaque,
    /// A polygon is transparent.
    Transparent,
}

/// A multiline is a list of points rendered with connecting line segments.
/// If constructed with Kind::Open, this is a multiline (unshaded).
/// If constructed with Kind::Closed, this is a closed, shaded polygon.
#[derive(Debug, Clone)]
pub struct Polygon3d {
    /// The points which describe a polygon or multiline.
    pub pts: Vec<Pt3d>,
    /// Whether this polygon is open or closed.
    pub kind: Kind,
}

/// Constructor for multilines, which are by definition open.
#[allow(non_snake_case)]
pub fn Multiline3d(a: impl IntoIterator<Item = Pt3d>) -> Polygon3d {
    Polygon3d {
        pts: a.into_iter().collect(),
        kind: Kind::Open,
    }
}

/// Constructor for polygons which are closed but may/may not be opaque or
/// transparent.
#[allow(non_snake_case)]
pub fn Polygon3d(a: impl IntoIterator<Item = Pt3d>, fill: Fill) -> Polygon3d {
    Polygon3d {
        pts: a.into_iter().collect(),
        kind: Kind::Closed(fill),
    }
}

impl Polygon3d {
    /// Turns a polygon3d into a vector of segment3ds.
    pub fn to_segments(&self) -> Vec<Segment3d> {
        match self.kind {
            Kind::Open => zip(self.pts.iter(), self.pts.iter().skip(1))
                .map(|(i, f)| Segment3d(*i, *f))
                .collect(),
            Kind::Closed(_) => zip(self.pts.iter(), self.pts.iter().cycle().skip(1))
                .map(|(i, f)| Segment3d(*i, *f))
                .collect(),
        }
    }
}

impl Add<Pt3d> for &Polygon3d {
    type Output = Polygon3d;
    fn add(self, rhs: Pt3d) -> Self::Output {
        Polygon3d {
            pts: self.pts.iter().map(|p| *p + rhs).collect(),
            ..self.clone()
        }
    }
}
impl Add<Pt3d> for Polygon3d {
    type Output = Polygon3d;
    fn add(self, rhs: Pt3d) -> Self::Output {
        &self + rhs
    }
}
impl AddAssign<Pt3d> for Polygon3d {
    fn add_assign(&mut self, rhs: Pt3d) {
        self.pts.iter_mut().for_each(|p| *p += rhs);
    }
}
impl Div<Pt3d> for Polygon3d {
    type Output = Polygon3d;
    fn div(self, rhs: Pt3d) -> Self::Output {
        Polygon3d {
            pts: self.pts.iter().map(|p| *p / rhs).collect(),
            ..self
        }
    }
}
impl Div<f64> for Polygon3d {
    type Output = Polygon3d;
    fn div(self, rhs: f64) -> Self::Output {
        Polygon3d {
            pts: self.pts.iter().map(|p| *p / rhs).collect(),
            ..self
        }
    }
}
impl DivAssign<Pt3d> for Polygon3d {
    fn div_assign(&mut self, rhs: Pt3d) {
        self.pts.iter_mut().for_each(|p| *p /= rhs);
    }
}
impl DivAssign<f64> for Polygon3d {
    fn div_assign(&mut self, rhs: f64) {
        self.pts.iter_mut().for_each(|p| *p /= rhs);
    }
}
impl Mul<Pt3d> for Polygon3d {
    type Output = Polygon3d;
    fn mul(self, rhs: Pt3d) -> Polygon3d {
        Polygon3d {
            pts: self.pts.iter().map(|p| *p * rhs).collect(),
            ..self
        }
    }
}
impl Mul<f64> for Polygon3d {
    type Output = Polygon3d;
    fn mul(mut self, rhs: f64) -> Polygon3d {
        self *= rhs;
        self
    }
}
impl MulAssign<Pt3d> for Polygon3d {
    fn mul_assign(&mut self, rhs: Pt3d) {
        self.pts.iter_mut().for_each(|p| *p *= rhs);
    }
}
impl MulAssign<f64> for Polygon3d {
    fn mul_assign(&mut self, rhs: f64) {
        self.pts.iter_mut().for_each(|p| *p *= rhs);
    }
}
impl Sub<Pt3d> for &Polygon3d {
    type Output = Polygon3d;
    fn sub(self, rhs: Pt3d) -> Self::Output {
        Polygon3d {
            pts: self.pts.iter().map(|p| *p - rhs).collect(),
            ..self.clone()
        }
    }
}
impl Sub<Pt3d> for Polygon3d {
    type Output = Polygon3d;
    fn sub(self, rhs: Pt3d) -> Self::Output {
        Polygon3d {
            pts: self.pts.iter().map(|p| *p - rhs).collect(),
            ..self
        }
    }
}
impl SubAssign<Pt3d> for Polygon3d {
    fn sub_assign(&mut self, rhs: Pt3d) {
        self.pts.iter_mut().for_each(|p| *p -= rhs);
    }
}