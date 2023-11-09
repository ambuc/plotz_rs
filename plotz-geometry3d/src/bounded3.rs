use crate::{
    group3::Group3,
    shapes::{cuboid3d::Cuboid, pt3::Pt3},
};
use anyhow::{anyhow, Result};
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

    pub fn center(&self) -> Pt3 {
        Pt3(
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

pub struct Bounds3Collector {
    x_min: FloatOrd<f64>,
    y_min: FloatOrd<f64>,
    z_min: FloatOrd<f64>,
    x_max: FloatOrd<f64>,
    y_max: FloatOrd<f64>,
    z_max: FloatOrd<f64>,
    items_seen: usize,
}

impl Default for Bounds3Collector {
    fn default() -> Self {
        Bounds3Collector {
            x_max: FloatOrd(f64::MIN),
            x_min: FloatOrd(f64::MAX),
            y_max: FloatOrd(f64::MIN),
            y_min: FloatOrd(f64::MAX),
            z_max: FloatOrd(f64::MIN),
            z_min: FloatOrd(f64::MAX),
            items_seen: 0,
        }
    }
}

impl Bounds3Collector {
    pub fn items_seen(&self) -> usize {
        self.items_seen
    }
    pub fn incorporate(&mut self, bounds: &Bounds3) -> Result<()> {
        self.x_max = max(self.x_max, FloatOrd(bounds.x_max));
        self.x_min = min(self.x_min, FloatOrd(bounds.x_min));
        self.y_max = max(self.y_max, FloatOrd(bounds.y_max));
        self.y_min = min(self.y_min, FloatOrd(bounds.y_min));
        self.z_max = max(self.z_max, FloatOrd(bounds.z_max));
        self.z_min = min(self.z_min, FloatOrd(bounds.z_min));
        self.items_seen += 1;

        Ok(())
    }
}

impl Bounded3 for Bounds3Collector {
    fn bounds3(&self) -> Result<Bounds3> {
        Ok(Bounds3 {
            x_min: self.x_min.0,
            x_max: self.x_max.0,
            y_min: self.y_min.0,
            y_max: self.y_max.0,
            z_min: self.z_min.0,
            z_max: self.z_max.0,
        })
    }
}

pub fn streaming_bbox<'a>(
    it: impl IntoIterator<Item = &'a (impl Bounded3 + 'a)>,
) -> Result<Bounds3> {
    let mut bc = Bounds3Collector::default();
    for i in it {
        bc.incorporate(&i.bounds3()?)?;
    }
    if bc.items_seen == 0 {
        return Err(anyhow!("no items seen"));
    }
    bc.bounds3()
}
