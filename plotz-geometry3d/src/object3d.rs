//! A 3d object.

use crate::point3d::Pt3d;

use {
    crate::{camera::Oblique, object3d_inner::Object3dInner, style::Style3d},
    plotz_geometry::object2d::Object2d,
};

/// A 3d object and some styling information for its 2d representation.
#[derive(Debug, Clone)]
pub struct Object3d {
    /// An inner object.
    pub inner: Object3dInner,
    /// A style.
    pub style: Option<Style3d>,
}

impl Object3d {
    /// New object.
    pub fn new(a: impl Into<Object3dInner>) -> Object3d {
        Object3d {
            inner: a.into(),
            style: None,
        }
    }
    /// Constructor with style.
    pub fn with_style(self, a: Style3d) -> Object3d {
        Object3d {
            style: Some(a),
            ..self
        }
    }

    /// Project oblique.
    pub fn project_oblique(&self, oblique_projection: &Oblique) -> Object2d {
        let mut d_o = Object2d::new(self.inner.project_oblique(oblique_projection));

        if let Some(Style3d { color, thickness }) = self.style {
            d_o = d_o.with_color(color).with_thickness(thickness);
        }
        d_o
    }

    /// The center of the object, projected along the view vector.
    pub fn dist_along(&self, view_vector: &Pt3d) -> f64 {
        self.inner.dist_along(view_vector)
    }
}
