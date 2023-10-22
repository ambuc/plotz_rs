use crate::{
    group3::Group3,
    shapes::{cuboid3d::Cuboid, pt3::Pt3},
};
use anyhow::Result;

#[derive(Debug, Copy, Clone)]
pub struct Bounds {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
    pub z_min: f64,
    pub z_max: f64,
}

impl Bounds {
    pub fn to_cuboid(&self) -> Group3<()> {
        Cuboid(
            (self.x_min, self.y_min, self.z_min),
            (self.x_span(), self.y_span(), self.z_span()),
        )
    }
    pub fn x_span(&self) -> f64 {
        self.x_max - self.x_min
    }
    pub fn y_span(&self) -> f64 {
        self.y_max - self.y_min
    }
    pub fn z_span(&self) -> f64 {
        self.z_max - self.z_min
    }
    pub fn center(&self) -> Pt3 {
        Pt3(
            self.x_min + (self.x_span()) / 2.0,
            self.y_min + (self.y_span()) / 2.0,
            self.z_min + (self.z_span()) / 2.0,
        )
    }
}
