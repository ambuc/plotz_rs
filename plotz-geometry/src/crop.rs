//! Crop
use {
    crate::{
        bounded::Bounds,
        shapes::polygon::{Pg2, PolygonConstructorError},
    },
    thiserror::Error,
};

/// A general error arising from trying to inspect whether a point lies in a
/// polygon.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ContainsPointError {
    /// The bounding polygon is Open (not Closed) and so it is underspecified to
    /// ask whether it contains a point.
    #[error("The bounding polygon is Open (not Closed) and so it is underspecified to ask whether it contains a point.")]
    PolygonIsOpen,
}

/// A general error arising from trying to crop something to a bounding polygon.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CropToPolygonError {
    /// The frame polygon is not closed.
    #[error("The frame polygon is not closed.")]
    ThisPolygonNotClosed,
    /// The inner polygon is not closed.
    #[error("The inner polygon is not closed.")]
    ThatPolygonNotClosed,
    /// The frame polygon is not positively oriented.
    #[error("The frame polygon is not positively oriented.")]
    ThisPolygonNotPositivelyOriented,
    /// The inner polygon is not positively oriented.
    #[error("The inner polygon is not positively oriented.")]
    ThatPolygonNotPositivelyOriented,
    /// Some inspection of whether a point lies in a polygon failed.
    #[error("Some inspection of whether a point lies in a polygon failed.")]
    ContainsPointError(#[from] ContainsPointError),
    /// Some Polygon construction failed.
    #[error("Some Polygon construction failed.")]
    PolygonConstructorError(#[from] PolygonConstructorError),
    /// Constructing a resultant polygon failed because we encountered a cycle.
    #[error("Constructing a resultant polygon failed because we encountered a cycle.")]
    CycleError,
}

/// Whether a point lies outside, inside, or on a vertex or edge of a polygon.
#[derive(Debug, PartialEq, Eq)]
pub enum PointLoc {
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
    fn crop_to(&self, other: &Pg2) -> Vec<Self::Output>
    where
        Self: Sized,
    {
        self.crop(other, CropType::Inclusive)
    }

    /// Crop self so that the portion of self overlapping other is removed.
    fn crop_excluding(&self, other: &Pg2) -> Vec<Self::Output>
    where
        Self: Sized,
    {
        self.crop(other, CropType::Exclusive)
    }

    /// Crop self to outer bounds.
    fn crop_to_bounds(&self, bounds: Bounds) -> Vec<Self::Output>
    where
        Self: Sized,
    {
        self.crop_to(&bounds.to_polygon())
    }

    /// general crop -- could be either type.
    fn crop(&self, other: &Pg2, crop_type: CropType) -> Vec<Self::Output>;
}
