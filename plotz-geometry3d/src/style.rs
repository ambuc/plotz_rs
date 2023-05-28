//! A style.

use plotz_geometry::shading::shade_config::ShadeConfig;

use crate::{obj3::Obj3, styled_obj3::StyledObj3};

use {plotz_color::*, std::fmt::Debug, typed_builder::TypedBuilder};

#[derive(Clone, Copy, TypedBuilder)]
pub struct Style3d {
    #[builder(default=&BLACK)]
    pub color: &'static ColorRGB,

    #[builder(default = 1.0)]
    pub thickness: f64,

    #[builder(default = None, setter(strip_option))]
    pub shading: Option<ShadeConfig>,
}

impl Style3d {
    pub fn with_color(self, c: &'static ColorRGB) -> Style3d {
        Self { color: c, ..self }
    }
    pub fn new(c: &'static ColorRGB, t: f64) -> Style3d {
        Self::builder().color(c).thickness(t).build()
    }

    pub fn apply(self, o: impl Into<Obj3>) -> StyledObj3 {
        StyledObj3::new(o.into()).with_style(self)
    }
}

impl Debug for Style3d {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Style3d {
            color,
            thickness,
            shading,
        } = self;
        write!(
            f,
            "Style3d::builder().color({:?}).thickness({:?}).shading({:?}).build()",
            color, thickness, shading
        )
    }
}
