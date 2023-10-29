//! A scene, i.e. a holder for 3d objects ready to be projected down onto a 2d
//! plane.

pub mod debug;
pub mod occluder;

use crate::{
    camera::Projection,
    obj3::Obj3,
    scene::{debug::SceneDebug, occluder::Occluder},
};
use anyhow::*;
use float_ord::FloatOrd;
use itertools::Itertools;
use plotz_color::ColorRGB;
use plotz_geometry::{obj::Obj, style::Style, *};
use std::fmt::Debug;
use tracing::*;
use typed_builder::TypedBuilder;

#[derive(Debug, Clone, Default, TypedBuilder)]
pub struct Scene {
    #[builder(default)]
    objects: Vec<(Obj3, Style)>,

    #[builder(default, setter(strip_option))]
    debug: Option<SceneDebug>,

    #[builder(default, setter(strip_option))]
    occluder: Option<Occluder>,

    #[builder(default)]
    projection: Projection,
}

impl Scene {
    #[instrument(skip(self))]
    pub fn project(self) -> Result<Vec<(Obj, Style)>> {
        match (self.projection, self.occluder) {
            (Projection::Oblique(obl), None) => Ok(self
                .objects
                .iter()
                .map(|sobj3| obl.project_styled_obj3(sobj3))
                .collect()),

            (Projection::Oblique(obl), Some(mut occluder)) => {
                let mut resultant: Vec<(Obj, Style)> = vec![];

                // add objects to the occluder in distance order.
                // start at the front (so that the objects in the front can
                // remain unmodified) and work backwards.

                let mut sorted_objs: Vec<(Obj3, Style)> = self
                    .objects
                    .into_iter()
                    .sorted_by(|(o1, _), (o2, _)| {
                        Ord::cmp(
                            &FloatOrd(o1.min_dist_along(&obl.view_vector())),
                            &FloatOrd(o2.min_dist_along(&obl.view_vector())),
                        )
                    })
                    .collect();

                // TODO(https://github.com/ambuc/plotz_rs/issues/1):
                // OK, here is the place to fix -- we don't want to sort by any
                // individual min or max. we want to take two obj3s, proj them
                // to obj2s, and find if they intersect.  if they don't
                // intersect, then the ordering doesn't matter (i think this
                // means we can order them by dist to obj center). if they do
                // intersect, then we need to look at the intersection point and
                // figure out if o1 or o2 is in front _at that point_ and order
                // by that.

                // optionally color according to depth.
                if let Some(x) = occluder.config.color_according_to_depth {
                    let length = sorted_objs.len();

                    for (i, (_, s)) in sorted_objs.iter_mut().enumerate() {
                        let pct: f64 = (i as f64) / (length as f64);
                        //
                        let c = x.at(pct);
                        s.color = ColorRGB {
                            r: c.r,
                            g: c.g,
                            b: c.b,
                        };
                    }
                }

                for sobj3 in sorted_objs {
                    let (obj, style) = obl.project_styled_obj3(&sobj3);

                    if let Some(SceneDebug {
                        draw_wireframes,
                        annotate: should_annotate,
                        ..
                    }) = &self.debug
                    {
                        if let Some(Style {
                            color, thickness, ..
                        }) = draw_wireframes
                        {
                            resultant.push((
                                obj.clone(),
                                Style {
                                    color: *color,
                                    thickness: *thickness,
                                    ..Default::default()
                                },
                            ));
                        }
                        if let Some(settings) = should_annotate {
                            resultant.extend(obj.annotate(settings).into_iter());
                        }
                    }

                    let dbg = format!("adding object:\n\t{:?}", &obj);
                    occluder.add((obj, style)).context(dbg)?;
                }
                resultant.extend(occluder.export()?);
                Ok(resultant)
            }
        }
    }
}
