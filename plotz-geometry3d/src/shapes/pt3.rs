//! A 3D point.
//!
use super::ry3::Ry3;
use crate::{
    bounded3::{Bounded3, Bounds3},
    shapes::sg3::Sg3,
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

#[derive(Hash, Copy, Clone, PartialOrd, Ord)]
pub struct Pt3 {
    pub x: FloatOrd<f64>,
    pub y: FloatOrd<f64>,
    pub z: FloatOrd<f64>,
}

#[allow(non_snake_case)]
pub fn Origin() -> Pt3 {
    (0, 0, 0).into()
}

impl Debug for Pt3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Pt3 { x, y, z } = self;
        write!(f, "Pt3({:.20},{:.20},{:.20})", x.0, y.0, z.0)
    }
}

impl PartialEq for Pt3 {
    fn eq(&self, other: &Self) -> bool {
        let e = 0.0000001;
        approx_eq!(f64, self.x.0, other.x.0, epsilon = e)
            && approx_eq!(f64, self.y.0, other.y.0, epsilon = e)
            && approx_eq!(f64, self.z.0, other.z.0, epsilon = e)
    }
}

impl Eq for Pt3 {}

// An alternate constructor for points.
#[allow(non_snake_case)]
pub fn Pt3<T1, T2, T3>(x: T1, y: T2, z: T3) -> Pt3
where
    f64: From<T1>,
    f64: From<T2>,
    f64: From<T3>,
{
    Pt3 {
        x: FloatOrd(x.into()),
        y: FloatOrd(y.into()),
        z: FloatOrd(z.into()),
    }
}

impl<T1, T2, T3> From<(T1, T2, T3)> for Pt3
where
    f64: From<T1>,
    f64: From<T2>,
    f64: From<T3>,
{
    fn from((x, y, z): (T1, T2, T3)) -> Pt3 {
        Pt3(x, y, z)
    }
}

impl<T> Into<(T, T, T)> for Pt3
where
    T: From<f64>,
{
    fn into(self) -> (T, T, T) {
        (self.x.0.into(), self.y.0.into(), self.z.0.into())
    }
}

impl Add<Pt3> for Pt3 {
    type Output = Self;
    fn add(self, rhs: Pt3) -> Self::Output {
        Pt3(self.x.0 + rhs.x.0, self.y.0 + rhs.y.0, self.z.0 + rhs.z.0)
    }
}
impl AddAssign<Pt3> for Pt3 {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: FloatOrd(self.x.0 + other.x.0),
            y: FloatOrd(self.y.0 + other.y.0),
            z: FloatOrd(self.z.0 + other.z.0),
        };
    }
}

impl Div<Pt3> for Pt3 {
    type Output = Self;
    fn div(self, rhs: Pt3) -> Self::Output {
        Pt3(self.x.0 / rhs.x.0, self.y.0 / rhs.y.0, self.z.0 / rhs.z.0)
    }
}
impl Div<f64> for Pt3 {
    type Output = Self;
    fn div(self, rhs: f64) -> Self::Output {
        Pt3(self.x.0 / rhs, self.y.0 / rhs, self.z.0 / rhs)
    }
}
impl DivAssign<Pt3> for Pt3 {
    fn div_assign(&mut self, rhs: Pt3) {
        self.x.0 /= rhs.x.0;
        self.y.0 /= rhs.y.0;
        self.z.0 /= rhs.z.0;
    }
}
impl DivAssign<f64> for Pt3 {
    fn div_assign(&mut self, rhs: f64) {
        self.x.0 /= rhs;
        self.y.0 /= rhs;
        self.z.0 /= rhs;
    }
}
impl Mul<Pt3> for Pt3 {
    type Output = Self;
    fn mul(self, rhs: Pt3) -> Self::Output {
        Pt3(self.x.0 * rhs.x.0, self.y.0 * rhs.y.0, self.z.0 * rhs.z.0)
    }
}
impl Mul<f64> for Pt3 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Pt3(self.x.0 * rhs, self.y.0 * rhs, self.z.0 * rhs)
    }
}
impl MulAssign<Pt3> for Pt3 {
    fn mul_assign(&mut self, rhs: Pt3) {
        self.x.0 *= rhs.x.0;
        self.y.0 *= rhs.y.0;
        self.z.0 *= rhs.z.0;
    }
}
impl MulAssign<f64> for Pt3 {
    fn mul_assign(&mut self, rhs: f64) {
        self.x.0 *= rhs;
        self.y.0 *= rhs;
        self.z.0 *= rhs;
    }
}
impl Sub<Pt3> for Pt3 {
    type Output = Self;
    fn sub(self, rhs: Pt3) -> Self::Output {
        Pt3(self.x.0 - rhs.x.0, self.y.0 - rhs.y.0, self.z.0 - rhs.z.0)
    }
}
impl SubAssign<Pt3> for Pt3 {
    fn sub_assign(&mut self, other: Self) {
        *self = Self {
            x: FloatOrd(self.x.0 - other.x.0),
            y: FloatOrd(self.y.0 - other.y.0),
            z: FloatOrd(self.z.0 - other.z.0),
        };
    }
}

impl Pt3 {
    // https://en.wikipedia.org/wiki/Dot_product
    pub fn dot(&self, other: &Pt3) -> f64 {
        (self.x.0 * other.x.0) + (self.y.0 * other.y.0) + (self.z.0 * other.z.0)
    }
    // average of two points.
    pub fn avg(&self, other: &Pt3) -> Pt3 {
        let avg_x = (self.x.0 + other.x.0) / 2.0;
        let avg_y = (self.y.0 + other.y.0) / 2.0;
        let avg_z = (self.z.0 + other.z.0) / 2.0;
        Pt3(avg_x, avg_y, avg_z)
    }

    /// Distance between two points.
    pub fn dist(&self, other: &Pt3) -> f64 {
        Sg3(*self, *other).abs()
    }
}

#[allow(non_snake_case)]
pub fn PolarPt3(r: f64, theta_rad: f64, phi_rad: f64) -> Result<Pt3> {
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

    Ok(Pt3(
        r * phi_rad.sin() * theta_rad.cos(),
        r * phi_rad.sin() * theta_rad.sin(),
        r * phi_rad.cos(),
    ))
}

impl Rotatable for Pt3 {
    fn rotate(&self, by_rad: f64, about: Ry3) -> Result<Pt3> {
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

        Ok(Pt3(
            /*b0=*/ r00 * a0 + r10 * a1 + r20 * a2,
            /*b1=*/ r01 * a0 + r11 * a1 + r21 * a2,
            /*b2=*/ r02 * a0 + r12 * a1 + r22 * a2,
        ) + sg3.f)
    }
    // foo
}

impl Bounded3 for Pt3 {
    fn bounds3(&self) -> Result<Bounds3> {
        Ok(Bounds3 {
            x_min: self.x.0,
            x_max: self.x.0,
            y_min: self.y.0,
            y_max: self.y.0,
            z_min: self.z.0,
            z_max: self.z.0,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{f32::consts::FRAC_1_SQRT_2, f64::consts::FRAC_PI_2};
    use test_case::test_case;

    #[test_case(Pt3(1,0,0), 0.0*FRAC_PI_2, Pt3(1,0,0))]
    #[test_case(Pt3(1,0,0), 0.5*FRAC_PI_2, Pt3(FRAC_1_SQRT_2,FRAC_1_SQRT_2,0))]
    #[test_case(Pt3(1,0,0), 1.0*FRAC_PI_2, Pt3(0,1,0))]
    #[test_case(Pt3(1,0,0), 1.5*FRAC_PI_2, Pt3(-1.0*FRAC_1_SQRT_2,FRAC_1_SQRT_2,0))]
    #[test_case(Pt3(1,0,0), 2.0*FRAC_PI_2, Pt3(-1,0,0))]
    #[test_case(Pt3(1,0,0), 2.5*FRAC_PI_2, Pt3(-1.0*FRAC_1_SQRT_2,-1.0*FRAC_1_SQRT_2,0))]
    #[test_case(Pt3(1,0,0), 3.0*FRAC_PI_2, Pt3(0,-1,0))]
    #[test_case(Pt3(1,0,0), 3.5*FRAC_PI_2, Pt3(FRAC_1_SQRT_2,-1.0*FRAC_1_SQRT_2,0))]
    #[test_case(Pt3(1,0,0), 4.0*FRAC_PI_2, Pt3(1,0,0))]
    fn test_rotate_z_axis(input: Pt3, by: f64, output: Pt3) -> Result<()> {
        let z_axis = Ry3(Origin(), 0.0, 0.0)?;
        assert_eq!(input.rotate(by, z_axis)?, output);
        Ok(())
    }

    #[test_case(Pt3(0,1,0), 0.0*FRAC_PI_2, Pt3(0,1,0))]
    #[test_case(Pt3(0,1,0), 1.0*FRAC_PI_2, Pt3(0,0,1))]
    #[test_case(Pt3(0,1,0), 2.0*FRAC_PI_2, Pt3(0,-1,0))]
    #[test_case(Pt3(0,1,0), 3.0*FRAC_PI_2, Pt3(0,0,-1))]
    #[test_case(Pt3(0,1,0), 4.0*FRAC_PI_2, Pt3(0,1,0))]
    fn test_rotate_x_axis(input: Pt3, by: f64, output: Pt3) -> Result<()> {
        let x_axis = Ry3(Origin(), 0.0, FRAC_PI_2)?;
        assert_eq!(input.rotate(by, x_axis)?, output);
        Ok(())
    }
}
