//! A style.

use std::fmt::Debug;

use {plotz_color::*, typed_builder::TypedBuilder};

/// Styling information for the 3d representation.
#[derive(Clone, Copy, TypedBuilder)]
pub struct Style3d {
    /// A color.
    #[builder(default=&BLACK)]
    pub color: &'static ColorRGB,

    /// A thickness.
    #[builder(default = 1.0)]
    pub thickness: f64,
}

impl Style3d {
    /// Modifier with color.
    pub fn with_color(self, c: &'static ColorRGB) -> Style3d {
        Self { color: c, ..self }
    }
}

impl Debug for Style3d {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Style3d { color, thickness } = self;
        write!(f, "color={:?} thickness={:?}", color, thickness)
    }
}
