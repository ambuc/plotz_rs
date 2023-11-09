//! The inner value of a Object2d, i.e. the enum which holds some geometric thingy.
#![allow(missing_docs)]

use crate::{
    bounded::{Bounded, Bounds},
    crop::{CropType, Croppable, PointLoc},
    group::Group,
    shapes::{curve::CurveArc, ml::Ml, pg::Pg, pgc::Pgc, pt::Pt, sg::Sg, txt::Txt},
    style::Style,
    *,
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
    /// A point.
    Pt(Pt),
    /// A polygon.
    Pg(Pg),
    /// A polygon with cavities.
    Pgc(Pgc),
    /// A segment.
    Sg(Sg),
    /// A multiline.
    Ml(Ml),
    /// An arc.
    CurveArc(CurveArc),
    /// A character to be printed in SVG, at a point.
    Txt(Txt),
    /// A group of other objects.
    Group(Group<Style>),
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
                    Obj::Pt(x) => {
                        x.$fn(rhs);
                    }
                    Obj::Ml(x) => {
                        x.$fn(rhs);
                    }
                    Obj::Txt(x) => {
                        x.$fn(rhs);
                    }
                    Obj::CurveArc(x) => {
                        x.$fn(rhs);
                    }
                    Obj::Group(x) => {
                        x.$fn(rhs);
                    }
                    Obj::Pg(x) => {
                        x.$fn(rhs);
                    }
                    Obj::Pgc(x) => {
                        x.$fn(rhs);
                    }
                    Obj::Sg(x) => {
                        x.$fn(rhs);
                    }
                }
            }
        }
    };
}

impl_ops_assign!(AddAssign, add_assign, Pt);
impl_ops_assign!(DivAssign, div_assign, f64);
impl_ops_assign!(MulAssign, mul_assign, f64);
impl_ops_assign!(RemAssign, rem_assign, Pt);
impl_ops_assign!(SubAssign, sub_assign, Pt);

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
                    Obj::Ml(x) => Obj::from(x.$fn(rhs)),
                    Obj::Pt(x) => Obj::from(x.$fn(rhs)),
                    Obj::Txt(x) => Obj::from(x.$fn(rhs)),
                    Obj::CurveArc(x) => Obj::from(x.$fn(rhs)),
                    Obj::Group(x) => Obj::from(x.$fn(rhs)),
                    Obj::Pg(x) => Obj::from(x.$fn(rhs)),
                    Obj::Pgc(x) => Obj::from(x.$fn(rhs)),
                    Obj::Sg(x) => Obj::from(x.$fn(rhs)),
                }
            }
        }
    };
}

impl_ops!(Add, add, Pt);
impl_ops!(Sub, sub, Pt);
impl_ops!(Mul, mul, f64);
impl_ops!(Div, div, f64);

impl Croppable for Obj {
    type Output = Obj;
    fn crop(&self, frame: &Pg, crop_type: CropType) -> Result<Vec<Self::Output>> {
        match &self {
            Obj::Pt(pt) => {
                assert_eq!(crop_type, CropType::Inclusive);
                if !matches!(frame.contains_pt(pt), Ok(PointLoc::Outside)) {
                    Ok(vec![self.clone()])
                } else {
                    Ok(vec![])
                }
            }
            Obj::Ml(ml) => Ok(ml
                .to_segments()
                .into_iter()
                .map(|sg| sg.crop(frame, crop_type))
                .flatten_ok()
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .map(Obj::from)
                .collect::<Vec<_>>()),
            Obj::Pg(pg) => Ok(pg
                .crop(frame, crop_type)?
                .into_iter()
                .map(Obj::from)
                .collect::<Vec<_>>()),
            Obj::Pgc(_) => unimplemented!("TODO: implement cropping for Pgc."),
            Obj::Sg(sg) => Ok(sg
                .crop(frame, crop_type)?
                .into_iter()
                .map(Obj::from)
                .collect::<Vec<_>>()),
            Obj::CurveArc(ca) => Ok(ca
                .crop(frame, crop_type)?
                .into_iter()
                .map(Obj::from)
                .collect::<Vec<_>>()),
            Obj::Txt(ch) => {
                assert_eq!(crop_type, CropType::Inclusive);
                if !matches!(frame.contains_pt(&ch.pt), Ok(PointLoc::Outside)) {
                    Ok(vec![self.clone()])
                } else {
                    Ok(vec![])
                }
            }
            Obj::Group(g) => Ok(g
                .crop(frame, crop_type)?
                .into_iter()
                .map(Obj::from)
                .collect::<Vec<_>>()),
        }
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

impl Object for Obj {
    fn objtype(&self) -> ObjType {
        match self {
            Obj::CurveArc(_) => ObjType::CurveArc,
            Obj::Group(_) => ObjType::Group,
            Obj::Ml(_) => ObjType::Multiline,
            Obj::Pg(_) => ObjType::Group,
            Obj::Pgc(_) => ObjType::PolygonWithCavities,
            Obj::Pt(_) => ObjType::Point,
            Obj::Sg(_) => ObjType::Segment,
            Obj::Txt(_) => ObjType::Text,
        }
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &Pt> + '_> {
        match self {
            Obj::Pt(p) => Box::new(p.iter()),
            Obj::Ml(ml) => Box::new(ml.iter()),
            Obj::Txt(ch) => Box::new(ch.iter()),
            Obj::CurveArc(ca) => Box::new(ca.iter()),
            Obj::Group(g) => Box::new(g.iter()),
            Obj::Pg(pg) => Box::new(pg.iter()),
            Obj::Pgc(pgc) => Box::new(pgc.iter()),
            Obj::Sg(sg) => Box::new(sg.iter()),
        }
    }

    fn iter_mut(&mut self) -> Box<dyn Iterator<Item = &mut Pt> + '_> {
        match self {
            Obj::Pt(p) => Box::new(p.iter_mut()),
            Obj::Ml(ml) => Box::new(ml.iter_mut()),
            Obj::Txt(ch) => Box::new(ch.iter_mut()),
            Obj::CurveArc(ca) => Box::new(ca.iter_mut()),
            Obj::Group(g) => Box::new(g.iter_mut()),
            Obj::Pg(pg) => Box::new(pg.iter_mut()),
            Obj::Pgc(pgc) => Box::new(pgc.iter_mut()),
            Obj::Sg(sg) => Box::new(sg.iter_mut()),
        }
    }
}
