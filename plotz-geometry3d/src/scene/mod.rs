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

impl Scene {
    pub fn new() -> Scene {
        Scene::builder().build()
    }

    pub fn project_with(&self, projection: Projection, occlusion: Occlusion) -> Vec<StyledObj2> {
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
                        &FloatOrd(o1.inner.min_dist_along(&obl.view_vector())),
                        &FloatOrd(o2.inner.min_dist_along(&obl.view_vector())),
                    )
                }) {
                    let obj2 = obj3.project_oblique(&obl);

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
                                .push(obj2.clone().with_color(color).with_thickness(*thickness));
                        }
                        if let Some(settings) = should_annotate {
                            resultant.extend(obj2.annotate(&settings));
                        }
                    }

                    occ.add(obj3.inner.clone(), obj2.inner, obj3.style.clone());
                }
                resultant.extend(occ.export());
                resultant
            }
        }
    }
}
