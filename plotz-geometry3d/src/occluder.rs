//! Occludes things. Cmon.

use plotz_color::*;
use plotz_geometry::{
    crop::Croppable, isxn::IsxnResult, object2d::Object2d, object2d_inner::Object2dInner,
};

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

    fn hide_a_behind_b(incoming: &Object2dInner, existing: &Object2dInner) -> Vec<Object2dInner> {
        // TODO(jbuckland): use quadtrees here to make this MUCH faster please!!!!
        // dbg!("incoming {:?} existing {:?}", incoming, existing);

        match (&incoming, &existing) {
            // points can/should be occluded, not handled yet.
            (Object2dInner::Point(_), _) | (_, Object2dInner::Point(_)) => {
                unimplemented!("no support for points yet")
            }
            // chars are points, see above.
            (Object2dInner::Char(_), _) | (_, Object2dInner::Char(_)) => {
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
                let is_collision =
                    pg1.intersects_detailed(&pg2)
                        .iter()
                        .any(|isxn_result| match isxn_result {
                            IsxnResult::MultipleIntersections(_) => false,
                            IsxnResult::OneIntersection(isxn) => {
                                if isxn.on_points_of_either() {
                                    false
                                } else {
                                    true
                                }
                            }
                        });

                if is_collision {
                    incoming.crop_to(pg2).expect("crop failed")
                } else {
                    vec![incoming.clone()]
                }
            }
            (Object2dInner::Polygon(pg), Object2dInner::Segment(sg))
            | (Object2dInner::Segment(sg), Object2dInner::Polygon(pg)) => {
                unimplemented!("no seg-to-polygon cropping");
                // let is_collision = pg
                //     .intersects_segment_detailed(&sg)
                //     .iter()
                //     .any(|isxn_result| match isxn_result {
                //         IsxnResult::MultipleIntersections(_) => false,
                //         IsxnResult::OneIntersection(isxn) => {
                //             if isxn.on_points_of_either() {
                //                 false
                //             } else {
                //                 true
                //             }
                //         }
                //     });
                // if is_collision {
                //     sg.crop_to(pg)
                //         .expect("crop failed")
                //         .into_iter()
                //         .map(|sg| Object2dInner::from(sg))
                //         .collect()
                // } else {
                //     vec![incoming.clone()]
                // }
            }
            (Object2dInner::Segment(sg1), Object2dInner::Segment(sg2)) => {
                unimplemented!("no seg-to-seg cropping");
                // let is_collision = match sg1.intersects(&sg2) {
                //     None => false,
                //     Some(IsxnResult::MultipleIntersections(_)) => false,
                //     Some(IsxnResult::OneIntersection(isxn)) => !isxn.on_points_of_either(),
                // };
                // if is_collision {
                //     // TODO(ambuc): implement cropping here
                //     unimplemented!("have not yet implemented 3d sg/sg crop");
                // } else {
                //     vec![incoming.clone()]
                // }
            }
        }
    }

    /// Incorporates an object.
    pub fn add(
        &mut self,
        incoming_obj2: Object2dInner,
        incoming_obj3: Object3dInner,
        _style3d: Option<Style3d>,
    ) {
        // debug
        self.objects.push((
            incoming_obj2.clone(),
            incoming_obj3.clone(),
            Some(Style3d::builder().color(&RED).thickness(0.05).build()),
        ));

        // if the collision is parallel, don't crop.
        // if the collision exists at a point, don't crop.
        // otherwise, there is a collision!

        self.objects
            .iter()
            .fold(
                // One incoming object.
                vec![incoming_obj2],
                // a set of incoming (reduced) objects, and a single existing object.
                |acc, (existing_obj2, _, _)| -> Vec<Object2dInner> {
                    acc.into_iter()
                        .map(|reduced_obj2| {
                            Occluder::hide_a_behind_b(&reduced_obj2, &existing_obj2)
                        })
                        .flatten()
                        .collect::<Vec<_>>()
                },
            )
            .into_iter()
            .for_each(|new_obj2| {
                self.objects.push((
                    new_obj2,
                    incoming_obj3.clone(),
                    Some(Style3d::builder().color(&GREEN).thickness(2.0).build()),
                ));
            });
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
