//! A 3d object.

use crate::{object_inner::ObjectInner, style::Style};

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
}
