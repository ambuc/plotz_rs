use crate::{
    group3::Group3,
    shapes::{cuboid3d::Cuboid, point3::Point3},
};
use anyhow::Result;
use enum_dispatch::enum_dispatch;
use float_ord::FloatOrd;
use std::cmp::{max, min};

#[derive(Debug, Copy, Clone)]
pub struct Bounds3 {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
    pub z_min: f64,
    pub z_max: f64,
}

impl Bounds3 {
    pub fn join(self, other: &Self) -> Self {
        Self {
            z_max: max(FloatOrd(self.z_max), FloatOrd(other.z_max)).0,
            y_max: max(FloatOrd(self.y_max), FloatOrd(other.y_max)).0,
            x_max: max(FloatOrd(self.x_max), FloatOrd(other.x_max)).0,
            z_min: min(FloatOrd(self.z_min), FloatOrd(other.z_min)).0,
            y_min: min(FloatOrd(self.y_min), FloatOrd(other.y_min)).0,
            x_min: min(FloatOrd(self.x_min), FloatOrd(other.x_min)).0,
        }
    }

    pub fn to_cuboid(&self) -> Group3<()> {
        Cuboid(
            (self.x_min, self.y_min, self.z_min),
            (self.x_span(), self.y_span(), self.z_span()),
        )
    }

    // TODO(ambuc): contains_pt

    pub fn x_span(&self) -> f64 {
        self.x_max - self.x_min
    }
    pub fn y_span(&self) -> f64 {
        self.y_max - self.y_min
    }
    pub fn z_span(&self) -> f64 {
        self.z_max - self.z_min
    }

    pub fn center(&self) -> Point3 {
        Point3(
            self.x_min + (self.x_span()) / 2.0,
            self.y_min + (self.y_span()) / 2.0,
            self.z_min + (self.z_span()) / 2.0,
        )
    }
}

#[enum_dispatch(Obj3)]
pub trait Bounded3 {
    fn bounds3(&self) -> Result<Bounds3>;
}

pub fn streaming_bbox<'a>(
    it: impl IntoIterator<Item = &'a (impl Bounded3 + 'a)>,
) -> Result<Bounds3> {
    it.into_iter().try_fold(
        Bounds3 {
            x_max: f64::MIN,
            x_min: f64::MAX,
            y_max: f64::MIN,
            y_min: f64::MAX,
            z_max: f64::MIN,
            z_min: f64::MAX,
        },
        |prev, x| {
            let b = x.bounds3()?;
            Ok(prev.join(&b))
        },
    )
}
