//! A general-purpose 3D geometry library.

#![allow(missing_docs)]

use anyhow::Result;
use bounded3::Bounded3;
use shapes::ry3::Ry3;

pub mod bounded3;
pub mod camera;
pub mod group3;
pub mod obj3;
pub mod scene;
pub mod shapes;

pub trait Rotatable
where
    Self: Sized,
{
    fn rotate(&self, by: f64, about: Ry3) -> Result<Self>;
}

pub trait RotatableBounds: Bounded3 + Rotatable {
    fn rotate_about_center_z_axis(&self, by: f64) -> Result<Self> {
        // theta_rad 0, phi_rad 0, straight up about z-axis;
        self.rotate(by, Ry3(self.bounds3()?.center(), 0.0, 0.0)?)
    }
}
