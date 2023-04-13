//! A 3d object.

use crate::point3d::Pt3d;

use {
    crate::{camera::Oblique, object_inner::ObjectInner, style::Style},
    plotz_geometry::draw_obj::DrawObj,
};

/// A 3d object and some styling information for its 2d representation.
#[derive(Debug, Clone)]
pub struct Object {
    /// An inner object.
    pub inner: ObjectInner,
    /// A style.
    pub style: Option<Style>,
}

impl Object {
    /// New object.
    pub fn new(a: impl Into<ObjectInner>) -> Object {
        Object {
            inner: a.into(),
            style: None,
        }
    }
    /// Constructor with style.
    pub fn with_style(self, a: Style) -> Object {
        Object {
            style: Some(a),
            ..self
        }
    }

    /// Project oblique.
    pub fn project_oblique(&self, oblique_projection: &Oblique) -> DrawObj {
        let mut d_o = DrawObj::new(self.inner.project_oblique(oblique_projection));

        if let Some(Style { color, thickness }) = self.style {
            d_o = d_o.with_color(color).with_thickness(thickness);
        }
        d_o
    }

    /// The center of the object, projected along the view vector.
    pub fn dist_along(&self, view_vector: &Pt3d) -> f64 {
        self.inner.dist_along(view_vector)
    }
}
