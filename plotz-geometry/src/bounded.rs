//! A trait representing the bounds and bounding box for an object.
#![allow(missing_docs)]

use crate::{
    crop::PointLoc,
    shapes::{pg::Pg, pt::Pt},
};
use anyhow::{anyhow, Result};
use enum_dispatch::enum_dispatch;
use float_ord::FloatOrd;

#[derive(Debug, Copy, Clone)]
pub struct Bounds {
    pub y_max: f64,
    pub y_min: f64,
    pub x_min: f64,
    pub x_max: f64,
}

impl Bounds {
    pub fn to_polygon(&self) -> Pg {
        Pg([
            self.x_min_y_max(),
            self.x_max_y_max(),
            self.x_max_y_min(),
            self.x_min_y_min(),
            self.x_min_y_max(),
        ])
        .unwrap()
    }

    pub fn contains_pt(&self, pt: Pt) -> Result<PointLoc> {
        self.to_polygon().contains_pt(&pt)
    }

    pub fn x_span(&self) -> f64 {
        self.x_max - self.x_min
    }
    pub fn y_span(&self) -> f64 {
        self.y_min - self.y_max
    }
    pub fn x_min_y_max(&self) -> Pt {
        Pt(self.x_min, self.y_max)
    }
    pub fn x_max_y_max(&self) -> Pt {
        Pt(self.x_max, self.y_max)
    }
    pub fn x_min_y_min(&self) -> Pt {
        Pt(self.x_min, self.y_min)
    }
    pub fn x_max_y_min(&self) -> Pt {
        Pt(self.x_max, self.y_min)
    }

    pub fn center(&self) -> Pt {
        Pt(
            self.x_min + (self.x_span() / 2.0),
            self.y_max + (self.y_span() / 2.0),
        )
    }
}

/// An object which is Bounded implements four cardinal bounds; the trait allows
/// a caller to discover the width, height, four corners, bounding box, and
/// center of that object.
///
/// Unlike most graphics systems, we assume that (0,0) is in the bottom-left.
#[enum_dispatch(Obj)]
pub trait Bounded {
    fn bounds(&self) -> Result<Bounds>;
}

#[derive(Default)]
pub struct BoundsCollector {
    y_max: Option<FloatOrd<f64>>,
    y_min: Option<FloatOrd<f64>>,
    x_min: Option<FloatOrd<f64>>,
    x_max: Option<FloatOrd<f64>>,
    items_seen: usize,
}

impl BoundsCollector {
    pub fn items_seen(&self) -> usize {
        self.items_seen
    }

    pub fn incorporate(&mut self, b: &impl Bounded) -> Result<()> {
        let bounds = b.bounds()?;
        self.y_max = Some(match self.y_max {
            None => FloatOrd(bounds.y_max),
            Some(existing) => std::cmp::max(existing, FloatOrd(bounds.y_max)),
        });
        self.y_min = Some(match self.y_min {
            None => FloatOrd(bounds.y_min),
            Some(existing) => std::cmp::min(existing, FloatOrd(bounds.y_min)),
        });
        self.x_max = Some(match self.x_max {
            None => FloatOrd(bounds.x_max),
            Some(existing) => std::cmp::max(existing, FloatOrd(bounds.x_max)),
        });
        self.x_min = Some(match self.x_min {
            None => FloatOrd(bounds.x_min),
            Some(existing) => std::cmp::min(existing, FloatOrd(bounds.x_min)),
        });
        self.items_seen += 1;
        Ok(())
    }
}

impl Bounded for BoundsCollector {
    fn bounds(&self) -> Result<Bounds> {
        Ok(Bounds {
            y_max: self.y_max.ok_or(anyhow!("absent"))?.0,
            y_min: self.y_min.ok_or(anyhow!("absent"))?.0,
            x_min: self.x_min.ok_or(anyhow!("absent"))?.0,
            x_max: self.x_max.ok_or(anyhow!("absent"))?.0,
        })
    }
}

/// Given an iterator of bounded items, computes the bounding box for that
/// collection.
pub fn streaming_bbox<'a>(it: impl IntoIterator<Item = &'a (impl Bounded + 'a)>) -> Result<Bounds> {
    let mut bc = BoundsCollector::default();
    for i in it {
        bc.incorporate(i)?;
    }
    if bc.items_seen == 0 {
        return Err(anyhow!("no items seen"));
    }
    bc.bounds()
}

#[cfg(test)]
mod test_super {
    use super::*;
    use crate::shapes::pt::Pt;

    #[test]
    fn test_streaming_bbox() {
        let polygons = vec![
            Pg([(0, 0), (1, 0), (1, 1)]).unwrap(),
            Pg([(2, 0), (3, 0), (3, 1)]).unwrap(),
            Pg([(0, 2), (1, 2), (1, 3)]).unwrap(),
        ];
        let bounds = streaming_bbox(&polygons).unwrap();
        assert_eq!(bounds.x_min_y_min(), Pt(0, 0));
        assert_eq!(bounds.x_min_y_max(), Pt(0, 3));
        assert_eq!(bounds.x_max_y_max(), Pt(3, 3));
        assert_eq!(bounds.x_max_y_min(), Pt(3, 0));
    }
}
