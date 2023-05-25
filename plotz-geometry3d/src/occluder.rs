//! Occludes things. Cmon.

use {
    crate::style::Style3d,
    plotz_geometry::{crop::Croppable, object2d::Object2d, object2d_inner::Object2dInner},
};

/// Occludes.
pub struct Occluder {
    /// Incorporated objects.
    objects: Vec<(Object2dInner, Option<Style3d>)>,
}

impl Occluder {
    /// new, empty occ.
    pub fn new() -> Occluder {
        Occluder { objects: vec![] }
    }

    fn hide_a_behind_b(incoming: &Object2dInner, existing: &Object2dInner) -> Vec<Object2dInner> {
        // TODO(jbuckland): use quadtrees here to make this MUCH faster please!!!!

        match (&incoming, &existing) {
            // points can/should be occluded, not handled yet.
            (Object2dInner::Point(_), _) => {
                unimplemented!("no support for points yet")
            }
            // chars are points, see above.
            (Object2dInner::Char(_), _) => {
                unimplemented!("no support for chars yet")
            }
            // groups are not handled yet.
            (Object2dInner::Group(_), _) | (_, Object2dInner::Group(_)) => {
                unimplemented!("no support for groups yet")
            }
            // curvearcs are not handled yet.
            (Object2dInner::CurveArc(_), _) | (_, Object2dInner::CurveArc(_)) => {
                unimplemented!("no support for curvearcs yet")
            }

            (Object2dInner::Polygon(pg1), Object2dInner::Polygon(pg2)) => {
                pg1.crop_excluding(pg2)
                    .into_iter()
                    .map(Object2dInner::from)
                    .collect()
            }
            (Object2dInner::Segment(_sg), Object2dInner::Polygon(_pg)) => {
                unimplemented!("no support for pg x sg yet");
            }

            //
            // you can't hide something behind a segment or a point or a char. don't be daft.
            (
                incoming,
                Object2dInner::Segment(_) | Object2dInner::Point(_) | Object2dInner::Char(_),
            ) => {
                vec![(**incoming).clone()]
            }
        }
    }

    /// Incorporates an object.
    pub fn add(&mut self, incoming: Object2dInner, style3d: Option<Style3d>) {
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

    /// Exports the occluded 2d objects.
    pub fn export(mut self) -> Vec<Object2d> {
        // we store them front-to-back, but we want to render them to svg back-to-front.
        self.objects.reverse();

        self.objects
            .into_iter()
            .map(|(obj, style)| {
                let mut o = Object2d::new(obj);
                if let Some(Style3d { color, thickness }) = style {
                    o = o.with_color(color).with_thickness(thickness);
                }
                o
            })
            .collect()
    }
}
