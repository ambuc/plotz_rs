//! Traits.

use typed_builder::TypedBuilder;

use crate::object2d::Object2d;

use {crate::point::Pt, std::ops::*};

/// A geometric figure made of points which might emit a boxed iterator of immutable points.
pub trait YieldPoints {
    /// Possibly yields a boxed iterator of immutable points.
    fn yield_pts(&self) -> Option<Box<dyn Iterator<Item = &Pt> + '_>>;
}

/// A geometric figure made of points which might emit a boxed iterator of mutable points.
pub trait YieldPointsMut {
    /// Possibly yields a boxed iterator of mutable points.
    fn yield_pts_mut(&mut self) -> Option<Box<dyn Iterator<Item = &mut Pt> + '_>>;
}

/// A geometric figure made of points which can be maniuplated by passing f:
/// impl Fn(&mut Pt) and mutating each one.
pub trait Mutable: YieldPointsMut {
    /// Mutate the points of a geometric figure by applying f(pt) to each of them.
    fn mutate(&mut self, f: impl Fn(&mut Pt)) -> bool {
        if let Some(yp) = self.yield_pts_mut() {
            yp.for_each(f);
            return true;
        }
        false
    }
}

/// A geometric figure which can be translated by an xy shift (represented by a Point).
pub trait Translatable: Add<Pt> + AddAssign<Pt> + Sub<Pt> + SubAssign<Pt> + Sized {}

/// A geometric figure which can be scaled by a factor of |f|.
pub trait Scalable<T>: Mul<T> + MulAssign<T> + Div<T> + DivAssign<T> + Sized {}

/// The same as |Translatable|, but in-place. (See add vs. add_assign.)
pub trait TranslatableAssign: AddAssign<Pt> + SubAssign<Pt> {}

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
    fn annotate(&self, settings: &AnnotationSettings) -> Vec<Object2d>;
}

impl Default for AnnotationSettings {
    fn default() -> Self {
        Self {
            font_size: 10.0,
            precision: 0,
        }
    }
}
