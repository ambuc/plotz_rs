//! A 3d object.

use {crate::obj3::Obj3, plotz_color::ColorRGB, plotz_geometry::style::Style, std::fmt::Debug};

#[derive(Clone)]
pub struct StyledObj3 {
    pub inner: Obj3,
    pub style: Option<Style>,
}

impl StyledObj3 {
    pub fn new(a: impl Into<Obj3>) -> StyledObj3 {
        StyledObj3 {
            inner: a.into(),
            style: None,
        }
    }
    pub fn with_style(self, a: Style) -> StyledObj3 {
        StyledObj3 {
            style: Some(a),
            ..self
        }
    }
    pub fn with_color(self, c: &'static ColorRGB) -> StyledObj3 {
        StyledObj3 {
            style: match self.style {
                None => Some(Style::builder().color(c).build()),
                Some(s) => Some(Style { color: c, ..s }),
            },
            ..self
        }
    }
}

impl Debug for StyledObj3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let StyledObj3 { inner, style } = self;
        match style {
            Some(style) => write!(f, "Object3d::new({:?}).with_style({:?})", inner, style),
            None => write!(f, "Object3d::new({:?})", inner),
        }
    }
}
