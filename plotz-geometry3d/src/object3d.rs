//! A 3d object.

use std::fmt::Debug;

use plotz_color::ColorRGB;

use crate::point3d::Pt3d;

use {
    crate::{camera::Oblique, object3d_inner::Object3dInner, style::Style3d},
    plotz_geometry::object2d::Object2d,
};

/// A 3d object and some styling information for its 2d representation.
#[derive(Clone)]
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
    /// Modifier with color.
    pub fn with_color(self, c: &'static ColorRGB) -> Object3d {
        Object3d {
            style: match self.style {
                None => Some(Style3d::builder().color(c).build()),
                Some(s) => Some(s.with_color(c)),
            },
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
    /// The maximum distance of this object, projected along the view vector.
    pub fn max_dist_along(&self, view_vector: &Pt3d) -> f64 {
        self.inner.max_dist_along(view_vector)
    }
    /// The minimum distance of this object, projected along the view vector.
    pub fn min_dist_along(&self, view_vector: &Pt3d) -> f64 {
        self.inner.min_dist_along(view_vector)
    }
}

impl Debug for Object3d {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Object3d { inner, style } = self;
        write!(f, "inner={:?} style={:?}", inner, style)
    }
}
