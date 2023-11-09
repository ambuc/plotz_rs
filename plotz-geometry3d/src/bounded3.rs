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

#[derive(Default)]
pub struct Bounds3Collector {
    min_x: Option<FloatOrd<f64>>,
    min_y: Option<FloatOrd<f64>>,
    min_z: Option<FloatOrd<f64>>,
    max_x: Option<FloatOrd<f64>>,
    max_y: Option<FloatOrd<f64>>,
    max_z: Option<FloatOrd<f64>>,
    items_seen: usize,
}

impl Bounds3Collector {
    pub fn items_seen(&self) -> usize {
        self.items_seen
    }
    pub fn incorporate(&mut self, b: &Bounds3) -> Result<()> {
        let Bounds3 {
            x_min,
            x_max,
            y_min,
            y_max,
            z_min,
            z_max,
        } = b;

        self.min_x = Some(match self.min_x {
            None => FloatOrd(*x_min),
            Some(e) => min(e, FloatOrd(*x_min)),
        });
        self.min_y = Some(match self.min_y {
            None => FloatOrd(*y_min),
            Some(e) => min(e, FloatOrd(*y_min)),
        });
        self.min_z = Some(match self.min_z {
            None => FloatOrd(*z_min),
            Some(e) => min(e, FloatOrd(*z_min)),
        });
        self.max_x = Some(match self.max_x {
            None => FloatOrd(*x_max),
            Some(e) => max(e, FloatOrd(*x_max)),
        });
        self.max_y = Some(match self.max_y {
            None => FloatOrd(*y_max),
            Some(e) => max(e, FloatOrd(*y_max)),
        });
        self.max_z = Some(match self.max_z {
            None => FloatOrd(*z_max),
            Some(e) => max(e, FloatOrd(*z_max)),
        });
        self.items_seen += 1;

        Ok(())
    }
}

impl Bounded3 for Bounds3Collector {
    fn bounds3(&self) -> Result<Bounds3> {
        Ok(Bounds3 {
            x_min: self.min_x.ok_or(anyhow!("x_min is absent"))?.0,
            x_max: self.max_x.ok_or(anyhow!("x_max is absent"))?.0,
            y_min: self.min_y.ok_or(anyhow!("y_min is absent"))?.0,
            y_max: self.max_y.ok_or(anyhow!("y_max is absent"))?.0,
            z_min: self.min_z.ok_or(anyhow!("z_min is absent"))?.0,
            z_max: self.max_z.ok_or(anyhow!("z_max is absent"))?.0,
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
