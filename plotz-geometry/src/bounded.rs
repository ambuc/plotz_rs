//! A trait representing the bounds and bounding box for an object.
use crate::{
    point::Pt,
    polygon::{Polygon, PolygonConstructorError},
};

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
