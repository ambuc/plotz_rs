//! The inner value of a Object2d, i.e. the enum which holds some geometric thingy.
#![allow(missing_docs)]

use crate::{
    bounded::{Bounded, Bounds},
    crop::{CropType, Croppable, PointLoc},
    group::Group,
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
    Point2d,
    Polygon2d,
    PolygonWithCavities2d,
    Segment2d,
    Multiline2d,
    CurveArc2d,
    Text2d,
    Group2d,
}

/// Either a polygon or a segment.
#[derive(Debug, PartialEq, Clone)]
#[enum_dispatch]
pub enum Obj {
    Point(Point),                             // A point.
    Polygon(Polygon),                         // A polygon.
    PolygonWithCavities(PolygonWithCavities), // A polygon with cavities.
    Segment(Segment),                         // A segment.
    Multiline(Multiline),                     // A multiline.
    CurveArc(CurveArc),                       // An arc.
    Text(Text),                               // A character to be printed in SVG, at a point.
    Group(Group<Style>),                      // A group of other objects.
}

crate::ops_defaults_t!(Obj, Point);

impl Croppable for Obj {
    type Output = Obj;
    fn crop(&self, frame: &Polygon, crop_type: CropType) -> Result<Vec<Self::Output>> {
        Ok(match &self {
            Obj::Point(pt) => {
                assert_eq!(crop_type, CropType::Inclusive);
                if !matches!(frame.contains_pt(pt), Ok(PointLoc::Outside)) {
                    vec![self.clone()]
                } else {
                    vec![]
                }
            }
            Obj::Multiline(ml) => ml
                .to_segments()
                .into_iter()
                .map(|sg| sg.crop(frame, crop_type))
                .flatten_ok()
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .map(Obj::from)
                .collect::<Vec<_>>(),
            Obj::Polygon(pg) => pg
                .crop(frame, crop_type)?
                .into_iter()
                .map(Obj::from)
                .collect::<Vec<_>>(),
            Obj::PolygonWithCavities(_) => unimplemented!("TODO: implement cropping for Pgc."),
            Obj::Segment(sg) => sg
                .crop(frame, crop_type)?
                .into_iter()
                .map(Obj::from)
                .collect::<Vec<_>>(),
            Obj::CurveArc(ca) => ca
                .crop(frame, crop_type)?
                .into_iter()
                .map(Obj::from)
                .collect::<Vec<_>>(),
            Obj::Text(ch) => {
                assert_eq!(crop_type, CropType::Inclusive);
                if !matches!(frame.contains_pt(&ch.pt), Ok(PointLoc::Outside)) {
                    vec![self.clone()]
                } else {
                    vec![]
                }
            }
            Obj::Group(g) => g
                .crop(frame, crop_type)?
                .into_iter()
                .map(Obj::from)
                .collect::<Vec<_>>(),
        })
    }

    fn crop_excluding(&self, other: &Polygon) -> Result<Vec<Self::Output>>
    where
        Self: Sized,
    {
        match &self {
            Obj::Point(pt) => {
                if matches!(other.contains_pt(pt), Ok(PointLoc::Outside)) {
                    Ok(vec![])
                } else {
                    Ok(vec![self.clone()])
                }
            }
            Obj::Multiline(ml) => Ok(ml
                .to_segments()
                .into_iter()
                .map(|sg| sg.crop_excluding(other))
                .flatten_ok()
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .map(Obj::from)
                .collect::<Vec<_>>()),
            Obj::Polygon(pg) => Ok(pg
                .crop_excluding(other)?
                .into_iter()
                .map(Obj::from)
                .collect::<Vec<_>>()),
            Obj::PolygonWithCavities(_) => unimplemented!("TODO: implement cropping for Pgc."),
            Obj::Segment(sg) => Ok(sg
                .crop_excluding(other)?
                .into_iter()
                .map(Obj::from)
                .collect::<Vec<_>>()),
            Obj::CurveArc(ca) => Ok(ca
                .crop_excluding(other)?
                .into_iter()
                .map(Obj::from)
                .collect::<Vec<_>>()),
            Obj::Text(ch) => {
                if matches!(other.contains_pt(&ch.pt), Ok(PointLoc::Outside)) {
                    Ok(vec![])
                } else {
                    Ok(vec![self.clone()])
                }
            }
            Obj::Group(g) => Ok(g
                .crop_excluding(other)?
                .into_iter()
                .map(Obj::from)
                .collect::<Vec<_>>()),
        }
    }
}
