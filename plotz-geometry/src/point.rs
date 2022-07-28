use float_ord::FloatOrd;
use std::convert::From;
use std::{
    hash::Hash,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, Sub, SubAssign},
};

/// A point in 2D space.
#[derive(Debug, Hash, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pt {
    /// The x-coordinate of the point.
    pub x: FloatOrd<f64>,
    /// The y-coordinate of the point.
    pub y: FloatOrd<f64>,
}

/// An alternate constructor for points.
///
/// ```
/// use plotz_geometry::point::Pt;
/// assert_eq!(Pt{x:1, y:2}, Pt(1, 2));
/// ```
#[allow(non_snake_case)]
pub fn Pt<T>(x: T, y: T) -> Pt
where
    f64: From<T>,
{
    Pt {
        x: FloatOrd(x.into()),
        y: FloatOrd(y.into()),
    }
}

/// An implicit constructor from tuples.
///
/// ```
/// use plotz_geometry::point::Pt;
/// assert_eq!(Pt{x:1, y:2}, (1, 2).into());
/// ```
impl From<(f64, f64)> for Pt {
    fn from((x, y): (f64, f64)) -> Pt {
        Pt(x, y)
    }
}

/// A modulo operator for rounding points.
///
/// ```
/// use plotz_geometry::point::Pt;
/// assert_eq!(Pt(1.5, 1.5) % (1.0, 1.0), Pt(0.5, 0.5));
/// ```
impl Rem<(f64, f64)> for Pt {
    type Output = Self;

    fn rem(self, modulus: (f64, f64)) -> Self::Output {
        Pt(self.x.0 % modulus.0, self.y.0 % modulus.1)
    }
}

/// A div-assign operator for points.
///
/// ```
/// use plotz_geometry::point::Pt;
/// let mut p = Pt(1.5, 1.5);
/// p /= 2.0;
/// assert_eq!(p, Pt(0.75, 0.75));
/// ```
impl DivAssign<f64> for Pt {
    fn div_assign(&mut self, rhs: f64) {
        self.x.0 /= rhs;
        self.y.0 /= rhs;
    }
}

/// A addition operator for points.
///
/// ```
/// use plotz_geometry::point::Pt;
/// assert_eq!(Pt(1, 2) + Pt(3, 4), Pt(4, 6));
/// ```
impl Add<Pt> for Pt {
    type Output = Self;
    fn add(self, rhs: Pt) -> Self::Output {
        Pt(self.x.0 + rhs.x.0, self.y.0 + rhs.y.0)
    }
}

/// A add-assign operator for points.
///
/// ```
/// use plotz_geometry::point::Pt;
/// let mut p = Pt(2, 4);
/// p += Pt(1, 2);
/// assert_eq!(p, Pt(3, 6));
/// ```
impl AddAssign<Pt> for Pt {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: FloatOrd(self.x.0 + other.x.0),
            y: FloatOrd(self.y.0 + other.y.0),
        };
    }
}

/// A subtraction operator for points.
///
/// ```
/// use plotz_geometry::point::Pt;
/// assert_eq!(Pt(1, 2) - Pt(3, 4), Pt(-2, -2));
/// ```
impl Sub<Pt> for Pt {
    type Output = Self;
    fn sub(self, rhs: Pt) -> Self::Output {
        Pt(self.x.0 - rhs.x.0, self.y.0 - rhs.y.0)
    }
}

/// A sub-assign operator for points.
///
/// ```
/// use plotz_geometry::point::Pt;
/// let mut p = Pt(2, 4);
/// p -= Pt(1, 2);
/// assert_eq!(p, Pt(1, 2));
/// ```
impl SubAssign<Pt> for Pt {
    fn sub_assign(&mut self, other: Self) {
        *self = Self {
            x: FloatOrd(self.x.0 - other.x.0),
            y: FloatOrd(self.y.0 - other.y.0),
        };
    }
}

/// A multiplication operator for points.
///
/// ```
/// use plotz_geometry::point::Pt;
/// assert_eq!(Pt(1, 2) * 2, Pt(2, 4));
/// ```
impl Mul<f64> for Pt {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Pt(self.x.0 * rhs, self.y.0 * rhs)
    }
}

/// A sub-assign operator for points.
///
/// ```
/// use plotz_geometry::point::Pt;
/// let mut p = Pt(2, 4);
/// p -= Pt(1, 2);
/// assert_eq!(p, Pt(1, 2));
/// ```
impl MulAssign<f64> for Pt {
    fn mul_assign(&mut self, rhs: f64) {
        self.x.0 *= rhs;
        self.y.0 *= rhs;
    }
}

/// A division operator for points.
///
/// ```
/// use plotz_geometry::point::Pt;
/// assert_eq!(Pt(1.0, 2.0) / 2.0, Pt(0.5, 1.0)); // floats
/// assert_eq!(Pt(1, 2) / 2, Pt(0, 1)); // ints
/// ```
impl Div<f64> for Pt {
    type Output = Self;
    fn div(self, rhs: f64) -> Self::Output {
        Pt(self.x.0 / rhs, self.y.0 / rhs)
    }
}

impl Pt {
    /// A rotation operation, for rotating one point about another. Accepts a |by|
    /// argument in radians.
    pub fn rotate(&mut self, about: &Pt, by: f64) {
        *self -= *about;
        *self = Pt(
            (by.cos() * self.x.0) - (by.sin() * self.y.0),
            (by.sin() * self.x.0) + (by.cos() * self.y.0),
        );
        *self += *about;
    }

    /// Dot prouduct of (origin, self) • (origin, other)
    pub fn dot(&self, other: &Pt) -> f64 {
        (self.x.0 * other.x.0) + (self.y.0 * other.y.0)
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;

    use super::*;

    #[test]
    fn test_rotate() {
        use float_eq::assert_float_eq;
        use std::f64::consts::PI;

        let origin = Pt(0.0, 0.0);
        let mut p = Pt(1.0, 0.0);

        p.rotate(/*about=*/ &origin, PI / 2.0);
        assert_float_eq!(p.x.0, 0.0, abs <= 0.000_1);
        assert_float_eq!(p.y.0, 1.0, abs <= 0.000_1);

        p.rotate(/*about=*/ &origin, PI / 2.0);
        assert_float_eq!(p.x.0, -1.0, abs <= 0.000_1);
        assert_float_eq!(p.y.0, 0.0, abs <= 0.000_1);

        p.rotate(/*about=*/ &origin, PI / 2.0);
        assert_float_eq!(p.x.0, 0.0, abs <= 0.000_1);
        assert_float_eq!(p.y.0, -1.0, abs <= 0.000_1);

        p.rotate(/*about=*/ &origin, PI / 2.0);
        assert_float_eq!(p.x.0, 1.0, abs <= 0.000_1);
        assert_float_eq!(p.y.0, 0.0, abs <= 0.000_1);
    }

    #[test]
    fn test_dot() {
        assert_float_eq!(Pt(1.0, 1.0).dot(&Pt(1.0, 0.0)), 1.0, abs <= 0.000_1);
        assert_float_eq!(Pt(7.0, 2.0).dot(&Pt(3.0, 6.0)), 33.0, abs <= 0.000_1);
    }
}
