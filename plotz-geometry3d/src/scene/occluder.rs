//! Occludes things. Cmon.

use anyhow::*;
use itertools::Itertools;
use plotz_geometry::{crop::Croppable, obj::Obj2, shading::shade_polygon, style::Style};
use tracing::*;
use typed_builder::TypedBuilder;

#[derive(Debug, Clone, Copy, Default, TypedBuilder)]
pub struct OccluderConfig {
    #[builder(default)]
    pub color_according_to_depth: Option<&'static colorgrad::Gradient>,
}

#[derive(Debug, Default, Clone, TypedBuilder)]
pub struct Occluder {
    #[builder(default)]
    pub config: OccluderConfig,

    #[builder(default)]
    pub objects: Vec<(Obj2, Style)>,
}

// Despite the name, this really only layers A atop B atop C and computes their
// crops. Maybe a better name would be |Obscurer|. Anyway.
impl Occluder {
    fn hide_a_behind_b(incoming: &Obj2, existing: &Obj2) -> Result<Vec<Obj2>> {
        // TODO(https://github.com/ambuc/plotz_rs/issues/3): use quadtrees here to make this MUCH faster please!!!!

        match (&incoming, &existing) {
            // points can/should be occluded, not handled yet.
            (Obj2::Point(_), _) => {
                unimplemented!("no support for points yet")
            }
            // chars are points, see above.
            (Obj2::Text(_), _) => {
                unimplemented!("no support for chars yet")
            }
            // groups are not handled yet.
            (Obj2::Group(_), _) | (_, Obj2::Group(_)) => {
                unimplemented!("no support for groups yet")
            }
            // curvearcs are not handled yet.
            (Obj2::CurveArc(_), _) | (_, Obj2::CurveArc(_)) => {
                unimplemented!("no support for curvearcs yet")
            }

            (Obj2::Multiline(_), _) => {
                unimplemented!("no support for multilines yet")
            }

            (Obj2::Polygon(a), Obj2::Polygon(b)) => Ok(a
                .crop_excluding(b)
                .context(format!("crop excluding: \na\n\t{:?}\n\nb\n\t{:?}", a, b))?
                .into_iter()
                .map(Obj2::from)
                .collect()),
            (Obj2::Segment(_sg), Obj2::Polygon(_pg)) => {
                unimplemented!("no support for pg x sg yet");
            }

            (Obj2::PolygonWithCavities(_), _) | (_, Obj2::PolygonWithCavities(_)) => {
                unimplemented!("TODO(jbuckland): implement cropping.")
            }

            //
            // you can't hide something behind a segment or a point or a char. don't be daft.
            (incoming, Obj2::Multiline(_) | Obj2::Segment(_) | Obj2::Point(_) | Obj2::Text(_)) => {
                Ok(vec![(**incoming).clone()])
            }
        }
    }

    // Incorporates an object
    #[instrument(skip(self, incoming2))]
    pub fn add(&mut self, incoming2: (Obj2, Style)) -> Result<()> {
        let mut incoming_os: Vec<(Obj2, Style)> = vec![incoming2.clone()];
        for (existing_o, _) in &self.objects {
            incoming_os = incoming_os
                .iter()
                .map(|(incoming_obj, _)| {
                    Occluder::hide_a_behind_b(incoming_obj, existing_o)
                        .context("occluding a behind b")
                })
                .flatten_ok()
                .collect::<Result<Vec<_>>>()
                .context("collecting objects")?
                .into_iter()
                .map(|obj| (obj, incoming2.1))
                .collect::<Vec<_>>();
        }
        self.objects.extend(incoming_os);
        Ok(())
    }

    // Exports the occluded 2d objects.
    #[instrument(skip(self))]
    pub fn export(mut self) -> Result<Vec<(Obj2, Style)>> {
        // we store them front-to-back, but we want to render them to svg back-to-front.
        self.objects.reverse();
        let x: Vec<_> = self
            .objects
            .into_iter()
            .map(export_obj)
            .flatten_ok()
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .collect();
        Ok(x)
    }
}

#[instrument]
fn export_obj((sobj, style): (Obj2, Style)) -> Result<Vec<(Obj2, Style)>> {
    match style {
        Style { shading: None, .. } => Ok(vec![(sobj, style)]),
        style @ Style {
            shading: Some(shade_config),
            ..
        } => match sobj {
            Obj2::Polygon(pg) => {
                if shade_config.along_face {
                    // TODO(https://github.com/ambuc/plotz_rs/issues/2): apply shade config here.
                    Ok(vec![])
                } else {
                    Ok(shade_polygon(&shade_config, &pg)
                        .unwrap()
                        .into_iter()
                        .map(|sg| (sg.into(), style))
                        .collect::<Vec<_>>())
                }
            }
            _ => Err(anyhow!("can't shade not a polygon")),
        },
    }
}
