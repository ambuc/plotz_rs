//! The inner value of a Object2d, i.e. the enum which holds some geometric thingy.
#![allow(missing_docs)]

use crate::{
    bounded::{Bounded, Bounds},
    crop::{CropType, Croppable, PointLocation},
    group::Group,
    overlaps::{polygon_overlaps_point, polygon_overlaps_segment},
    shapes::{
        curve::CurveArc, multiline::Multiline, point::Point, polygon::Polygon,
        polygon_with_cavity::PolygonWithCavities, segment::Segment, text::Text,
    },
    style::Style,
    Object,
};
use anyhow::Result;
use enum_dispatch::enum_dispatch;
use itertools::Itertools;
use std::{fmt::Debug, ops::*};

pub enum ObjType2d {
    // Roughly in complexity order.
    Point2d,
    Segment2d,
    Multiline2d,
    Polygon2d,
    PolygonWithCavities2d,
    CurveArc2d,
    Text2d,
    Group2d,
}

/// Either a polygon or a segment.
#[derive(Debug, PartialEq, Clone)]
#[enum_dispatch]
pub enum Obj2 {
    // Roughly in complexity order.
    Point(Point),                             // A point.
    Segment(Segment),                         // A segment.
    Multiline(Multiline),                     // A multiline.
    Polygon(Polygon),                         // A polygon.
    PolygonWithCavities(PolygonWithCavities), // A polygon with cavities.
    CurveArc(CurveArc),                       // An arc.
    Text(Text),                               // A character to be printed in SVG, at a point.
    Group(Group<Style>),                      // A group of other objects.
}

crate::ops_defaults_t!(Obj2, Point);

impl Croppable for Obj2 {
    type Output = Obj2;
    fn crop(&self, frame: &Polygon, crop_type: CropType) -> Result<Vec<Self::Output>> {
        match &self {
            Obj2::Point(p) => match polygon_overlaps_point(frame, p)? {
                Some((_, p)) => Ok(vec![Obj2::Point(p)]),
                None => Ok(vec![]),
            },
            Obj2::Segment(sg) => match polygon_overlaps_segment(frame, sg)? {
                Some((_, sgops)) => Ok(sgops.into_iter().map(|sgop| sgop.to_obj(sg)).collect()),
                None => todo!(),
            },
            Obj2::Multiline(_ml) => todo!("when polygon_overlaps_multiline is implemented."),
            Obj2::Polygon(pg) => Ok(pg
                .crop(frame, crop_type)?
                .into_iter()
                .map(Obj2::from)
                .collect::<Vec<_>>()),
            Obj2::PolygonWithCavities(_) => todo!(),
            Obj2::CurveArc(_) => todo!(),
            Obj2::Text(_) => todo!(),
            Obj2::Group(_) => todo!(),
        }
    }

    fn crop_excluding(&self, other: &Polygon) -> Result<Vec<Self::Output>>
    where
        Self: Sized,
    {
        match &self {
            Obj2::Point(pt) => {
                if matches!(other.contains_pt_deprecated(pt), Ok(PointLocation::Outside)) {
                    Ok(vec![])
                } else {
                    Ok(vec![self.clone()])
                }
            }
            Obj2::Multiline(ml) => Ok(ml
                .to_segments()
                .into_iter()
                .map(|sg| sg.crop_excluding(other))
                .flatten_ok()
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .map(Obj2::from)
                .collect::<Vec<_>>()),
            Obj2::Polygon(pg) => Ok(pg
                .crop_excluding(other)?
                .into_iter()
                .map(Obj2::from)
                .collect::<Vec<_>>()),
            Obj2::PolygonWithCavities(_) => unimplemented!("TODO: implement cropping for Pgc."),
            Obj2::Segment(sg) => Ok(sg
                .crop_excluding(other)?
                .into_iter()
                .map(Obj2::from)
                .collect::<Vec<_>>()),
            Obj2::CurveArc(ca) => Ok(ca
                .crop_excluding(other)?
                .into_iter()
                .map(Obj2::from)
                .collect::<Vec<_>>()),
            Obj2::Text(ch) => {
                if matches!(
                    other.contains_pt_deprecated(&ch.pt),
                    Ok(PointLocation::Outside)
                ) {
                    Ok(vec![])
                } else {
                    Ok(vec![self.clone()])
                }
            }
            Obj2::Group(g) => Ok(g
                .crop_excluding(other)?
                .into_iter()
                .map(Obj2::from)
                .collect::<Vec<_>>()),
        }
    }
}
