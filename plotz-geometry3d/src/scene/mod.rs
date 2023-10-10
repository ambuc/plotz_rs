//! A scene, i.e. a holder for 3d objects ready to be projected down onto a 2d
//! plane.

use plotz_geometry::obj::Obj;

pub mod debug;
mod occluder;

use crate::{
    camera::{Occlusion, Projection},
    scene::{debug::SceneDebug, occluder::Occluder},
    styled_obj3::StyledObj3,
};
use float_ord::FloatOrd;
use itertools::Itertools;
use plotz_geometry::{style::Style, *};
use std::fmt::Debug;
use typed_builder::TypedBuilder;

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

    pub fn project_with(&self, projection: Projection, occlusion: Occlusion) -> Vec<(Obj, Style)> {
        match (projection, occlusion) {
            (Projection::Oblique(obl), Occlusion::False) => self
                .objects
                .iter()
                .map(|sobj3| obl.project_styled_obj3(sobj3))
                .collect(),

            (Projection::Oblique(obl), Occlusion::True) => {
                let mut resultant: Vec<(Obj, Style)> = vec![];

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
                    let (obj, style) = obl.project_styled_obj3(sobj3);

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
                                obj.clone(),
                                Style {
                                    color,
                                    thickness: *thickness,
                                    ..Default::default()
                                },
                            ));
                        }
                        if let Some(settings) = should_annotate {
                            resultant.extend(obj.annotate(settings).into_iter());
                        }
                    }

                    occ.add((obj, style));
                }
                resultant.extend(occ.export());
                resultant
            }
        }
    }
}
