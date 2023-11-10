//! General 1D and 2D interpolation and extrapolation algorithms.
use crate::{shapes::point::Point, utils::Percent};
use anyhow::{anyhow, Result};
use float_cmp::approx_eq;

// Given bounding values |a| and |b|, and an intermediate value |i| which is
// within |a..b|, return the percent along |ab| which |i| lays.
fn interpolate_checked(a: f64, b: f64, i: f64) -> Result<f64> {
    let v = (i - a) / (b - a);
    if v < 0_f64 {
        return Err(anyhow!("below zero"));
    }
    if v > 1_f64 {
        return Err(anyhow!("above one"));
    }
    Ok(v)
}

/// Given the line |ab| defined by points |a| and |b|, and another point |i|
/// which lies along it, return the percent along |ab| which |i| lies.
pub fn interpolate_2d_checked(a: Point, b: Point, i: Point) -> Result<Percent> {
    let x_same = approx_eq!(f64, a.x, b.x);
    let y_same = approx_eq!(f64, a.y, b.y);
    match (x_same, y_same) {
        (true, true) => Err(anyhow!("points are the same")),
        (false, true) => {
            let v_x = interpolate_checked(a.x, b.x, i.x)?;
            Ok(Percent::new(v_x)?)
        }
        (true, false) => {
            let v_y = interpolate_checked(a.y, b.y, i.y)?;
            Ok(Percent::new(v_y)?)
        }
        (false, false) => {
            let v_x = interpolate_checked(a.x, b.x, i.x)?;
            let v_y = interpolate_checked(a.y, b.y, i.y)?;
            match approx_eq!(f64, v_x, v_y, epsilon = 0.0003) {
                true => Ok(Percent::new(v_x)?),
                false => Err(anyhow!("point not on line")),
            }
        }
    }
}

/// Given the line |ab| defined by points |a| and |b| and a percentage |p|,
/// return the interpolated point which lies at a fraction of |i| along |ab|.
pub fn extrapolate_2d(a: Point, b: Point, p: f64) -> Point {
    a + ((b - a) * p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolate_checked() -> Result<()> {
        assert!(interpolate_checked(0.0, 1.0, -0.1).is_err());
        assert_eq!(interpolate_checked(0.0, 1.0, 0.0)?, 0.0);
        assert_eq!(interpolate_checked(0.0, 1.0, 0.5)?, 0.5);
        assert_eq!(interpolate_checked(0.0, 1.0, 1.0)?, 1.0);
        assert!(interpolate_checked(0.0, 1.0, 1.1).is_err());
        Ok(())
    }

    #[test]
    fn test_interpolate_2d_checked() -> Result<()> {
        assert!(interpolate_2d_checked(Point(0, 0), Point(1, 1), Point(-0.1, -0.1)).is_err());
        assert_eq!(
            interpolate_2d_checked(Point(0, 0), Point(1, 1), Point(0, 0))?,
            Percent::Zero,
        );
        assert_eq!(
            interpolate_2d_checked(Point(0, 0), Point(1, 1), Point(0.5, 0.5))?,
            Percent::Val(0.5),
        );
        assert_eq!(
            interpolate_2d_checked(Point(0, 0), Point(1, 1), Point(1, 1))?,
            Percent::One,
        );
        assert!(interpolate_2d_checked(Point(0, 0), Point(1, 1), Point(1.1, 1.1)).is_err());

        // not on line
        assert!(interpolate_2d_checked(Point(0, 0), Point(1, 1), Point(1, 0)).is_err());
        assert!(interpolate_2d_checked(Point(0, 0), Point(1, 1), Point(0, 1)).is_err());
        Ok(())
    }
}
