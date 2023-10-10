//! A general-purpose 2D geometry library.

#![deny(missing_docs)]

pub mod bounded;
pub mod crop;
pub mod group;
pub mod interpolate;
pub mod intersection;
pub mod obj;
pub mod style;

pub mod grid;
pub mod shading;
pub mod shapes;

use crate::{obj::Obj, shapes::pt::Pt, style::Style};
use enum_dispatch::enum_dispatch;
use std::ops::*;
use typed_builder::TypedBuilder;

/// A geometric figure which can be translated by an xy shift (represented by a Point).
#[enum_dispatch(Obj)]
pub trait Translatable: Add<Pt> + AddAssign<Pt> + Sub<Pt> + SubAssign<Pt> + Sized {}

/// A geometric figure which can be scaled by a factor of |f|.
pub trait Scalable<T>: Mul<T> + MulAssign<T> + Div<T> + DivAssign<T> + Sized {}

/// The same as |Translatable|, but in-place. (See add vs. add_assign.)
#[enum_dispatch(Obj)]
pub trait TranslatableAssign: AddAssign<Pt> + SubAssign<Pt> {}

/// The same as |Scalable|, but in-place. (See add vs. add_assign.)
#[enum_dispatch(Obj)]
pub trait ScalableAssign: MulAssign<f64> + DivAssign<f64> {}

/// A geometric figure made of points with floating-point xy components which
/// can be modified in-place to snap to the nearest interval.
pub trait Roundable {
    /// Rounds the points of a floating-point geometric figure to the nearest interval |f|.
    /// Example: 0.351.round_to_nearest(0.1) => 0.3.
    fn round_to_nearest(&mut self, f: f64);
}

/// A geometric figure which can be empty. Most can't.
#[enum_dispatch(Obj)]
pub trait Nullable {
    /// Is it empty?
    fn is_empty(&self) -> bool;
}

/// Settings for debug annotation (font size, etc.)
#[derive(Debug, Clone, TypedBuilder)]
pub struct AnnotationSettings {
    /// Font size.
    pub font_size: f64,
    /// Decimals of precision:
    pub precision: usize,
}

/// Something which can have its points and segments labelled.
pub trait Annotatable {
    /// Return the labelled points and segments.
    fn annotate(&self, settings: &AnnotationSettings) -> Vec<(Obj, Style)>;
}

impl Default for AnnotationSettings {
    fn default() -> Self {
        Self {
            font_size: 10.0,
            precision: 0,
        }
    }
}
