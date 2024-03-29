//! A general-purpose 2D geometry library.

#![deny(missing_docs)]

pub mod bounded;
pub mod crop;
pub mod grid;
pub mod group;
pub mod interpolate;
pub mod intersection;
pub mod obj2;
pub mod overlaps;
pub mod shading;
pub mod shapes;
pub mod style;

#[macro_use]
pub mod macros;

mod utils;

use crate::{obj2::Obj2, shapes::point::Point, style::Style};
use enum_dispatch::enum_dispatch;
use obj2::ObjType2d;
use shapes::text::Text;
use std::ops::*;
use typed_builder::TypedBuilder;

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
#[enum_dispatch(Obj2)]
pub trait Object {
    /// Return the labelled points and segments.
    fn annotate(&self, settings: &AnnotationSettings) -> Vec<(Obj2, Style)> {
        let mut a = vec![];

        let AnnotationSettings {
            font_size,
            precision,
        } = settings;
        for (_idx, pt) in self.iter().enumerate() {
            let x = format!("{:.1$}", pt.x, precision);
            let y = format!("{:.1$}", pt.y, precision);
            a.push((
                Text {
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
    fn objtype(&self) -> ObjType2d;

    /// Iterator
    fn iter(&self) -> Box<dyn Iterator<Item = &Point> + '_>;

    /// Mutable iterator
    fn iter_mut(&mut self) -> Box<dyn Iterator<Item = &mut Point> + '_>;
}
