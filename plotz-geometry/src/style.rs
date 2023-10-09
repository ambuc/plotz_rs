#![allow(missing_docs)]

use {crate::shading::shade_config::ShadeConfig, plotz_color::*, std::fmt::Debug};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Style {
    pub color: &'static ColorRGB,
    pub thickness: f64,
    pub shading: Option<ShadeConfig>,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            color: &BLACK,
            thickness: 1.0,
            shading: None,
        }
    }
}
