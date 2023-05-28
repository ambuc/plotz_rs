//! Occludes things. Cmon.

use {
    crate::style::Style3d,
    plotz_geometry::{crop::Croppable, obj2::Obj2, styled_obj2::StyledObj2},
};

pub struct Occluder {
    objects: Vec<(Obj2, Option<Style3d>)>,
}

impl Occluder {
    pub fn new() -> Occluder {
        Occluder { objects: vec![] }
    }

    fn hide_a_behind_b(incoming: &Obj2, existing: &Obj2) -> Vec<Obj2> {
        // TODO(jbuckland): use quadtrees here to make this MUCH faster please!!!!

        match (&incoming, &existing) {
            // points can/should be occluded, not handled yet.
            (Obj2::Pt(_), _) => {
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
            (incoming, Obj2::Sg2(_) | Obj2::Pt(_) | Obj2::Txt(_)) => {
                vec![(**incoming).clone()]
            }
        }
    }

    // Incorporates an object.
    pub fn add(&mut self, incoming: Obj2, style3d: Option<Style3d>) {
        let mut incoming_os = vec![incoming];
        for (existing_o, _) in &self.objects {
            incoming_os = incoming_os
                .iter()
                .map(|incoming_o| Occluder::hide_a_behind_b(incoming_o, &existing_o))
                .flatten()
                .collect::<Vec<_>>();
        }
        self.objects.extend(
            incoming_os
                .into_iter()
                .map(|incoming_o| (incoming_o, style3d)),
        );
    }

    // Exports the occluded 2d objects.
    pub fn export(mut self) -> Vec<StyledObj2> {
        // we store them front-to-back, but we want to render them to svg back-to-front.
        self.objects.reverse();

        self.objects
            .into_iter()
            .map(|(obj, style)| {
                let mut o = StyledObj2::new(obj);
                if let Some(Style3d { color, thickness }) = style {
                    o = o.with_color(color).with_thickness(thickness);
                }
                o
            })
            .collect()
    }
}
