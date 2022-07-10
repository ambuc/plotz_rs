use float_cmp::approx_eq;
use num::Float;
use std::convert::From;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, Sub, SubAssign};

/// A point in 2D space.
#[derive(Debug, Hash, Copy, Clone)]
pub struct Pt<T> {
    /// The x-coordinate of the point.
    pub x: T,
    /// The y-coordinate of the point.
    pub y: T,
}

/// An alternate constructor for points.
///
/// ```
/// use plotz_geometry::point::Pt;
/// assert_eq!(Pt{x:1, y:2}, Pt(1, 2));
/// ```
#[allow(non_snake_case)]
pub fn Pt<T>(x: T, y: T) -> Pt<T> {
    Pt { x, y }
}

impl PartialEq for Pt<i32> {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}
impl PartialEq for Pt<f32> {
    fn eq(&self, other: &Self) -> bool {
        approx_eq!(f32, self.x, other.x, ulps = 2) && approx_eq!(f32, self.y, other.y, ulps = 2)
    }
}

impl Eq for Pt<i32> {}
impl Eq for Pt<f32> {}

/// An implicit constructor from tuples.
///
/// ```
/// use plotz_geometry::point::Pt;
/// assert_eq!(Pt{x:1, y:2}, (1, 2).into());
/// ```
impl<T> From<(T, T)> for Pt<T> {
    fn from((x, y): (T, T)) -> Pt<T> {
        Pt(x, y)
    }
}

/// A copy constructor from another numeric type.
///
/// ```
/// use plotz_geometry::point::Pt;
/// let pt_t: Pt<u8> = Pt(1, 2);
/// let pt_u: Pt<f64> = (&pt_t).into();
/// ```
impl<T, U> From<&Pt<T>> for Pt<U>
where
    T: Copy,
    U: From<T>,
{
    fn from(p: &Pt<T>) -> Pt<U> {
        Pt(p.x.into(), p.y.into())
    }
}

/// A modulo operator for rounding points.
///
/// ```
/// use plotz_geometry::point::Pt;
/// assert_eq!(Pt(1.5, 1.5) % (1.0, 1.0), Pt(0.5, 0.5));
/// ```
impl<T> Rem<(T, T)> for Pt<T>
where
    T: Rem<Output = T>,
{
    type Output = Self;

    fn rem(self, modulus: (T, T)) -> Self::Output {
        Pt(self.x % modulus.0, self.y % modulus.1)
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
impl<T> DivAssign<T> for Pt<T>
where
    T: DivAssign<T> + Copy,
{
    fn div_assign(&mut self, rhs: T) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

/// A addition operator for points.
///
/// ```
/// use plotz_geometry::point::Pt;
/// assert_eq!(Pt(1, 2) + Pt(3, 4), Pt(4, 6));
/// ```
impl<T> Add<Pt<T>> for Pt<T>
where
    T: Add<Output = T>,
{
    type Output = Self;
    fn add(self, rhs: Pt<T>) -> Self::Output {
        Pt {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
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
impl<T> AddAssign<Pt<T>> for Pt<T>
where
    T: Add<Output = T> + Copy,
{
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x + other.x,
            y: self.y + other.y,
        };
    }
}

/// A subtraction operator for points.
///
/// ```
/// use plotz_geometry::point::Pt;
/// assert_eq!(Pt(1, 2) - Pt(3, 4), Pt(-2, -2));
/// ```
impl<T> Sub<Pt<T>> for Pt<T>
where
    T: Sub<Output = T>,
{
    type Output = Self;
    fn sub(self, rhs: Pt<T>) -> Self::Output {
        Pt {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
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
impl<T> SubAssign<Pt<T>> for Pt<T>
where
    T: Sub<Output = T> + Copy,
{
    fn sub_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x - other.x,
            y: self.y - other.y,
        };
    }
}

/// A multiplication operator for points.
///
/// ```
/// use plotz_geometry::point::Pt;
/// assert_eq!(Pt(1, 2) * 2, Pt(2, 4));
/// ```
impl<T> Mul<T> for Pt<T>
where
    T: Mul<Output = T> + Copy,
{
    type Output = Self;
    fn mul(self, rhs: T) -> Self::Output {
        Pt {
            x: self.x * rhs,
            y: self.y * rhs,
        }
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
impl<T> MulAssign<T> for Pt<T>
where
    T: MulAssign + Copy,
{
    fn mul_assign(&mut self, rhs: T) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

/// A division operator for points.
///
/// ```
/// use plotz_geometry::point::Pt;
/// assert_eq!(Pt(1.0, 2.0) / 2.0, Pt(0.5, 1.0)); // floats
/// assert_eq!(Pt(1, 2) / 2, Pt(0, 1)); // ints
/// ```
impl<T> Div<T> for Pt<T>
where
    T: Div<Output = T> + Copy,
{
    type Output = Self;
    fn div(self, rhs: T) -> Self::Output {
        Pt {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl<T> Pt<T> {
    /// A rotation operation, for rotating one point about another. Accepts a |by|
    /// argument in radians.
    pub fn rotate(&mut self, about: &Pt<T>, by: T)
    where
        T: Float,
    {
        *self -= *about;
        *self = Pt(
            (by.cos() * self.x) - (by.sin() * self.y),
            (by.sin() * self.x) + (by.cos() * self.y),
        );
        *self += *about;
    }

    /// Dot prouduct of (origin, self) • (origin, other)
    pub fn dot(&self, other: &Pt<T>) -> T
    where
        T: Float,
    {
        (self.x * other.x) + (self.y * other.y)
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
        assert_float_eq!(p.x, 0.0, abs <= 0.000_1);
        assert_float_eq!(p.y, 1.0, abs <= 0.000_1);

        p.rotate(/*about=*/ &origin, PI / 2.0);
        assert_float_eq!(p.x, -1.0, abs <= 0.000_1);
        assert_float_eq!(p.y, 0.0, abs <= 0.000_1);

        p.rotate(/*about=*/ &origin, PI / 2.0);
        assert_float_eq!(p.x, 0.0, abs <= 0.000_1);
        assert_float_eq!(p.y, -1.0, abs <= 0.000_1);

        p.rotate(/*about=*/ &origin, PI / 2.0);
        assert_float_eq!(p.x, 1.0, abs <= 0.000_1);
        assert_float_eq!(p.y, 0.0, abs <= 0.000_1);
    }

    #[test]
    fn test_dot() {
        assert_float_eq!(Pt(1.0, 1.0).dot(&Pt(1.0, 0.0)), 1.0, abs <= 0.000_1);
        assert_float_eq!(Pt(7.0, 2.0).dot(&Pt(3.0, 6.0)), 33.0, abs <= 0.000_1);
    }
}
