use crate::point::Pt;
use float_cmp::{approx_eq, ApproxEq};
use num::Float;

fn interpolate_checked<T>(a: T, b: T, i: T) -> Result<f64, ()>
where
    T: Float + Copy,
    f64: From<T>,
{
    let v = (i - a) / (b - a);
    if v < T::zero() || v > T::one() {
        return Err(());
    }
    Ok(f64::from(v))
}

pub fn interpolate_2d_checked<T>(a: Pt<T>, b: Pt<T>, i: Pt<T>) -> Result<f64, ()>
where
    T: ApproxEq + Float + Copy,
    f64: From<T>,
{
    let x_same = approx_eq!(T, a.x, b.x);
    let y_same = approx_eq!(T, a.y, b.y);
    match (x_same, y_same) {
        (true, true) => Err(()),
        (false, true) => {
            let v_x = interpolate_checked(a.x, b.x, i.x)?;
            Ok(v_x)
        }
        (true, false) => {
            let v_y = interpolate_checked(a.y, b.y, i.y)?;
            Ok(v_y)
        }
        (false, false) => {
            let v_x = interpolate_checked(a.x, b.x, i.x)?;
            let v_y = interpolate_checked(a.y, b.y, i.y)?;
            match approx_eq!(f64, v_x, v_y) {
                true => Ok(v_x),
                false => Err(()),
            }
        }
    }
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
