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
        Pg([
            self.tl_bound(),
            self.tr_bound(),
            self.br_bound(),
            self.bl_bound(),
            self.tl_bound(),
        ])
    }
    /// Whether or not bounds contain a point.
    pub fn contains_pt(&self, pt: Pt) -> Result<PointLoc> {
        self.to_polygon().contains_pt(&pt)
    }
}

impl Bounded for Bounds {
    fn bounds(&self) -> Bounds {
        *self
    }
}

/// An object which is Bounded implements four cardinal bounds; the trait allows
/// a caller to discover the width, height, four corners, bounding box, and
/// center of that object.
///
/// Unlike most graphics systems, we assume that (0,0) is in the bottom-left.
#[enum_dispatch(Obj)]
pub trait Bounded {
    /// The right bound of an object.
    fn right_bound(&self) -> f64 {
        self.bounds().right_bound
    }
    /// The left bound of an object.
    fn left_bound(&self) -> f64 {
        self.bounds().left_bound
    }
    /// The top bound of an object.
    fn top_bound(&self) -> f64 {
        self.bounds().top_bound
    }
    /// The bottom bound of an object.
    fn bottom_bound(&self) -> f64 {
        self.bounds().bottom_bound
    }
    /// The width of an object.
    fn width(&self) -> f64 {
        self.right_bound() - self.left_bound()
    }
    /// The height of an object.
    fn height(&self) -> f64 {
        self.bottom_bound() - self.top_bound()
    }
    /// The point at the top-left corner of an object's bounding box.
    fn tl_bound(&self) -> Pt {
        Pt(self.left_bound(), self.top_bound())
    }
    /// The point at the top-right corner of an object's bounding box.
    fn tr_bound(&self) -> Pt {
        Pt(self.right_bound(), self.top_bound())
    }
    /// The point at the bottom-left corner of an object's bounding box.
    fn bl_bound(&self) -> Pt {
        Pt(self.left_bound(), self.bottom_bound())
    }
    /// The point at the bottom-right corner of an object's bounding box.
    fn br_bound(&self) -> Pt {
        Pt(self.right_bound(), self.bottom_bound())
    }

    /// The center of the bounding box of an object.
    fn bbox_center(&self) -> Pt {
        Pt(
            self.left_bound() + (self.width() / 2.0),
            self.top_bound() + (self.height() / 2.0),
        )
    }

    /// Internal use only.
    fn bounds(&self) -> Bounds;
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
    pub fn incorporate(&mut self, b: &impl Bounded) {
        // top
        self.bound_t = Some(match self.bound_t {
            None => FloatOrd(b.top_bound()),
            Some(existing) => std::cmp::max(existing, FloatOrd(b.top_bound())),
        });
        // bottom
        self.bound_b = Some(match self.bound_b {
            None => FloatOrd(b.bottom_bound()),
            Some(existing) => std::cmp::min(existing, FloatOrd(b.bottom_bound())),
        });
        // right
        self.bound_r = Some(match self.bound_r {
            None => FloatOrd(b.right_bound()),
            Some(existing) => std::cmp::max(existing, FloatOrd(b.right_bound())),
        });
        // left
        self.bound_l = Some(match self.bound_l {
            None => FloatOrd(b.left_bound()),
            Some(existing) => std::cmp::min(existing, FloatOrd(b.left_bound())),
        });
        self.items_seen += 1;
    }
}

impl Bounded for BoundsCollector {
    fn bounds(&self) -> Bounds {
        Bounds {
            top_bound: self.bound_t.expect("").0,
            bottom_bound: self.bound_b.expect("").0,
            left_bound: self.bound_l.expect("").0,
            right_bound: self.bound_r.expect("").0,
        }
    }
}

/// Given an iterator of bounded items, computes the bounding box for that
/// collection.
pub fn streaming_bbox<'a>(it: impl IntoIterator<Item = &'a (impl Bounded + 'a)>) -> Result<Bounds> {
    let mut bc = BoundsCollector::default();
    for i in it {
        bc.incorporate(i);
    }
    if bc.items_seen == 0 {
        return Err(anyhow!("no items seen"));
    }
    Ok(bc.bounds())
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
        assert_eq!(bounds.bl_bound(), Pt(0, 0));
        assert_eq!(bounds.tl_bound(), Pt(0, 3));
        assert_eq!(bounds.tr_bound(), Pt(3, 3));
        assert_eq!(bounds.br_bound(), Pt(3, 0));
    }
}
