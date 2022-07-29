use crate::point::Pt;
use float_cmp::approx_eq;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum InterpolationError {
    #[error("i < a")]
    BelowZero,
    #[error("i > b")]
    AboveOne,
}

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

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Interpolation2dError {
    #[error("value out of range")]
    RangeError(#[from] InterpolationError),
    #[error("a and b are the same")]
    PointsSame,
    #[error("i is not on line a->b")]
    PointNotOnLine,
}

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
