//! A scene, i.e. a holder for 3d objects ready to be projected down onto a 2d
//! plane.

use plotz_geometry::obj2::Obj2;

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
    plotz_geometry::{style::Style, traits::Annotatable},
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

    pub fn project_with(&self, projection: Projection, occlusion: Occlusion) -> Vec<(Obj2, Style)> {
        match (projection, occlusion) {
            (Projection::Oblique(obl), Occlusion::False) => self
                .objects
                .iter()
                .map(|sobj3| obl.project_styled_obj3(sobj3))
                .collect(),

            (Projection::Oblique(obl), Occlusion::True) => {
                let mut resultant: Vec<(Obj2, Style)> = vec![];

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
                    let (obj2, style) = obl.project_styled_obj3(sobj3);

                    if let Some(SceneDebug {
                        draw_wireframes,
                        annotate: should_annotate,
                    }) = &self.debug
                    {
                        if let Some(Style {
                            color, thickness, ..
                        }) = draw_wireframes
                        {
                            resultant.push((
                                obj2.clone(),
                                Style::builder().color(color).thickness(*thickness).build(),
                            ));
                        }
                        if let Some(settings) = should_annotate {
                            resultant.extend(obj2.annotate(settings).into_iter());
                        }
                    }

                    occ.add((obj2, style));
                }
                resultant.extend(occ.export());
                resultant
            }
        }
    }
}
