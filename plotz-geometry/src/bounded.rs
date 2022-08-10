//! A trait representing the bounds and bounding box for an object.
use crate::{
    point::Pt,
    polygon::{Polygon, PolygonConstructorError},
};
use float_ord::FloatOrd;

/// A general error arising from trying to derive the bounding box for a thing.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum BoundingBoxError {
    /// Could not construct the bounding box polygon.
    #[error("Could not construct bounding box polygon.")]
    PolygonConstructorError(#[from] PolygonConstructorError),
}

/// An object which is Bounded implements four cardinal bounds; the trait allows
/// a caller to discover the width, height, four corners, bounding box, and
/// center of that object.
///
/// Unlike most graphics systems, we assume that (0,0) is in the bottom-left.
pub trait Bounded {
    /// The right bound of an object.
    fn right_bound(&self) -> f64;
    /// The left bound of an object.
    fn left_bound(&self) -> f64;
    /// The top bound of an object.
    fn top_bound(&self) -> f64;
    /// The bottom bound of an object.
    fn bottom_bound(&self) -> f64;
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
    /// The [bounding box](https://en.wikipedia.org/wiki/Minimum_bounding_box)
    /// for an object.
    fn bbox(&self) -> Result<Polygon, BoundingBoxError> {
        Ok(Polygon([
            self.tl_bound(),
            self.tr_bound(),
            self.br_bound(),
            self.bl_bound(),
        ])?)
    }
    /// The center of the bounding box of an object.
    fn bbox_center(&self) -> Pt {
        Pt(
            self.left_bound() + (self.width() / 2.0),
            self.top_bound() + (self.height() / 2.0),
        )
    }
}

struct BoundsCollector {
    bound_t: Option<FloatOrd<f64>>,
    bound_b: Option<FloatOrd<f64>>,
    bound_l: Option<FloatOrd<f64>>,
    bound_r: Option<FloatOrd<f64>>,
}

impl BoundsCollector {
    pub fn new() -> BoundsCollector {
        BoundsCollector {
            bound_t: None,
            bound_b: None,
            bound_l: None,
            bound_r: None,
        }
    }
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
    }
}

impl Bounded for BoundsCollector {
    fn top_bound(&self) -> f64 {
        self.bound_t.expect("top bound should be present").0
    }
    fn bottom_bound(&self) -> f64 {
        self.bound_b.expect("bottom bound should be present").0
    }
    fn left_bound(&self) -> f64 {
        self.bound_l.expect("left bound should be present").0
    }
    fn right_bound(&self) -> f64 {
        self.bound_r.expect("right bound should be present").0
    }
}

/// Given an iterator of bounded items, computes the bounding box for that
/// collection.
pub fn streaming_bbox<'a, T: 'a + Bounded>(
    it: impl IntoIterator<Item = &'a T>,
) -> Result<Polygon, BoundingBoxError> {
    let mut bc = BoundsCollector::new();
    for i in it {
        bc.incorporate(i);
    }
    bc.bbox()
}

#[cfg(test)]
mod test_super {
    use super::*;

    #[test]
    fn test_streaming_bbox() {
        let polygons = vec![
            Polygon([Pt(0, 0), Pt(1, 0), Pt(1, 1)]).unwrap(),
            Polygon([Pt(2, 0), Pt(3, 0), Pt(3, 1)]).unwrap(),
            Polygon([Pt(0, 2), Pt(1, 2), Pt(1, 3)]).unwrap(),
        ];
        assert_eq!(
            streaming_bbox(&polygons).unwrap(),
            Polygon([Pt(0, 0), Pt(0, 3), Pt(3, 3), Pt(3, 0)]).unwrap()
        );
    }
}
