//! A config for shading.

use typed_builder::TypedBuilder;

/// Config for controlling crosshatching.
#[derive(Debug, Copy, Clone, TypedBuilder, PartialEq)]
pub struct ShadeConfig {
    /// The gap between lines.
    pub gap: f64,

    /// The slope of a line.
    /// zero is flat.
    /// 1.0 is diagonal northeast (southwest).
    /// -1.0 is diagonal northwest (southeast).
    /// infinity is straight up-and-down.
    pub slope: f64,

    /// The thickness of a line (SVG only.)
    #[builder(default = 1.0)]
    pub thickness: f64,

    /// whether or not to zig zag.
    #[builder(default = false)]
    pub switchback: bool,
}
