#![allow(missing_docs)]

use {
    crate::shading::shade_config::ShadeConfig, plotz_color::*, std::fmt::Debug,
    typed_builder::TypedBuilder,
};

#[derive(Clone, Copy, TypedBuilder)]
pub struct Style {
    #[builder(default=&BLACK)]
    pub color: &'static ColorRGB,

    #[builder(default = 1.0)]
    pub thickness: f64,

    #[builder(default = None, setter(strip_option))]
    pub shading: Option<ShadeConfig>,
}

impl Style {
    pub fn with_color(self, c: &'static ColorRGB) -> Style {
        Self { color: c, ..self }
    }
    pub fn new(c: &'static ColorRGB, t: f64) -> Style {
        Self::builder().color(c).thickness(t).build()
    }
}

impl Debug for Style {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Style {
            color,
            thickness,
            shading,
        } = self;
        write!(
            f,
            "Style::builder().color({:?}).thickness({:?}).shading({:?}).build()",
            color, thickness, shading
        )
    }
}
