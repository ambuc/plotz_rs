//! A scene, i.e. a holder for 3d objects ready to be projected down onto a 2d
//! plane.

use {
    crate::{
        camera::{Occlusion, Projection},
        object3d::Object3d,
        occluder::Occluder,
        style::Style3d,
    },
    float_ord::FloatOrd,
    itertools::Itertools,
    plotz_geometry::{object2d::Object2d, traits::Annotatable, traits::AnnotationSettings},
    std::fmt::Debug,
    typed_builder::TypedBuilder,
};

/// Debug settings.
#[derive(Debug, Clone, TypedBuilder)]
pub struct DebugSettings {
    /// A style for drawing wireframes, if configured.
    #[builder(default, setter(strip_option))]
    draw_wireframes: Option<Style3d>,

    /// Whether or not to annotate everything.
    #[builder(default, setter(strip_option))]
    annotate: Option<AnnotationSettings>,
}

/// A scene of 3d objects ready to be projected down to a 2d plane.
#[derive(Debug, Clone, TypedBuilder)]
pub struct Scene {
    /// Some objects.
    #[builder(default)]
    objects: Vec<Object3d>,

    /// Some debug settings.
    #[builder(default, setter(strip_option))]
    debug: Option<DebugSettings>,
}

impl Scene {
    /// A new scene.
    pub fn new() -> Scene {
        Scene::builder().build()
    }

    /// Projects the scene onto a camera, renders to 2d, and returns a vector of object2ds.
    pub fn project_with(&self, projection: Projection, occlusion: Occlusion) -> Vec<Object2d> {
        match (projection, occlusion) {
            (Projection::Oblique(obl), Occlusion::False) => self
                .objects
                .iter()
                .map(|obj| obj.project_oblique(&obl))
                .collect(),

            (Projection::Oblique(obl), Occlusion::True) => {
                let mut resultant = vec![];

                // add objects to the occluder in distance order.
                // start at the front (so that the objects in the front can
                // remain unmodified) and work backwards.
                let mut occ = Occluder::new();

                for obj3 in self.objects.iter().sorted_by(|o1, o2| {
                    Ord::cmp(
                        &FloatOrd(o1.min_dist_along(&obl.view_vector)),
                        &FloatOrd(o2.min_dist_along(&obl.view_vector)),
                    )
                }) {
                    let obj2 = obj3.project_oblique(&obl);

                    if let Some(DebugSettings {
                        draw_wireframes,
                        annotate: should_annotate,
                    }) = &self.debug
                    {
                        if let Some(Style3d { color, thickness }) = draw_wireframes {
                            resultant
                                .push(obj2.clone().with_color(color).with_thickness(*thickness));
                        }
                        if let Some(settings) = should_annotate {
                            resultant.extend(obj2.annotate(&settings));
                        }
                    }

                    occ.add(obj2.inner, obj3.style.clone());
                }
                resultant.extend(occ.export());
                resultant
            }
        }
    }
}
