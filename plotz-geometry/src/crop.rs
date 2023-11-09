//! Crop
use crate::{bounded::Bounds, shapes::polygon::Polygon};
use anyhow::Result;

/// Whether a point lies outside, inside, or on a vertex or edge of a polygon.
#[derive(Debug, PartialEq, Eq)]
pub enum PointLocation {
    /// A point lies outside a polygon.
    Outside,
    /// A point lies inside a polygon.
    Inside,
    /// A point lies on the nth point of a polygon.
    OnPoint(usize),
    /// A point lies on the nth segment of a polygon.
    OnSegment(usize),
}

/// Types of crops.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum CropType {
    /// The traditional crop -- cropping a to b inclusively preserves the bit(s) of
    /// a which is also in b.
    Inclusive,
    /// The other one -- cropping a to b exclusively preserves the bit(s) of a
    /// which are not in b.
    Exclusive,
}

/// Crops
pub trait Croppable {
    /// The output type of cropping this thingy. Why is this an associated type?
    /// Simple: I'm not 100% sure that the output type of cropping T is always
    /// Vec<T>. What if it's not! What then!?
    type Output;

    /// Crop self to an outer frame
    fn crop_to(&self, other: &Polygon) -> Result<Vec<Self::Output>>
    where
        Self: Sized,
    {
        self.crop(other, CropType::Inclusive)
    }

    /// Crop self so that the portion of self overlapping other is removed.
    fn crop_excluding(&self, other: &Polygon) -> Result<Vec<Self::Output>>
    where
        Self: Sized,
    {
        self.crop(other, CropType::Exclusive)
    }

    /// Crop self to outer bounds.
    fn crop_to_bounds(&self, bounds: Bounds) -> Result<Vec<Self::Output>>
    where
        Self: Sized,
    {
        self.crop_to(&bounds.to_polygon())
    }

    /// general crop -- could be either type.
    fn crop(&self, other: &Polygon, crop_type: CropType) -> Result<Vec<Self::Output>>;
}
