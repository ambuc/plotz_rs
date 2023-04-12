//! A style.

use {plotz_color::*, typed_builder::TypedBuilder};

/// Styling information for the 3d representation.
#[derive(Debug, Clone, TypedBuilder)]
pub struct Style {
    /// A color.
    #[builder(default=&BLACK)]
    pub color: &'static ColorRGB,

    /// A thickness.
    #[builder(default = 1.0)]
    pub thickness: f64,
}
