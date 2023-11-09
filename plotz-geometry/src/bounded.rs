//! A trait representing the bounds and bounding box for an object.
#![allow(missing_docs)]

use std::cmp::{max, min};

use crate::{
    crop::PointLoc,
    shapes::{point::Point, polygon::Polygon},
};
use anyhow::Result;
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
    pub fn join(self, other: &Self) -> Self {
        Self {
            y_max: max(FloatOrd(self.y_max), FloatOrd(other.y_max)).0,
            x_max: max(FloatOrd(self.x_max), FloatOrd(other.x_max)).0,
            y_min: min(FloatOrd(self.y_min), FloatOrd(other.y_min)).0,
            x_min: min(FloatOrd(self.x_min), FloatOrd(other.x_min)).0,
        }
    }

    pub fn to_polygon(&self) -> Polygon {
        Polygon([
            self.x_min_y_max(),
            self.x_max_y_max(),
            self.x_max_y_min(),
            self.x_min_y_min(),
            self.x_min_y_max(),
        ])
        .unwrap()
    }

    pub fn contains_pt(&self, pt: Point) -> Result<PointLoc> {
        self.to_polygon().contains_pt(&pt)
    }

    pub fn x_span(&self) -> f64 {
        self.x_max - self.x_min
    }
    pub fn y_span(&self) -> f64 {
        self.y_min - self.y_max
    }
    pub fn x_min_y_max(&self) -> Point {
        Point(self.x_min, self.y_max)
    }
    pub fn x_max_y_max(&self) -> Point {
        Point(self.x_max, self.y_max)
    }
    pub fn x_min_y_min(&self) -> Point {
        Point(self.x_min, self.y_min)
    }
    pub fn x_max_y_min(&self) -> Point {
        Point(self.x_max, self.y_min)
    }

    pub fn center(&self) -> Point {
        Point(
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

pub fn streaming_bbox<'a>(it: impl IntoIterator<Item = &'a (impl Bounded + 'a)>) -> Result<Bounds> {
    it.into_iter().try_fold(
        Bounds {
            x_max: f64::MIN,
            x_min: f64::MAX,
            y_max: f64::MIN,
            y_min: f64::MAX,
        },
        |prev, x| {
            let b = x.bounds()?;
            Ok(prev.join(&b))
        },
    )
}

#[cfg(test)]
mod test_super {
    use super::*;
    use crate::shapes::point::Point;

    #[test]
    fn test_streaming_bbox() {
        let polygons = vec![
            Polygon([(0, 0), (1, 0), (1, 1)]).unwrap(),
            Polygon([(2, 0), (3, 0), (3, 1)]).unwrap(),
            Polygon([(0, 2), (1, 2), (1, 3)]).unwrap(),
        ];
        let bounds = streaming_bbox(&polygons).unwrap();
        assert_eq!(bounds.x_min_y_min(), Point(0, 0));
        assert_eq!(bounds.x_min_y_max(), Point(0, 3));
        assert_eq!(bounds.x_max_y_max(), Point(3, 3));
        assert_eq!(bounds.x_max_y_min(), Point(3, 0));
    }
}
