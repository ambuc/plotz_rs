//! A 3D point.
//!
use crate::{
    bounded3::{Bounded3, Bounds3},
    shapes::sg3::Sg3,
    Rotatable,
};
use anyhow::{anyhow, Result};
use float_ord::FloatOrd;
use std::{
    convert::From,
    f64::consts::{PI, TAU},
    fmt::Debug,
    hash::Hash,
    ops::*,
};

use super::ry3::Ry3;

#[derive(Hash, Copy, Clone, PartialOrd, PartialEq, Eq, Ord)]
pub struct Pt3 {
    pub x: FloatOrd<f64>,
    pub y: FloatOrd<f64>,
    pub z: FloatOrd<f64>,
}

impl Debug for Pt3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Pt3 { x, y, z } = self;
        write!(f, "p3!({:.2},{:.2},{:.2})", x.0, y.0, z.0)
    }
}

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
        r * theta_rad.sin() * phi_rad.cos(),
        r * theta_rad.sin() * phi_rad.sin(),
        r * theta_rad.cos(),
    ))
}

impl Rotatable for Pt3 {
    fn rotate(&self, by_rad: f64, about: Ry3) -> Result<Pt3> {
        // https://en.wikipedia.org/wiki/Rotation_matrix#Rotation_matrix_from_axis_and_angle
        // R = [ ux ux (1 - cos t) +    cos t  ,  ux uy (1 - cos t) - uz sin t   ,  ux uz (1 - cos t) + uy sin t  ]
        // .   [ uy ux (1 - cos t) + uz sin t  ,  uy uy (1 - cos t) +    cos t   ,  uy uz (1 - cos t) + ux sin t  ]
        // .   [ uz ux (1 - cos t) + uy sin t  ,  uz uy (1 - cos t) + ux sin t   ,  uz uz (1 - cos t) +    cos t  ]
        // how to use? R * input = output;
        // in 3d, R = [ R00 R10 R20 ] [ I0 ] = [ O0 ]

        // .          [ R01 R11 R21 ] [ I1 ] = [ O1 ]
        // .          [ R02 R12 R22 ] [ I2 ] = [ O2 ]
        // so, R00*I0 + R10*I1 + R20*I2 ==> O0
        //   , R01*I0 + R11*I1 + R21*I2 ==> O1
        //   , R02*I0 + R12*I1 + R22*I2 ==> O2
        // O = [ O0, O1, O2 ]

        let sg3 = about.to_sg3(1.0)?;
        let (ux, uy, uz) = (sg3.f.x.0, sg3.f.y.0, sg3.f.z.0);
        let cost: f64 = by_rad.cos();
        let sint: f64 = by_rad.sin();
        let omct: f64 = 1.0 - cost;

        let (r00, r10, r20): (f64, f64, f64) = (
            ux * ux * omct + cost,
            ux * uy * omct - uz * sint,
            ux * uz * omct + uy * sint,
        );
        let (r01, r11, r21): (f64, f64, f64) = (
            uy * ux * omct + uz * sint,
            uy * uy * omct + cost,
            uy * uz * omct + ux * sint,
        );
        let (r02, r12, r22): (f64, f64, f64) = (
            uz * ux * omct + uy * sint,
            uz * uy * omct + ux * sint,
            uz * uz * omct + cost,
        );
        let (i0, i1, i2): (f64, f64, f64) = (self.x.0, self.y.0, self.z.0);

        let (o0, o1, o2): (f64, f64, f64) = (
            r00 * i0 + r10 * i1 + r20 * i2,
            r01 * i0 + r11 * i1 + r21 * i2,
            r02 * i0 + r12 * i1 + r22 * i2,
        );

        Ok(Pt3(o0, o1, o2))
    }
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
