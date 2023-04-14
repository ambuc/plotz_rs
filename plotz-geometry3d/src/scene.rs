//! A scene, i.e. a holder for 3d objects ready to be projected down onto a 2d
//! plane.

use {
    crate::{
        camera::{Occlusion, Projection},
        object3d::Object3d,
        object3d_inner::Object3dInner,
        occluder::Occluder,
        point3d::Pt3d,
        style::Style3d,
    },
    float_ord::FloatOrd,
    itertools::Itertools,
    plotz_geometry::object2d::Object2d,
};

/// A scene of 3d objects ready to be projected down to a 2d plane.
#[derive(Debug, Clone)]
pub struct Scene {
    /// Some objects.
    pub objects: Vec<Object3d>,
}

impl Scene {
    /// A new scene.
    pub fn new() -> Scene {
        Scene { objects: vec![] }
    }
    /// Make a scene from some objects.
    pub fn from(a: impl IntoIterator<Item = Object3d>) -> Scene {
        Scene {
            objects: a.into_iter().collect(),
        }
    }
    /// Make a scene from some objects
    pub fn from_objects_with_style(
        a: impl IntoIterator<Item = Object3dInner>,
        style: Style3d,
    ) -> Scene {
        Scene {
            objects: a
                .into_iter()
                .map(|a| Object3d::new(a).with_style(style.clone()))
                .collect(),
        }
    }

    /// Projects the scene onto a camera, renders to 2d, and returns a vector of object2ds.
    pub fn project_with(&self, projection: Projection, occlusion: Occlusion) -> Vec<Object2d> {
        match (projection, occlusion) {
            //
            (Projection::Oblique(obl), Occlusion::False) => self
                .objects
                .iter()
                .map(|obj| obj.project_oblique(&obl))
                .collect::<Vec<_>>(),
            //
            (Projection::Oblique(obl), Occlusion::True) => {
                let view_vector = Pt3d(-1.0, -1.0, -1.0);

                let mut occ = Occluder::new();

                for obj3 in self.objects.iter().sorted_by(|o1, o2| {
                    Ord::cmp(
                        &FloatOrd(o1.dist_along(&view_vector)),
                        &FloatOrd(o2.dist_along(&view_vector)),
                    )
                }) {
                    let obj2 = obj3.project_oblique(&obl);
                    occ.add(obj2.inner, obj3.inner.clone(), obj3.style.clone());
                }

                occ.export()
            }
        }
    }
}