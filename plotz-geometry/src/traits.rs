//! Traits.

use crate::{obj2::Obj2, style::Style};

use {crate::shapes::pt2::Pt2, std::ops::*, typed_builder::TypedBuilder};

/// A geometric figure which can be translated by an xy shift (represented by a Point).
pub trait Translatable: Add<Pt2> + AddAssign<Pt2> + Sub<Pt2> + SubAssign<Pt2> + Sized {}

/// A geometric figure which can be scaled by a factor of |f|.
pub trait Scalable<T>: Mul<T> + MulAssign<T> + Div<T> + DivAssign<T> + Sized {}

/// The same as |Translatable|, but in-place. (See add vs. add_assign.)
pub trait TranslatableAssign: AddAssign<Pt2> + SubAssign<Pt2> {}

/// The same as |Scalable|, but in-place. (See add vs. add_assign.)
pub trait ScalableAssign: MulAssign<f64> + DivAssign<f64> {}

/// A geometric figure made of points with floating-point xy components which
/// can be modified in-place to snap to the nearest interval.
pub trait Roundable {
    /// Rounds the points of a floating-point geometric figure to the nearest interval |f|.
    /// Example: 0.351.round_to_nearest(0.1) => 0.3.
    fn round_to_nearest(&mut self, f: f64);
}

/// A geometric figure which can be empty. Most can't.
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
    fn annotate(&self, settings: &AnnotationSettings) -> Vec<(Obj2, Style)>;
}

impl Default for AnnotationSettings {
    fn default() -> Self {
        Self {
            font_size: 10.0,
            precision: 0,
        }
    }
}
