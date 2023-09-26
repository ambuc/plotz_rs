//! A scene, i.e. a holder for 3d objects ready to be projected down onto a 2d
//! plane.

pub mod debug;
mod occluder;

use {
    crate::{
        camera::{Occlusion, Projection},
        scene::{debug::SceneDebug, occluder::Occluder},
        styled_obj3::StyledObj3,
    },
    float_ord::FloatOrd,
    itertools::Itertools,
    plotz_geometry::{style::Style, styled_obj2::StyledObj2, traits::Annotatable},
    std::fmt::Debug,
    typed_builder::TypedBuilder,
};

#[derive(Debug, Clone, TypedBuilder)]
pub struct Scene {
    #[builder(default)]
    objects: Vec<StyledObj3>,

    #[builder(default, setter(strip_option))]
    debug: Option<SceneDebug>,
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene {
    pub fn new() -> Scene {
        Scene::builder().build()
    }

    pub fn project_with(&self, projection: Projection, occlusion: Occlusion) -> Vec<StyledObj2> {
        match (projection, occlusion) {
            (Projection::Oblique(obl), Occlusion::False) => self
                .objects
                .iter()
                .map(|sobj3| obl.project_styled_obj3(sobj3))
                .collect(),

            (Projection::Oblique(obl), Occlusion::True) => {
                let mut resultant = vec![];

                // add objects to the occluder in distance order.
                // start at the front (so that the objects in the front can
                // remain unmodified) and work backwards.
                let mut occ = Occluder::new();

                for sobj3 in self.objects.iter().sorted_by(|o1, o2| {
                    Ord::cmp(
                        &FloatOrd(o1.inner.min_dist_along(&obl.view_vector())),
                        &FloatOrd(o2.inner.min_dist_along(&obl.view_vector())),
                    )
                }) {
                    let sobj2 = obl.project_styled_obj3(sobj3);

                    if let Some(SceneDebug {
                        draw_wireframes,
                        annotate: should_annotate,
                    }) = &self.debug
                    {
                        if let Some(Style {
                            color, thickness, ..
                        }) = draw_wireframes
                        {
                            resultant
                                .push(sobj2.clone().with_color(color).with_thickness(*thickness));
                        }
                        if let Some(settings) = should_annotate {
                            resultant.extend(sobj2.annotate(settings));
                        }
                    }

                    occ.add(sobj2.clone());
                }
                resultant.extend(occ.export());
                resultant
            }
        }
    }
}
