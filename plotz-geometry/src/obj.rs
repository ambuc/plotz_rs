//! The inner value of a Object2d, i.e. the enum which holds some geometric thingy.
#![allow(missing_docs)]

use crate::{
    bounded::{Bounded, Bounds},
    crop::{CropType, Croppable, PointLoc},
    group::Group,
    shapes::{curve::CurveArc, ml::Ml, pg::Pg, pgc::Pgc, pt::Pt, sg::Sg, txt::Txt},
    style::Style,
};
use anyhow::Result;
use enum_dispatch::enum_dispatch;
use itertools::Itertools;
use std::{fmt::Debug, ops::*};

pub enum ObjType {
    Point,
    Polygon,
    PolygonWithCavities,
    Segment,
    Multiline,
    CurveArc,
    Text,
    Group,
}

/// Either a polygon or a segment.
#[derive(Debug, PartialEq, Clone)]
#[enum_dispatch]
pub enum Obj {
    Pt(Pt),              // A point.
    Pg(Pg),              // A polygon.
    Pgc(Pgc),            // A polygon with cavities.
    Sg(Sg),              // A segment.
    Ml(Ml),              // A multiline.
    CurveArc(CurveArc),  // An arc.
    Txt(Txt),            // A character to be printed in SVG, at a point.
    Group(Group<Style>), // A group of other objects.
}

macro_rules! impl_ops {
    ($trait:ident, $fn:ident, $rhs:ident) => {
        impl<T> $trait<T> for Obj
        where
            T: Into<$rhs>,
        {
            type Output = Obj;
            fn $fn(self, rhs: T) -> Self::Output {
                let rhs = rhs.into();
                match self {
                    Obj::CurveArc(x) => Obj::from(x.$fn(rhs)),
                    Obj::Group(x) => Obj::from(x.$fn(rhs)),
                    Obj::Ml(x) => Obj::from(x.$fn(rhs)),
                    Obj::Pg(x) => Obj::from(x.$fn(rhs)),
                    Obj::Pgc(x) => Obj::from(x.$fn(rhs)),
                    Obj::Pt(x) => Obj::from(x.$fn(rhs)),
                    Obj::Sg(x) => Obj::from(x.$fn(rhs)),
                    Obj::Txt(x) => Obj::from(x.$fn(rhs)),
                }
            }
        }
    };
}

macro_rules! impl_ops_assign {
    ($trait:ident, $fn:ident, $rhs:ident) => {
        impl<T> $trait<T> for Obj
        where
            T: Into<$rhs>,
        {
            fn $fn(&mut self, rhs: T) {
                let rhs = rhs.into();
                match self {
                    Obj::CurveArc(x) => {
                        x.$fn(rhs);
                    }
                    Obj::Group(x) => {
                        x.$fn(rhs);
                    }
                    Obj::Ml(x) => {
                        x.$fn(rhs);
                    }
                    Obj::Pg(x) => {
                        x.$fn(rhs);
                    }
                    Obj::Pgc(x) => {
                        x.$fn(rhs);
                    }
                    Obj::Pt(x) => {
                        x.$fn(rhs);
                    }
                    Obj::Sg(x) => {
                        x.$fn(rhs);
                    }
                    Obj::Txt(x) => {
                        x.$fn(rhs);
                    }
                }
            }
        }
    };
}

impl_ops!(Add, add, Pt);
impl_ops!(Div, div, f64);
impl_ops!(Mul, mul, f64);
impl_ops!(Sub, sub, Pt);
impl_ops_assign!(AddAssign, add_assign, Pt);
impl_ops_assign!(DivAssign, div_assign, f64);
impl_ops_assign!(MulAssign, mul_assign, f64);
impl_ops_assign!(RemAssign, rem_assign, Pt);
impl_ops_assign!(SubAssign, sub_assign, Pt);

impl Croppable for Obj {
    type Output = Obj;
    fn crop(&self, frame: &Pg, crop_type: CropType) -> Result<Vec<Self::Output>> {
        Ok(match &self {
            Obj::Pt(pt) => {
                assert_eq!(crop_type, CropType::Inclusive);
                if !matches!(frame.contains_pt(pt), Ok(PointLoc::Outside)) {
                    vec![self.clone()]
                } else {
                    vec![]
                }
            }
            Obj::Ml(ml) => ml
                .to_segments()
                .into_iter()
                .map(|sg| sg.crop(frame, crop_type))
                .flatten_ok()
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .map(Obj::from)
                .collect::<Vec<_>>(),
            Obj::Pg(pg) => pg
                .crop(frame, crop_type)?
                .into_iter()
                .map(Obj::from)
                .collect::<Vec<_>>(),
            Obj::Pgc(_) => unimplemented!("TODO: implement cropping for Pgc."),
            Obj::Sg(sg) => sg
                .crop(frame, crop_type)?
                .into_iter()
                .map(Obj::from)
                .collect::<Vec<_>>(),
            Obj::CurveArc(ca) => ca
                .crop(frame, crop_type)?
                .into_iter()
                .map(Obj::from)
                .collect::<Vec<_>>(),
            Obj::Txt(ch) => {
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

    fn crop_excluding(&self, other: &Pg) -> Result<Vec<Self::Output>>
    where
        Self: Sized,
    {
        match &self {
            Obj::Pt(pt) => {
                if matches!(other.contains_pt(pt), Ok(PointLoc::Outside)) {
                    Ok(vec![])
                } else {
                    Ok(vec![self.clone()])
                }
            }
            Obj::Ml(ml) => Ok(ml
                .to_segments()
                .into_iter()
                .map(|sg| sg.crop_excluding(other))
                .flatten_ok()
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .map(Obj::from)
                .collect::<Vec<_>>()),
            Obj::Pg(pg) => Ok(pg
                .crop_excluding(other)?
                .into_iter()
                .map(Obj::from)
                .collect::<Vec<_>>()),
            Obj::Pgc(_) => unimplemented!("TODO: implement cropping for Pgc."),
            Obj::Sg(sg) => Ok(sg
                .crop_excluding(other)?
                .into_iter()
                .map(Obj::from)
                .collect::<Vec<_>>()),
            Obj::CurveArc(ca) => Ok(ca
                .crop_excluding(other)?
                .into_iter()
                .map(Obj::from)
                .collect::<Vec<_>>()),
            Obj::Txt(ch) => {
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
