//! A scene, i.e. a holder for 3d objects ready to be projected down onto a 2d
//! plane.

use {
    crate::{camera::Projection, object::Object, object_inner::ObjectInner, style::Style},
    plotz_geometry::draw_obj::DrawObj,
};

/// A scene of 3d objects ready to be projected down to a 2d plane.
#[derive(Debug, Clone)]
pub struct Scene {
    /// Some objects.
    pub objects: Vec<Object>,
}

impl Scene {
    /// A new scene.
    pub fn new() -> Scene {
        Scene { objects: vec![] }
    }
    /// Make a scene from some objects.
    pub fn from(a: impl IntoIterator<Item = Object>) -> Scene {
        Scene {
            objects: a.into_iter().collect(),
        }
    }
    /// Make a scene from some objects
    pub fn from_objects_with_style(
        a: impl IntoIterator<Item = ObjectInner>,
        style: Style,
    ) -> Scene {
        Scene {
            objects: a
                .into_iter()
                .map(|a| Object::new(a).with_style(style.clone()))
                .collect(),
        }
    }

    /// Projects the scene onto a camera, renders to 2d, and returns a vector of drawobjects.
    pub fn project_with(&self, projection: Projection) -> Vec<DrawObj> {
        match projection {
            Projection::Oblique(oblique_projection) => self
                .objects
                .iter()
                .flat_map(|obj| obj.project_oblique(&oblique_projection))
                .collect::<Vec<_>>(),
        }
    }
}
