//! General 1D and 2D interpolation and extrapolation algorithms.
use crate::point::Pt;
use float_cmp::approx_eq;

/// A general error arising from trying to interpolate a value some percentage
/// between two other values.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum InterpolationError {
    /// the resultant percentage was below zero. (Should have been between zero and one.)
    #[error("the resultant percentage was below zero. (Should have been between zero and one.)")]
    BelowZero,
    /// the resultant percentage was above one. (Should have been between zero and one.)
    #[error("the resultant percentage was above one. (Should have been between zero and one.)")]
    AboveOne,
}

// Given bounding values |a| and |b|, and an intermediate value |i| which is
// within |a..b|, return the percent along |ab| which |i| lays.
fn interpolate_checked(a: f64, b: f64, i: f64) -> Result<f64, InterpolationError> {
    let v = (i - a) / (b - a);
    if v < 0_f64 {
        return Err(InterpolationError::BelowZero);
    }
    if v > 1_f64 {
        return Err(InterpolationError::AboveOne);
    }
    Ok(v)
}

/// A general error arising from trying to interpolate a point some percentage
/// between two other points.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Interpolation2dError {
    /// Point |i| lies on |ab| but is either too small (behind |a|) or too large (beyond |b|).
    #[error(
        "Point |i| lies on |ab| but is either too small (behind |a|) or too large (beyond |b|)."
    )]
    RangeError(#[from] InterpolationError),
    /// Points |a| and |b| are the same, so interpolation cannot be performed.
    #[error("Points |a| and |b| are the same, so interpolation cannot be performed.")]
    PointsSame,
    /// Point |i| does not lie on the line |ab|.
    #[error("Point |i| does not lie on the line |ab|.")]
    PointNotOnLine,
}

/// Given the line |ab| defined by points |a| and |b|, and another point |i|
/// which lies along it, return the percent along |ab| which |i| lies.
pub fn interpolate_2d_checked(a: Pt, b: Pt, i: Pt) -> Result<f64, Interpolation2dError> {
    let x_same = approx_eq!(f64, a.x.0, b.x.0);
    let y_same = approx_eq!(f64, a.y.0, b.y.0);
    match (x_same, y_same) {
        (true, true) => Err(Interpolation2dError::PointsSame),
        (false, true) => {
            let v_x = interpolate_checked(a.x.0, b.x.0, i.x.0)?;
            Ok(v_x)
        }
        (true, false) => {
            let v_y = interpolate_checked(a.y.0, b.y.0, i.y.0)?;
            Ok(v_y)
        }
        (false, false) => {
            let v_x = interpolate_checked(a.x.0, b.x.0, i.x.0)?;
            let v_y = interpolate_checked(a.y.0, b.y.0, i.y.0)?;
            match approx_eq!(f64, v_x, v_y) {
                true => Ok(v_x),
                false => Err(Interpolation2dError::PointNotOnLine),
            }
        }
    }
}

/// Given the line |ab| defined by points |a| and |b| and a percentage |p|,
/// return the interpolated point which lies at a fraction of |i| along |ab|.
pub fn extrapolate_2d(a: Pt, b: Pt, p: f64) -> Pt {
    a + ((b - a) * p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolate_checked() {
        assert!(interpolate_checked(0.0, 1.0, -0.1).is_err());
        assert_eq!(interpolate_checked(0.0, 1.0, 0.0), Ok(0.0));
        assert_eq!(interpolate_checked(0.0, 1.0, 0.5), Ok(0.5));
        assert_eq!(interpolate_checked(0.0, 1.0, 1.0), Ok(1.0));
        assert!(interpolate_checked(0.0, 1.0, 1.1).is_err());
    }

    #[test]
    fn test_interpolate_2d_checked() {
        assert!(interpolate_2d_checked(Pt(0.0, 0.0), Pt(1.0, 1.0), Pt(-0.1, -0.1)).is_err());
        assert_eq!(
            interpolate_2d_checked(Pt(0.0, 0.0), Pt(1.0, 1.0), Pt(0.0, 0.0)),
            Ok(0.0)
        );
        assert_eq!(
            interpolate_2d_checked(Pt(0.0, 0.0), Pt(1.0, 1.0), Pt(0.5, 0.5)),
            Ok(0.5)
        );
        assert_eq!(
            interpolate_2d_checked(Pt(0.0, 0.0), Pt(1.0, 1.0), Pt(1.0, 1.0)),
            Ok(1.0)
        );
        assert!(interpolate_2d_checked(Pt(0.0, 0.0), Pt(1.0, 1.0), Pt(1.1, 1.1)).is_err());

        // not on line
        assert!(interpolate_2d_checked(Pt(0.0, 0.0), Pt(1.0, 1.0), Pt(1.0, 0.0)).is_err());
        assert!(interpolate_2d_checked(Pt(0.0, 0.0), Pt(1.0, 1.0), Pt(0.0, 1.0)).is_err());
    }
}
