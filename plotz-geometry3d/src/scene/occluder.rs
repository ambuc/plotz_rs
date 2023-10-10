//! Occludes things. Cmon.

use plotz_geometry::{crop::Croppable, obj::Obj, shading::shade_polygon, style::Style};

pub struct Occluder {
    objects: Vec<(Obj, Style)>,
}

impl Occluder {
    pub fn new() -> Occluder {
        Occluder { objects: vec![] }
    }

    fn hide_a_behind_b(incoming: &Obj, existing: &Obj) -> Vec<Obj> {
        // TODO(jbuckland): use quadtrees here to make this MUCH faster please!!!!

        match (&incoming, &existing) {
            // points can/should be occluded, not handled yet.
            (Obj::Pt(_), _) => {
                unimplemented!("no support for points yet")
            }
            // chars are points, see above.
            (Obj::Txt(_), _) => {
                unimplemented!("no support for chars yet")
            }
            // groups are not handled yet.
            (Obj::Group(_), _) | (_, Obj::Group(_)) => {
                unimplemented!("no support for groups yet")
            }
            // curvearcs are not handled yet.
            (Obj::CurveArc(_), _) | (_, Obj::CurveArc(_)) => {
                unimplemented!("no support for curvearcs yet")
            }

            (Obj::Pg(a), Obj::Pg(b)) => a.crop_excluding(b).into_iter().map(Obj::from).collect(),
            (Obj::Sg(_sg), Obj::Pg(_pg)) => {
                unimplemented!("no support for pg x sg yet");
            }

            //
            // you can't hide something behind a segment or a point or a char. don't be daft.
            (incoming, Obj::Sg(_) | Obj::Pt(_) | Obj::Txt(_)) => {
                vec![(**incoming).clone()]
            }
        }
    }

    // Incorporates an object.
    pub fn add(&mut self, incoming2: (Obj, Style)) {
        let mut incoming_os: Vec<(Obj, Style)> = vec![incoming2.clone()];
        for (existing_o, _) in &self.objects {
            incoming_os = incoming_os
                .iter()
                .flat_map(|(incoming_obj, _)| Occluder::hide_a_behind_b(incoming_obj, existing_o))
                .map(|obj| (obj, incoming2.1))
                .collect::<Vec<_>>();
        }
        self.objects.extend(incoming_os);
    }

    // Exports the occluded 2d objects.
    pub fn export(mut self) -> Vec<(Obj, Style)> {
        // we store them front-to-back, but we want to render them to svg back-to-front.
        self.objects.reverse();
        self.objects.into_iter().flat_map(export_obj).collect()
    }
}

fn export_obj((sobj, style): (Obj, Style)) -> Vec<(Obj, Style)> {
    match style {
        Style { shading: None, .. } => {
            vec![(sobj, style)]
        }
        style @ Style {
            shading: Some(shade_config),
            ..
        } => match sobj {
            Obj::Pg(pg) => {
                if shade_config.along_face {
                    // TODO(jbuckland): apply shade config here.
                    // TODO(jbuckland): apply shade config here.
                    // TODO(jbuckland): apply shade config here.
                    // TODO(jbuckland): apply shade config here.
                    // TODO(jbuckland): apply shade config here.
                    // TODO(jbuckland): apply shade config here.
                    vec![]
                } else {
                    shade_polygon(&shade_config, &pg)
                        .unwrap()
                        .into_iter()
                        .map(|sg| (sg.into(), style))
                        .collect::<Vec<_>>()
                }
            }
            _ => {
                panic!("can't shade not a polygon.")
            }
        },
    }
}
