//! A 3D point.
//!
use super::ray3::Ray3;
use crate::{
    bounded3::{Bounded3, Bounds3},
    shapes::segment3::Segment3,
    Rotatable,
};
use anyhow::{anyhow, Result};
use float_cmp::approx_eq;
use float_ord::FloatOrd;
use std::{
    convert::From,
    f64::consts::{PI, TAU},
    fmt::Debug,
    hash::Hash,
    ops::*,
};

#[derive(Copy, Clone)]
pub struct Point3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl PartialOrd for Point3 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Point3 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        FloatOrd(self.x)
            .cmp(&FloatOrd(other.x))
            .then(FloatOrd(self.y).cmp(&FloatOrd(other.y)))
            .then(FloatOrd(self.z).cmp(&FloatOrd(other.z)))
    }
}

impl Hash for Point3 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        FloatOrd(self.x).hash(state);
        FloatOrd(self.y).hash(state);
        FloatOrd(self.z).hash(state);
    }
}

#[allow(non_snake_case)]
pub fn Origin() -> Point3 {
    (0, 0, 0).into()
}

impl Debug for Point3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Point3 { x, y, z } = self;
        write!(f, "Pt3({:.1},{:.1},{:.1})", x, y, z)
    }
}

impl PartialEq for Point3 {
    fn eq(&self, other: &Self) -> bool {
        let e = 0.0000001;
        approx_eq!(f64, self.x, other.x, epsilon = e)
            && approx_eq!(f64, self.y, other.y, epsilon = e)
            && approx_eq!(f64, self.z, other.z, epsilon = e)
    }
}

impl Eq for Point3 {}

// An alternate constructor for points.
#[allow(non_snake_case)]
pub fn Point3<T1, T2, T3>(x: T1, y: T2, z: T3) -> Point3
where
    f64: From<T1>,
    f64: From<T2>,
    f64: From<T3>,
{
    Point3 {
        x: x.into(),
        y: y.into(),
        z: z.into(),
    }
}

impl<T1, T2, T3> From<(T1, T2, T3)> for Point3
where
    f64: From<T1>,
    f64: From<T2>,
    f64: From<T3>,
{
    fn from((x, y, z): (T1, T2, T3)) -> Point3 {
        Point3(x, y, z)
    }
}

impl<T> From<Point3> for (T, T, T)
where
    T: From<f64>,
{
    fn from(val: Point3) -> Self {
        (val.x.into(), val.y.into(), val.z.into())
    }
}

impl From<f64> for Point3 {
    fn from(n: f64) -> Self {
        (n, n, n).into()
    }
}

macro_rules! ops_trait {
    ($trait:ident, $fn:ident) => {
        impl<T> $trait<T> for Point3
        where
            T: Into<Point3>,
        {
            type Output = Self;
            fn $fn(self, rhs: T) -> Self::Output {
                let rhs = rhs.into();
                Point3(self.x.$fn(rhs.x), self.y.$fn(rhs.y), self.z.$fn(rhs.z))
            }
        }
    };
}

macro_rules! ops_mut_trait {
    ($trait:ident, $fn:ident) => {
        impl<T> $trait<T> for Point3
        where
            T: Into<Point3>,
        {
            fn $fn(&mut self, rhs: T) {
                let rhs = rhs.into();
                self.x.$fn(rhs.x);
                self.y.$fn(rhs.y);
                self.z.$fn(rhs.z);
            }
        }
    };
}

ops_mut_trait!(AddAssign, add_assign);
ops_mut_trait!(DivAssign, div_assign);
ops_mut_trait!(MulAssign, mul_assign);
ops_mut_trait!(RemAssign, rem_assign);
ops_mut_trait!(SubAssign, sub_assign);
ops_trait!(Add, add);
ops_trait!(Div, div);
ops_trait!(Mul, mul);
ops_trait!(Rem, rem);
ops_trait!(Sub, sub);

impl Point3 {
    // https://en.wikipedia.org/wiki/Dot_product
    pub fn dot(&self, other: &Point3) -> f64 {
        (self.x * other.x) + (self.y * other.y) + (self.z * other.z)
    }
    // average of two points.
    pub fn avg(&self, other: &Point3) -> Point3 {
        Point3(
            (self.x + other.x) / 2.0,
            (self.y + other.y) / 2.0,
            (self.z + other.z) / 2.0,
        )
    }

    /// Distance between two points.
    pub fn dist(&self, other: &Point3) -> f64 {
        Segment3(*self, *other).abs()
    }
}

#[allow(non_snake_case)]
pub fn PolarPoint3(r: f64, theta_rad: f64, phi_rad: f64) -> Result<Point3> {
    if !(0.0..=TAU).contains(&theta_rad) {
        return Err(anyhow!(format!(
            "theta_rad ({:?}) must be in range 0..=2PI",
            theta_rad
        )));
    }
    if !(0.0..=PI).contains(&phi_rad) {
        return Err(anyhow!(format!(
            "phi_rad ({:?}) must be in range 0..=PI",
            phi_rad
        )));
    }

    Ok(Point3(
        r * phi_rad.sin() * theta_rad.cos(),
        r * phi_rad.sin() * theta_rad.sin(),
        r * phi_rad.cos(),
    ))
}

impl Rotatable for Point3 {
    fn rotate(&self, by_rad: f64, about: Ray3) -> Result<Point3> {
        // https://en.wikipedia.org/wiki/Rotation_matrix#Rotation_matrix_from_axis_and_angle
        // R = [ ux ux (1 - cos t) +    cos t  ,  ux uy (1 - cos t) - uz sin t   ,  ux uz (1 - cos t) + uy sin t  ]
        // .   [ uy ux (1 - cos t) + uz sin t  ,  uy uy (1 - cos t) +    cos t   ,  uy uz (1 - cos t) - ux sin t  ]
        // .   [ uz ux (1 - cos t) - uy sin t  ,  uz uy (1 - cos t) + ux sin t   ,  uz uz (1 - cos t) +    cos t  ]
        // how to use? R * input = output;
        // in 3d, R = [ R00 R10 R20 ] [ A0 ] = [ B0 ]
        // .          [ R01 R11 R21 ] [ A1 ] = [ B1 ]
        // .          [ R02 R12 R22 ] [ A2 ] = [ B2 ]
        // so, R00*A0 + R10*A1 + R20*A2 ==> B0
        //   , R01*A0 + R11*A1 + R21*A2 ==> B1
        //   , R02*A0 + R12*A1 + R22*A2 ==> B2
        // B = [ B0, B1, B2 ]

        // just say no to rounding error accumulation
        let t = by_rad % TAU;

        let sg3 = about.to_sg3_with_len(1.0)?;
        let (ux, uy, uz): (f64, f64, f64) = (sg3.f - sg3.i).into();
        let cost: f64 = t.cos();
        let sint: f64 = t.sin();

        let (r00, r10, r20, r01, r11, r21, r02, r12, r22) = (
            ux * ux * (1.0 - cost) + cost,
            ux * uy * (1.0 - cost) - uz * sint,
            ux * uz * (1.0 - cost) + uy * sint,
            uy * ux * (1.0 - cost) + uz * sint,
            uy * uy * (1.0 - cost) + cost,
            uy * uz * (1.0 - cost) - ux * sint,
            uz * ux * (1.0 - cost) - uy * sint,
            uz * uy * (1.0 - cost) + ux * sint,
            uz * uz * (1.0 - cost) + cost,
        );

        let (a0, a1, a2): (f64, f64, f64) = (*self - sg3.f).into();

        Ok(Point3(
            /*b0=*/ r00 * a0 + r10 * a1 + r20 * a2,
            /*b1=*/ r01 * a0 + r11 * a1 + r21 * a2,
            /*b2=*/ r02 * a0 + r12 * a1 + r22 * a2,
        ) + sg3.f)
    }
    // foo
}

impl Bounded3 for Point3 {
    fn bounds3(&self) -> Result<Bounds3> {
        Ok(Bounds3 {
            x_min: self.x,
            x_max: self.x,
            y_min: self.y,
            y_max: self.y,
            z_min: self.z,
            z_max: self.z,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{f32::consts::FRAC_1_SQRT_2, f64::consts::FRAC_PI_2};
    use test_case::test_case;

    #[test_case(Point3(1,0,0), 0.0*FRAC_PI_2, Point3(1,0,0))]
    #[test_case(Point3(1,0,0), 0.5*FRAC_PI_2, Point3(FRAC_1_SQRT_2,FRAC_1_SQRT_2,0))]
    #[test_case(Point3(1,0,0), 1.0*FRAC_PI_2, Point3(0,1,0))]
    #[test_case(Point3(1,0,0), 1.5*FRAC_PI_2, Point3(-1.0*FRAC_1_SQRT_2,FRAC_1_SQRT_2,0))]
    #[test_case(Point3(1,0,0), 2.0*FRAC_PI_2, Point3(-1,0,0))]
    #[test_case(Point3(1,0,0), 2.5*FRAC_PI_2, Point3(-1.0*FRAC_1_SQRT_2,-1.0*FRAC_1_SQRT_2,0))]
    #[test_case(Point3(1,0,0), 3.0*FRAC_PI_2, Point3(0,-1,0))]
    #[test_case(Point3(1,0,0), 3.5*FRAC_PI_2, Point3(FRAC_1_SQRT_2,-1.0*FRAC_1_SQRT_2,0))]
    #[test_case(Point3(1,0,0), 4.0*FRAC_PI_2, Point3(1,0,0))]
    fn test_rotate_z_axis(input: Point3, by: f64, output: Point3) -> Result<()> {
        let z_axis = Ray3(Origin(), 0.0, 0.0)?;
        assert_eq!(input.rotate(by, z_axis)?, output);
        Ok(())
    }

    #[test_case(Point3(0,1,0), 0.0*FRAC_PI_2, Point3(0,1,0))]
    #[test_case(Point3(0,1,0), 1.0*FRAC_PI_2, Point3(0,0,1))]
    #[test_case(Point3(0,1,0), 2.0*FRAC_PI_2, Point3(0,-1,0))]
    #[test_case(Point3(0,1,0), 3.0*FRAC_PI_2, Point3(0,0,-1))]
    #[test_case(Point3(0,1,0), 4.0*FRAC_PI_2, Point3(0,1,0))]
    fn test_rotate_x_axis(input: Point3, by: f64, output: Point3) -> Result<()> {
        let x_axis = Ray3(Origin(), 0.0, FRAC_PI_2)?;
        assert_eq!(input.rotate(by, x_axis)?, output);
        Ok(())
    }
}
