//! A general-purpose 2D geometry library.

#![deny(missing_docs)]

pub mod bounded;
pub mod crop;
pub mod grid;
pub mod group;
pub mod interpolate;
pub mod intersection;
pub mod obj;
pub mod shading;
pub mod shapes;
pub mod style;

#[macro_use]
pub mod macros;

use crate::{obj::Obj, shapes::pt::Pt, style::Style};
use enum_dispatch::enum_dispatch;
use obj::ObjType;
use shapes::txt::Txt;
use std::ops::*;
use typed_builder::TypedBuilder;

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

/// Settings for debug annotation (font size, etc.)
#[derive(Debug, Clone, TypedBuilder)]
pub struct AnnotationSettings {
    /// Font size.
    pub font_size: f64,
    /// Decimals of precision:
    pub precision: usize,
}

impl Default for AnnotationSettings {
    fn default() -> Self {
        Self {
            font_size: 10.0,
            precision: 0,
        }
    }
}

/// A 2d object.
#[enum_dispatch(Obj)]
pub trait Object {
    /// Return the labelled points and segments.
    fn annotate(&self, settings: &AnnotationSettings) -> Vec<(Obj, Style)> {
        let mut a = vec![];

        let AnnotationSettings {
            font_size,
            precision,
        } = settings;
        for (_idx, pt) in self.iter().enumerate() {
            let x = format!("{:.1$}", pt.x, precision);
            let y = format!("{:.1$}", pt.y, precision);
            a.push((
                Txt {
                    pt: *pt,
                    inner: format!("({}, {})", x, y),
                    font_size: *font_size,
                }
                .into(),
                Style::default(),
            ));
        }

        a
    }

    /// Is it empty?
    fn is_empty(&self) -> bool {
        self.iter().count() == 0
    }

    /// What type of object is this?
    fn objtype(&self) -> ObjType;

    /// Iterator
    fn iter(&self) -> Box<dyn Iterator<Item = &Pt> + '_>;

    /// Mutable iterator
    fn iter_mut(&mut self) -> Box<dyn Iterator<Item = &mut Pt> + '_>;
}
