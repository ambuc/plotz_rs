//! Occludes things. Cmon.

use plotz_geometry::{
    draw_obj::DrawObj, draw_obj_inner::DrawObjInner, segment::IntersectionOutcome,
};

use crate::{object_inner::ObjectInner, style::Style};

/// Occludes.
pub struct Occluder {
    /// Incorporated objects. 
    objects: Vec<(DrawObjInner, ObjectInner, Option<Style>)>,
}

impl Occluder {
    /// new, empty occ.
    pub fn new() -> Occluder {
        Occluder { objects: vec![] }
    }

    /// Incorporates an object.
    pub fn add(
        &mut self,
        incoming_obj2: DrawObjInner,
        incoming_obj3: ObjectInner,
        style: Option<Style>,
    ) {
        // let is_collision = self.objects.iter().any(|(existing_obj2, _, _)| -> bool {
        //     // check collision
        //     match (&incoming_obj2, &existing_obj2) {
        //         // TODO
        //         (DrawObjInner::Polygon(pg1), DrawObjInner::Polygon(pg2)) => {
        //             pg1.intersects(&pg2)
        //         }
        //         (DrawObjInner::Polygon(pg), DrawObjInner::Segment(sg))
        //         | (DrawObjInner::Segment(sg), DrawObjInner::Polygon(pg)) => {
        //             pg.intersects_segment(&sg)
        //         }
        //         (DrawObjInner::Segment(sg1), DrawObjInner::Segment(sg2)) => {
        //             match sg1.intersects(&sg2) {
        //                 Some(IntersectionOutcome::LineSegmentsAreColinear) => false,
        //                 Some(IntersectionOutcome::LineSegmentsAreTheSame) => false,
        //                 Some(IntersectionOutcome::LineSegmentsAreTheSameButReversed) => false,
        //                 Some(IntersectionOutcome::Yes(_)) => false,
        //                 None => false,
        //             }
        //         }

        //         // points cannot collide :)
        //         (DrawObjInner::Point(_), _)
        //         | (_, DrawObjInner::Point(_))
        //         | (DrawObjInner::Char(_), _)
        //         | (_, DrawObjInner::Char(_)) => false,

        //         // group collision and curve collision are "too hard" for me
        //         (DrawObjInner::Group(_), _) | (_, DrawObjInner::Group(_)) => {
        //             todo!("group collision not yet implemented.")
        //         }
        //         (DrawObjInner::CurveArc(_), _) | (_, DrawObjInner::CurveArc(_)) => {
        //             todo!("curvearc collision not yet implemented.")
        //         }
        //     }
        // });

        // match is_collision {
        //     true => {
        //         // do a crop
        //     }
        //     false => {
        self.objects.push((incoming_obj2, incoming_obj3, style));
        //     }
        // }
    }

    /// Exports the occluded 2d objects.
    pub fn export(self) -> Vec<DrawObj> {
        self.objects
            .into_iter()
            .map(|(doi, _, style)| match style {
                None => DrawObj::new(doi),
                Some(Style { color, thickness }) => DrawObj::new(doi)
                    .with_color(color)
                    .with_thickness(thickness),
            })
            .collect()
    }
}
