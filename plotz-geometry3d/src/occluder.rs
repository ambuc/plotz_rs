//! Occludes things. Cmon.

use plotz_geometry::{object2d::Object2d, object2d_inner::Object2dInner};

use crate::{object3d_inner::Object3dInner, style::Style3d};

/// Occludes.
pub struct Occluder {
    /// Incorporated objects.
    objects: Vec<(Object2dInner, Object3dInner, Option<Style3d>)>,
}

impl Occluder {
    /// new, empty occ.
    pub fn new() -> Occluder {
        Occluder { objects: vec![] }
    }

    /// Incorporates an object.
    pub fn add(
        &mut self,
        incoming_obj2: Object2dInner,
        incoming_obj3: Object3dInner,
        style3d: Option<Style3d>,
    ) {
        dbg!(&incoming_obj2);

        let is_collision = self.objects.iter().any(|(existing_obj2, _, _)| -> bool {
            // check collision
            match (&incoming_obj2, &existing_obj2) {
                // TODO
                (Object2dInner::Polygon(pg1), Object2dInner::Polygon(pg2)) => {
                    let is = pg1.intersects_detailed(&pg2);
                    dbg!(&is);
                    !is.is_empty()
                }
                (Object2dInner::Polygon(pg), Object2dInner::Segment(sg))
                | (Object2dInner::Segment(sg), Object2dInner::Polygon(pg)) => {
                    pg.intersects_segment(&sg)
                }
                (Object2dInner::Segment(sg1), Object2dInner::Segment(sg2)) => {
                    sg1.intersects(&sg2).is_some()
                }

                // points cannot collide :)
                (Object2dInner::Point(_), _)
                | (_, Object2dInner::Point(_))
                | (Object2dInner::Char(_), _)
                | (_, Object2dInner::Char(_)) => false,

                // group collision and curve collision are "too hard" for me
                (Object2dInner::Group(_), _) | (_, Object2dInner::Group(_)) => {
                    todo!("group collision not yet implemented.")
                }
                (Object2dInner::CurveArc(_), _) | (_, Object2dInner::CurveArc(_)) => {
                    todo!("curvearc collision not yet implemented.")
                }
            }
        });

        match is_collision {
            true => {
                // do a crop
            }
            false => {
                self.objects.push((incoming_obj2, incoming_obj3, style3d));
            }
        }
    }

    /// Exports the occluded 2d objects.
    pub fn export(self) -> Vec<Object2d> {
        self.objects
            .into_iter()
            .map(|(doi, _, style)| match style {
                None => Object2d::new(doi),
                Some(Style3d { color, thickness }) => Object2d::new(doi)
                    .with_color(color)
                    .with_thickness(thickness),
            })
            .collect()
    }
}
