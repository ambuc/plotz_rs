//! A 3D ray.

use crate::shapes::{point3::Pt3, segment3::Sg3};
use anyhow::{anyhow, Result};
use std::f64::consts::{PI, TAU};

use super::point3::PolarPt3;

/// A ray (3d) which emits from a Pt3 and goes in a direction (3d).
#[derive(Copy, Clone, Debug)]
pub struct Ry3 {
    pt3: Pt3,

    // theta and phi are "the mathematics convention" https://en.wikipedia.org/wiki/Spherical_coordinate_system.
    // theta represents the angle around the xy plane, starting from the positive x axis and rotating ccw around the z-axis.
    // 0 <= theta <= 2PI.
    theta_rad: f64,

    // phi represents the angle 'down' from the z-axis.
    // 0 <= phi <= PI.
    // you might choose to think of this as the 'zenith angle'.
    phi_rad: f64,
}

#[allow(non_snake_case)]
pub fn Ry3(pt3: Pt3, theta_rad: f64, phi_rad: f64) -> Result<Ry3> {
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
    Ok(Ry3 {
        pt3,
        theta_rad,
        phi_rad,
    })
}

impl Ry3 {
    pub fn to_sg3_with_len(&self, len: f64) -> Result<Sg3> {
        Ok(Sg3 {
            i: self.pt3,
            f: self.pt3 + PolarPt3(len, self.theta_rad, self.phi_rad)?,
        })
    }
}
