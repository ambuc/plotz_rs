//! A 3d object.

use {
    crate::{camera::Oblique, obj3::Obj3, shapes::pt3::Pt3},
    plotz_color::ColorRGB,
    plotz_geometry::{style::Style, styled_obj2::StyledObj2},
    std::fmt::Debug,
};

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
                Some(s) => Some(s.with_color(c)),
            },
            ..self
        }
    }

    // Project oblique.
    pub fn project_oblique(&self, oblique_projection: &Oblique) -> StyledObj2 {
        let mut d_o = StyledObj2::new(self.inner.project_oblique(oblique_projection));

        if let Some(Style {
            color, thickness, ..
        }) = self.style
        {
            d_o = d_o.with_color(color).with_thickness(thickness);
        }
        d_o
    }

    // The center of the object, projected along the view vector.
    pub fn dist_along(&self, view_vector: &Pt3) -> f64 {
        self.inner.dist_along(view_vector)
    }
    // The maximum distance of this object, projected along the view vector.
    pub fn max_dist_along(&self, view_vector: &Pt3) -> f64 {
        self.inner.max_dist_along(view_vector)
    }
    // The minimum distance of this object, projected along the view vector.
    pub fn min_dist_along(&self, view_vector: &Pt3) -> f64 {
        self.inner.min_dist_along(view_vector)
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
