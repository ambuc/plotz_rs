//! A general-purpose 3D geometry library.

#![allow(missing_docs)]

use anyhow::Result;
use shapes::ry3::Ry3;

pub mod camera;
pub mod group3;
pub mod obj3;
pub mod scene;
pub mod shapes;

pub trait Rotatable {
    fn rotate(&mut self, by: f64, about: Ry3) -> Result<()>;

    fn rotate_about_z_center(&mut self, by: f64) -> Result<()> {
        todo!("?")
    }
}
