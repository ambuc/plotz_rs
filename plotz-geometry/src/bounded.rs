//! A trait representing the bounds and bounding box for an object.
use crate::{
    crop::PointLoc,
    shapes::{pg::Pg, pt::Pt},
};
use anyhow::{anyhow, Result};
use enum_dispatch::enum_dispatch;
use float_ord::FloatOrd;

/// The bounds of a geometric object.
#[derive(Debug, Copy, Clone)]
pub struct Bounds {
    /// Top bound.
    pub top_bound: f64,
    /// Bottom bound.
    pub bottom_bound: f64,
    /// Left bound.
    pub left_bound: f64,
    /// Right bound.
    pub right_bound: f64,
}

impl Bounds {
    /// Creates a frame, suitable for cropping.
    pub fn to_polygon(&self) -> Pg {
        Pg([self.tl(), self.tr(), self.br(), self.bl(), self.tl()])
    }
    /// Whether or not bounds contain a point.
    pub fn contains_pt(&self, pt: Pt) -> Result<PointLoc> {
        self.to_polygon().contains_pt(&pt)
    }
    /// The right bound of an object.
    pub fn r(&self) -> f64 {
        self.right_bound
    }
    /// The left bound of an object.
    pub fn l(&self) -> f64 {
        self.left_bound
    }
    /// The top bound of an object.
    pub fn t(&self) -> f64 {
        self.top_bound
    }
    /// The bottom bound of an object.
    pub fn b(&self) -> f64 {
        self.bottom_bound
    }
    /// The width of an object.
    pub fn w(&self) -> f64 {
        self.r() - self.l()
    }
    /// The height of an object.
    pub fn h(&self) -> f64 {
        self.b() - self.t()
    }
    /// The point at the top-left corner of an object's bounding box.
    pub fn tl(&self) -> Pt {
        Pt(self.l(), self.t())
    }
    /// The point at the top-right corner of an object's bounding box.
    pub fn tr(&self) -> Pt {
        Pt(self.r(), self.t())
    }
    /// The point at the bottom-left corner of an object's bounding box.
    pub fn bl(&self) -> Pt {
        Pt(self.l(), self.b())
    }
    /// The point at the bottom-right corner of an object's bounding box.
    pub fn br(&self) -> Pt {
        Pt(self.r(), self.b())
    }

    /// The center of the bounding box of an object.
    pub fn center(&self) -> Pt {
        Pt(self.l() + (self.w() / 2.0), self.t() + (self.h() / 2.0))
    }
}

impl Bounded for Bounds {
    fn bounds(&self) -> Result<Bounds> {
        Ok(*self)
    }
}

/// An object which is Bounded implements four cardinal bounds; the trait allows
/// a caller to discover the width, height, four corners, bounding box, and
/// center of that object.
///
/// Unlike most graphics systems, we assume that (0,0) is in the bottom-left.
#[enum_dispatch(Obj)]
pub trait Bounded {
    /// Internal use only.
    fn bounds(&self) -> Result<Bounds>;
}

/// A handy struct for collecting the outer bounds of a streaming iterator of
/// polygons.
pub struct BoundsCollector {
    bound_t: Option<FloatOrd<f64>>,
    bound_b: Option<FloatOrd<f64>>,
    bound_l: Option<FloatOrd<f64>>,
    bound_r: Option<FloatOrd<f64>>,
    items_seen: usize,
}

impl Default for BoundsCollector {
    /// A new bounds collector.
    fn default() -> Self {
        BoundsCollector {
            bound_t: None,
            bound_b: None,
            bound_l: None,
            bound_r: None,
            items_seen: 0_usize,
        }
    }
}

impl BoundsCollector {
    /// How many items has this seen?
    pub fn items_seen(&self) -> usize {
        self.items_seen
    }

    /// Incorporate a new polygon to this bounds collector.
    pub fn incorporate(&mut self, b: &impl Bounded) -> Result<()> {
        let bounds = b.bounds()?;
        // top
        self.bound_t = Some(match self.bound_t {
            None => FloatOrd(bounds.t()),
            Some(existing) => std::cmp::max(existing, FloatOrd(bounds.t())),
        });
        // bottom
        self.bound_b = Some(match self.bound_b {
            None => FloatOrd(bounds.b()),
            Some(existing) => std::cmp::min(existing, FloatOrd(bounds.b())),
        });
        // right
        self.bound_r = Some(match self.bound_r {
            None => FloatOrd(bounds.r()),
            Some(existing) => std::cmp::max(existing, FloatOrd(bounds.r())),
        });
        // left
        self.bound_l = Some(match self.bound_l {
            None => FloatOrd(bounds.l()),
            Some(existing) => std::cmp::min(existing, FloatOrd(bounds.l())),
        });
        self.items_seen += 1;
        Ok(())
    }
}

impl Bounded for BoundsCollector {
    fn bounds(&self) -> Result<Bounds> {
        Ok(Bounds {
            top_bound: self.bound_t.ok_or(anyhow!("absent"))?.0,
            bottom_bound: self.bound_b.ok_or(anyhow!("absent"))?.0,
            left_bound: self.bound_l.ok_or(anyhow!("absent"))?.0,
            right_bound: self.bound_r.ok_or(anyhow!("absent"))?.0,
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
    use crate::shapes::{pg::Pg, pt::Pt};

    #[test]
    fn test_streaming_bbox() {
        let polygons = vec![
            Pg([(0, 0), (1, 0), (1, 1)]),
            Pg([(2, 0), (3, 0), (3, 1)]),
            Pg([(0, 2), (1, 2), (1, 3)]),
        ];
        let bounds = streaming_bbox(&polygons).unwrap();
        assert_eq!(bounds.bl(), Pt(0, 0));
        assert_eq!(bounds.tl(), Pt(0, 3));
        assert_eq!(bounds.tr(), Pt(3, 3));
        assert_eq!(bounds.br(), Pt(3, 0));
    }
}
