//! A 3d object.

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
    pub fn project_oblique(&self, oblique_projection: &Oblique) -> Vec<DrawObj> {
        let mut dos = self
            .inner
            .project_oblique(oblique_projection)
            .into_iter()
            .map(|doi| DrawObj::new(doi))
            .collect::<Vec<_>>();

        if let Some(Style { color, thickness }) = self.style {
            dos = dos
                .into_iter()
                .map(|d_o| d_o.clone().with_color(color).with_thickness(thickness))
                .collect();
        }
        dos
    }
}
