use crate::point::Pt;
use num::Float;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

/// Returns true if the three points are listed in a counter-clockwise order.
fn ccw<T>(a: Pt<T>, b: Pt<T>, c: Pt<T>) -> bool
where
    T: Float,
{
    (c.y - a.y) * (b.x - a.x) > (b.y - a.y) * (c.x - a.x)
}

/// A segment in 2D space, with initial and final points.
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub struct Segment<T> {
    /// The initial point of the segment.
    pub i: Pt<T>,
    /// The final point of the segment.
    pub f: Pt<T>,
}

/// An alternate constructor for segments.
///
/// ```
/// use plotz_geometry::{point::Pt, segment::Segment};
/// assert_eq!(Segment{i: Pt(0,0), f: Pt(0,1)}, Segment(Pt(0,0), Pt(0,1)));
/// ```
#[allow(non_snake_case)]
pub fn Segment<T>(i: Pt<T>, f: Pt<T>) -> Segment<T> {
    Segment { i, f }
}

impl<T> Segment<T>
where
    T: Float,
{
    /// The slope of a line segment.
    pub fn slope(&self) -> T {
        (self.f.y - self.i.y) / (self.f.x - self.i.x)
    }

    /// A rotation operation, for rotating a line segment about a point. Accepts
    /// a |by| argument in radians.
    pub fn rotate(&mut self, about: &Pt<T>, by: T) {
        self.i.rotate(about, by);
        self.f.rotate(about, by);
    }

    /// Returns true if one line segment intersects another.
    /// If two line segments share a point, returns false.
    /// If two line segments are parallel and overlapping, returns false.
    /// If two line segments are the same, returns false.
    pub fn intersects(self, other: &Segment<T>) -> bool
    where
        T: Float,
    {
        ccw(self.i, other.i, other.f) != ccw(self.f, other.i, other.f)
            && ccw(self.i, self.f, other.i) != ccw(self.i, self.f, other.f)
    }
}

/// An add operation between a segment and a point. This can be seen as
/// transposition by |rhs|.
///
/// ```
/// use plotz_geometry::{point::Pt, segment::Segment};
/// assert_eq!(
///       Segment(Pt(0,0), Pt(1,1))
///     + Pt(1,0),  
///    // --------
///       Segment(Pt(1,0), Pt(2,1))
/// );
/// ```
impl<T> Add<Pt<T>> for Segment<T>
where
    T: Add<Output = T> + Copy,
{
    type Output = Segment<T>;
    fn add(self, rhs: Pt<T>) -> Self::Output {
        Segment(self.i + rhs, self.f + rhs)
    }
}

/// An add-assign operation between a segment and a point. This can be seen as a
/// transposition by |rhs|.
///
/// ```
/// use plotz_geometry::{point::Pt, segment::Segment};
/// let mut s = Segment(Pt(0,0), Pt(1,1));
/// s += Pt(1,0);
/// assert_eq!(s, Segment(Pt(1,0), Pt(2,1)));
/// ```
impl<T> AddAssign<Pt<T>> for Segment<T>
where
    T: Add<Output = T> + Copy,
{
    fn add_assign(&mut self, rhs: Pt<T>) {
        *self = Segment(self.i + rhs, self.f + rhs);
    }
}

/// A division operation between a segment and a point. This can be seen as
/// scaling by |rhs|.
///
/// ```
/// use plotz_geometry::{point::Pt, segment::Segment};
/// assert_eq!(
///       Segment(Pt(0.0,0.0), Pt(1.0,1.0))
///     / 2.0,  
///    // --------
///       Segment(Pt(0.0,0.0), Pt(0.5,0.5))
/// );
/// ```
impl<T> Div<T> for Segment<T>
where
    T: Float,
{
    type Output = Segment<T>;
    fn div(self, rhs: T) -> Self::Output {
        Segment(self.i / rhs, self.f / rhs)
    }
}

/// An division-assign operation between a segment and a point. This can be seen
/// as a scaling by |rhs|.
///
/// ```
/// use plotz_geometry::{point::Pt, segment::Segment};
/// let mut s = Segment(Pt(0.0,0.0), Pt(1.0,1.0));
/// s /= 2.0;
/// assert_eq!(s, Segment(Pt(0.0,0.0), Pt(0.5,0.5)));
/// ```
impl<T> DivAssign<T> for Segment<T>
where
    T: Float,
{
    fn div_assign(&mut self, rhs: T) {
        *self = Segment(self.i / rhs, self.f / rhs)
    }
}

/// A multiplication operation between a segment and a point. This can be seen
/// as scaling by |rhs|.
///
/// ```
/// use plotz_geometry::{point::Pt, segment::Segment};
/// assert_eq!(
///       Segment(Pt(0.0,0.0), Pt(1.0,1.0))
///     * 2.0,  
///    // --------
///       Segment(Pt(0.0,0.0), Pt(2.0,2.0))
/// );
/// ```
impl<T> Mul<T> for Segment<T>
where
    T: Float,
{
    type Output = Segment<T>;
    fn mul(self, rhs: T) -> Self::Output {
        Segment(self.i * rhs, self.f * rhs)
    }
}

/// An multiplication-assign operation between a segment and a point. This can
/// be seen as a scaling by |rhs|.
///
/// ```
/// use plotz_geometry::{point::Pt, segment::Segment};
/// let mut s = Segment(Pt(0.0,0.0), Pt(1.0,1.0));
/// s *= 2.0;
/// assert_eq!(s, Segment(Pt(0.0,0.0), Pt(2.0,2.0)));
/// ```
impl<T> MulAssign<T> for Segment<T>
where
    T: Float,
{
    fn mul_assign(&mut self, rhs: T) {
        *self = Segment(self.i * rhs, self.f * rhs);
    }
}

/// A subtraction operation between a segment and a point. This can be seen
/// as translation by |rhs|.
///
/// ```
/// use plotz_geometry::{point::Pt, segment::Segment};
/// assert_eq!(
///       Segment(Pt(0.0,0.0), Pt(1.0,1.0))
///     - Pt(1.0,2.0),
///    // --------
///       Segment(Pt(-1.0,-2.0), Pt(0.0,-1.0))
/// );
/// ```
impl<T> Sub<Pt<T>> for Segment<T>
where
    T: Float,
{
    type Output = Segment<T>;
    fn sub(self, rhs: Pt<T>) -> Self::Output {
        Segment {
            i: self.i - rhs,
            f: self.f - rhs,
        }
    }
}

/// An subtraction-assign operation between a segment and a point. This can
/// be seen as translation by |rhs|.
///
/// ```
/// use plotz_geometry::{point::Pt, segment::Segment};
/// let mut s = Segment(Pt(0.0,0.0), Pt(1.0,1.0));
/// s -= Pt(1.0,2.0);
/// assert_eq!(s, Segment(Pt(-1.0,-2.0), Pt(0.0,-1.0)));
/// ```
impl<T> SubAssign<Pt<T>> for Segment<T>
where
    T: Sub<Output = T> + Copy,
{
    fn sub_assign(&mut self, rhs: Pt<T>) {
        *self = Segment(self.i - rhs, self.f - rhs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ccw() {
        let a = Pt(0.0, 0.0);
        let b = Pt(1.0, 0.0);
        let c = Pt(1.0, 1.0);
        let d = Pt(0.0, 1.0);
        //   ^
        //   |
        //   D  C
        //   |
        // --A--B->
        //   |

        // True if three points are listed in counter-clockwise order:
        assert!(ccw(a, b, c));
        assert!(ccw(b, c, d));
        assert!(ccw(c, d, a));
        assert!(ccw(d, a, b));

        // False if three points listed are not in counter-clockwise order:
        assert!(!ccw(c, b, a));
        assert!(!ccw(b, a, d));
        assert!(!ccw(a, d, c));
        assert!(!ccw(d, c, b));
    }

    #[test]
    fn test_slope() {
        //   ^
        //   |
        //   A  B  C
        //   |
        //   D  E  F
        //   |
        // --G--H--I->
        //   |
        let a = Pt(0.0, 2.0);
        let b = Pt(1.0, 2.0);
        let c = Pt(2.0, 2.0);
        let d = Pt(0.0, 1.0);
        let e = Pt(1.0, 1.0);
        let f = Pt(2.0, 1.0);
        let g = Pt(0.0, 0.0);
        let h = Pt(1.0, 0.0);
        let i = Pt(2.0, 0.0);

        // m=0
        assert_eq!(Segment(g, h).slope(), 0.0);
        assert_eq!(Segment(g, i).slope(), 0.0);

        // m=0.5
        assert_eq!(Segment(g, f).slope(), 0.5);
        assert_eq!(Segment(d, c).slope(), 0.5);

        // m=1
        assert_eq!(Segment(g, e).slope(), 1.0);
        assert_eq!(Segment(g, c).slope(), 1.0);

        // m=2.0
        assert_eq!(Segment(h, c).slope(), 2.0);
        assert_eq!(Segment(g, b).slope(), 2.0);

        // m=inf
        assert_eq!(Segment(g, a).slope(), std::f64::INFINITY);
        assert_eq!(Segment(g, d).slope(), std::f64::INFINITY);

        // m=-0.5
        assert_eq!(Segment(a, f).slope(), -0.5);
        assert_eq!(Segment(d, i).slope(), -0.5);

        // m=-1.0
        assert_eq!(Segment(a, e).slope(), -1.0);
        assert_eq!(Segment(a, i).slope(), -1.0);

        // m=-2.0
        assert_eq!(Segment(b, i).slope(), -2.0);
        assert_eq!(Segment(a, h).slope(), -2.0);

        // m=-inf
        assert_eq!(Segment(a, g).slope(), -1.0 * std::f64::INFINITY);
        assert_eq!(Segment(d, g).slope(), -1.0 * std::f64::INFINITY);

        // slope is independent of start/end
        assert_eq!(Segment(a, c).slope(), Segment(c, a).slope());
        assert_eq!(Segment(a, f).slope(), Segment(f, a).slope());
        assert_eq!(Segment(a, i).slope(), Segment(i, a).slope());
        assert_eq!(Segment(a, h).slope(), Segment(h, a).slope());
    }

    #[test]
    fn test_rotate() {
        use float_eq::assert_float_eq;
        use std::f64::consts::PI;

        let origin = Pt(0.0, 0.0);

        //      ^
        //      |
        //      |  F
        // <----+--I->
        //      |
        //      |
        //      v
        let mut s = Segment(Pt(1.0, 0.0), Pt(1.0, 0.5));

        s.rotate(/*about=*/ &origin, PI / 2.0);
        //      ^
        //     FI
        //      |
        // <----+---->
        //      |
        //      |
        //      v
        assert_float_eq!(s.i.x, 0.0, abs <= 0.000_1);
        assert_float_eq!(s.i.y, 1.0, abs <= 0.000_1);
        assert_float_eq!(s.f.x, -0.5, abs <= 0.000_1);
        assert_float_eq!(s.f.y, 1.0, abs <= 0.000_1);

        s.rotate(/*about=*/ &origin, PI / 2.0);
        //      ^
        //      |
        //      |
        // <-I--+---->
        //   F  |
        //      |
        //      v
        assert_float_eq!(s.i.x, -1.0, abs <= 0.000_1);
        assert_float_eq!(s.i.y, 0.0, abs <= 0.000_1);
        assert_float_eq!(s.f.x, -1.0, abs <= 0.000_1);
        assert_float_eq!(s.f.y, -0.5, abs <= 0.000_1);

        s.rotate(/*about=*/ &origin, PI / 2.0);
        //      ^
        //      |
        //      |
        // <----+---->
        //      |
        //      IF
        //      v
        assert_float_eq!(s.i.x, 0.0, abs <= 0.000_1);
        assert_float_eq!(s.i.y, -1.0, abs <= 0.000_1);
        assert_float_eq!(s.f.x, 0.5, abs <= 0.000_1);
        assert_float_eq!(s.f.y, -1.0, abs <= 0.000_1);

        s.rotate(/*about=*/ &origin, PI / 2.0);
        //      ^
        //      |
        //      |  F
        // <----+--I->
        //      |
        //      |
        //      v
        assert_float_eq!(s.i.x, 1.0, abs <= 0.000_1);
        assert_float_eq!(s.i.y, 0.0, abs <= 0.000_1);
        assert_float_eq!(s.f.x, 1.0, abs <= 0.000_1);
        assert_float_eq!(s.f.y, 0.5, abs <= 0.000_1);
    }

    #[test]
    fn test_intersects() {
        //   ^
        //   |
        //   A  B  C
        //   |
        //   D  E  F
        //   |
        // --G--H--I->
        //   |
        let a = Pt(0.0, 2.0);
        let b = Pt(1.0, 2.0);
        let c = Pt(2.0, 2.0);
        let d = Pt(0.0, 1.0);
        let e = Pt(1.0, 1.0);
        let f = Pt(2.0, 1.0);
        let g = Pt(0.0, 0.0);
        let h = Pt(1.0, 0.0);
        let i = Pt(2.0, 0.0);

        // CG intersects AF, AH, and AI.
        assert!(Segment(a, f).intersects(&Segment(c, g)));
        assert!(Segment(a, h).intersects(&Segment(c, g)));
        assert!(Segment(a, i).intersects(&Segment(c, g)));

        // but CG does not intersect AB, AC, AD, AE, or AG
        assert!(!Segment(a, b).intersects(&Segment(c, g)));
        assert!(!Segment(a, c).intersects(&Segment(c, g)));
        assert!(!Segment(a, d).intersects(&Segment(c, g)));
        assert!(!Segment(a, e).intersects(&Segment(c, g)));
        assert!(!Segment(a, g).intersects(&Segment(c, g)));

        // Segments whose points overlap are not intersecting.
        // AB does not intersect BC.
        assert!(!Segment(a, b).intersects(&Segment(b, c)));
        // And AC does not intersect CI.
        assert!(!Segment(a, c).intersects(&Segment(c, i)));

        // A segment does intersect itself.
        assert!(!Segment(a, b).intersects(&Segment(a, b)));
        assert!(!Segment(a, b).intersects(&Segment(b, a)));

        // A segment does not intersect a parallel segment.
        assert!(!Segment(a, b).intersects(&Segment(a, c)));
    }
}
