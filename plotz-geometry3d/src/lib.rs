//! A general-purpose 3D geometry library.

#![allow(missing_docs)]

use crate::{
    obj3::{Obj3, ObjType},
    shapes::{pt3::Pt3, ry3::Ry3},
};
use anyhow::Result;
use bounded3::Bounded3;
use enum_dispatch::enum_dispatch;
use std::f64::consts::FRAC_PI_2;

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
    fn rotate_about_center_x_axis(&self, by: f64) -> Result<Self> {
        // theta_rad 0, phi_rad PI/2, along x axis
        self.rotate(by, Ry3(self.bounds3()?.center(), 0.0, FRAC_PI_2)?)
    }
    fn rotate_about_center_y_axis(&self, by: f64) -> Result<Self> {
        // theta_rad PI/2, phi_rad PI/2, along y axis
        self.rotate(by, Ry3(self.bounds3()?.center(), FRAC_PI_2, FRAC_PI_2)?)
    }
    fn rotate_about_center_z_axis(&self, by: f64) -> Result<Self> {
        // theta_rad 0, phi_rad 0, straight up about z-axis;
        self.rotate(by, Ry3(self.bounds3()?.center(), 0.0, 0.0)?)
    }
}

/// A 3d object.
#[enum_dispatch(Obj3)]
pub trait Object {
    /// Is it empty?
    fn is_empty(&self) -> bool {
        self.iter().count() == 0
    }

    /// What type of object is this?
    fn objtype(&self) -> ObjType;

    /// Iterator
    fn iter(&self) -> Box<dyn Iterator<Item = &Pt3> + '_>;

    /// Mutable iterator
    fn iter_mut(&mut self) -> Box<dyn Iterator<Item = &mut Pt3> + '_>;
}
