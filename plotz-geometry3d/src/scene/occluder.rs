//! Occludes things. Cmon.

use plotz_geometry::{
    crop::Croppable, obj2::Obj2, shading::shade_polygon, style::Style, styled_obj2::StyledObj2,
};

pub struct Occluder {
    objects: Vec<(Obj2, Style)>,
}

impl Occluder {
    pub fn new() -> Occluder {
        Occluder { objects: vec![] }
    }

    fn hide_a_behind_b(incoming: &Obj2, existing: &Obj2) -> Vec<Obj2> {
        // TODO(jbuckland): use quadtrees here to make this MUCH faster please!!!!

        match (&incoming, &existing) {
            // points can/should be occluded, not handled yet.
            (Obj2::Pt2(_), _) => {
                unimplemented!("no support for points yet")
            }
            // chars are points, see above.
            (Obj2::Txt(_), _) => {
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

            (Obj2::Pg2(pg1), Obj2::Pg2(pg2)) => pg1
                .crop_excluding(pg2)
                .into_iter()
                .map(Obj2::from)
                .collect(),
            (Obj2::Sg2(_sg), Obj2::Pg2(_pg)) => {
                unimplemented!("no support for pg x sg yet");
            }

            //
            // you can't hide something behind a segment or a point or a char. don't be daft.
            (incoming, Obj2::Sg2(_) | Obj2::Pt2(_) | Obj2::Txt(_)) => {
                vec![(**incoming).clone()]
            }
        }
    }

    // Incorporates an object.
    pub fn add(&mut self, incoming2: (Obj2, Style)) {
        let mut incoming_os: Vec<(Obj2, Style)> = vec![incoming2.clone()];
        for (existing_o, _) in &self.objects {
            incoming_os = incoming_os
                .iter()
                .flat_map(|(incoming_obj2, _)| Occluder::hide_a_behind_b(incoming_obj2, existing_o))
                .map(|obj2| (obj2, incoming2.1))
                .collect::<Vec<_>>();
        }
        self.objects.extend(incoming_os);
    }

    // Exports the occluded 2d objects.
    pub fn export(mut self) -> Vec<StyledObj2> {
        // we store them front-to-back, but we want to render them to svg back-to-front.
        self.objects.reverse();
        self.objects
            .into_iter()
            .map(|(inner, style)| StyledObj2 { inner, style })
            .flat_map(export_obj)
            .collect()
    }
}

fn export_obj(sobj2: StyledObj2) -> Vec<StyledObj2> {
    match sobj2.style {
        Style { shading: None, .. } => {
            vec![sobj2]
        }
        style @ Style {
            shading: Some(shade_config),
            ..
        } => match sobj2.inner {
            Obj2::Pg2(pg2) => {
                if shade_config.along_face {
                    // TODO(jbuckland): apply shade config here.
                    // TODO(jbuckland): apply shade config here.
                    // TODO(jbuckland): apply shade config here.
                    // TODO(jbuckland): apply shade config here.
                    // TODO(jbuckland): apply shade config here.
                    // TODO(jbuckland): apply shade config here.
                    vec![]
                } else {
                    shade_polygon(&shade_config, &pg2)
                        .unwrap()
                        .into_iter()
                        .map(|sg2| StyledObj2::new(sg2).with_style(style))
                        .collect::<Vec<_>>()
                }
            }
            _ => {
                panic!("can't shade not a polygon.")
            }
        },
    }
}
