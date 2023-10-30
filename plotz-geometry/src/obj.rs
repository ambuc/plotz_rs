//! The inner value of a Object2d, i.e. the enum which holds some geometric thingy.

use crate::{
    bounded::{Bounded, Bounds},
    crop::{CropType, Croppable, PointLoc},
    group::Group,
    shapes::{curve::CurveArc, ml::Ml, pg::Pg, pt::Pt, sg::Sg, txt::Txt},
    style::Style,
    *,
};
use anyhow::Result;
use enum_dispatch::enum_dispatch;
use itertools::Itertools;
use std::{fmt::Debug, ops::*};

/// Either a polygon or a segment.
#[derive(Debug, PartialEq, Clone)]
#[enum_dispatch]
pub enum Obj {
    /// A point.
    Pt(Pt),
    /// A polygon.
    Pg(Pg),
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

impl Obj {
    /// Iterator.
    pub fn iter(&self) -> Box<dyn Iterator<Item = &Pt> + '_> {
        match self {
            Obj::Pt(p) => Box::new(p.iter()),
            Obj::Ml(ml) => Box::new(ml.iter()),
            Obj::Txt(ch) => Box::new(ch.iter()),
            Obj::CurveArc(ca) => Box::new(ca.iter()),
            Obj::Group(g) => Box::new(g.iter()),
            Obj::Pg(pg) => Box::new(pg.iter()),
            Obj::Sg(sg) => Box::new(sg.iter()),
        }
    }

    /// Mutable iterator.
    pub fn iter_mut(&mut self) -> Box<dyn Iterator<Item = &mut Pt> + '_> {
        match self {
            Obj::Pt(p) => Box::new(p.iter_mut()),
            Obj::Ml(ml) => Box::new(ml.iter_mut()),
            Obj::Txt(ch) => Box::new(ch.iter_mut()),
            Obj::CurveArc(ca) => Box::new(ca.iter_mut()),
            Obj::Group(g) => Box::new(g.iter_mut()),
            Obj::Pg(pg) => Box::new(pg.iter_mut()),
            Obj::Sg(sg) => Box::new(sg.iter_mut()),
        }
    }
}

impl<T> RemAssign<T> for Obj
where
    T: Into<Pt>,
{
    fn rem_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        match self {
            Obj::Pt(p) => {
                *p %= rhs;
            }
            Obj::Ml(ml) => {
                *ml %= rhs;
            }
            Obj::Txt(ch) => {
                *ch %= rhs;
            }
            Obj::CurveArc(ca) => {
                *ca %= rhs;
            }
            Obj::Group(g) => {
                *g %= rhs;
            }
            Obj::Pg(pg) => {
                *pg %= rhs;
            }
            Obj::Sg(sg) => {
                *sg %= rhs;
            }
        }
    }
}

impl<T> Add<T> for Obj
where
    T: Into<Pt>,
{
    type Output = Obj;
    fn add(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        match self {
            Obj::Pt(p) => Obj::from(p + rhs),
            Obj::Ml(ml) => Obj::from(ml + rhs),
            Obj::Txt(ch) => Obj::from(ch + rhs),
            Obj::CurveArc(ca) => Obj::from(ca + rhs),
            Obj::Group(g) => Obj::from(g + rhs),
            Obj::Pg(pg) => Obj::from(pg + rhs),
            Obj::Sg(sg) => Obj::from(sg + rhs),
        }
    }
}

impl<T> Sub<T> for Obj
where
    T: Into<Pt>,
{
    type Output = Obj;
    fn sub(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        match self {
            Obj::Pt(p) => Obj::from(p - rhs),
            Obj::Ml(ml) => Obj::from(ml - rhs),
            Obj::Txt(ch) => Obj::from(ch - rhs),
            Obj::CurveArc(ca) => Obj::from(ca - rhs),
            Obj::Group(g) => Obj::from(g - rhs),
            Obj::Pg(pg) => Obj::from(pg - rhs),
            Obj::Sg(sg) => Obj::from(sg - rhs),
        }
    }
}
impl Mul<f64> for Obj {
    type Output = Obj;
    fn mul(self, rhs: f64) -> Self::Output {
        match self {
            Obj::Ml(ml) => Obj::from(ml * rhs),
            Obj::Pt(p) => Obj::from(p * rhs),
            Obj::Txt(ch) => Obj::from(ch * rhs),
            Obj::CurveArc(ca) => Obj::from(ca * rhs),
            Obj::Group(g) => Obj::from(g * rhs),
            Obj::Pg(pg) => Obj::from(pg * rhs),
            Obj::Sg(sg) => Obj::from(sg * rhs),
        }
    }
}
impl Div<f64> for Obj {
    type Output = Obj;
    fn div(self, rhs: f64) -> Self::Output {
        match self {
            Obj::Ml(ml) => Obj::from(ml / rhs),
            Obj::Pt(p) => Obj::from(p / rhs),
            Obj::Txt(ch) => Obj::from(ch / rhs),
            Obj::CurveArc(ca) => Obj::from(ca / rhs),
            Obj::Group(g) => Obj::from(g / rhs),
            Obj::Pg(pg) => Obj::from(pg / rhs),
            Obj::Sg(sg) => Obj::from(sg / rhs),
        }
    }
}
impl<T> AddAssign<T> for Obj
where
    T: Into<Pt>,
{
    fn add_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        match self {
            Obj::Ml(ml) => {
                *ml += rhs;
            }
            Obj::Pt(p) => {
                *p += rhs;
            }
            Obj::Txt(ch) => {
                *ch += rhs;
            }
            Obj::CurveArc(ca) => {
                *ca += rhs;
            }
            Obj::Group(g) => {
                *g += rhs;
            }
            Obj::Pg(pg) => {
                *pg += rhs;
            }
            Obj::Sg(sg) => {
                *sg += rhs;
            }
        }
    }
}
impl<T> SubAssign<T> for Obj
where
    T: Into<Pt>,
{
    fn sub_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        match self {
            Obj::Pt(p) => {
                *p -= rhs;
            }
            Obj::Txt(ch) => {
                *ch -= rhs;
            }
            Obj::CurveArc(ca) => {
                *ca -= rhs;
            }
            Obj::Group(g) => {
                *g -= rhs;
            }
            Obj::Pg(pg) => {
                *pg -= rhs;
            }
            Obj::Ml(ml) => {
                *ml -= rhs;
            }
            Obj::Sg(sg) => {
                *sg -= rhs;
            }
        }
    }
}

impl MulAssign<f64> for Obj {
    fn mul_assign(&mut self, rhs: f64) {
        match self {
            Obj::Pt(p) => {
                *p *= rhs;
            }
            Obj::Txt(ch) => {
                *ch *= rhs;
            }
            Obj::CurveArc(ca) => {
                *ca *= rhs;
            }
            Obj::Group(g) => {
                *g *= rhs;
            }
            Obj::Pg(pg) => {
                *pg *= rhs;
            }
            Obj::Sg(sg) => {
                *sg *= rhs;
            }
            Obj::Ml(ml) => {
                *ml *= rhs;
            }
        }
    }
}

impl DivAssign<f64> for Obj {
    fn div_assign(&mut self, rhs: f64) {
        match self {
            Obj::Ml(ml) => {
                *ml /= rhs;
            }
            Obj::Pt(p) => {
                *p /= rhs;
            }
            Obj::Txt(ch) => {
                *ch /= rhs;
            }
            Obj::CurveArc(ca) => {
                *ca /= rhs;
            }
            Obj::Group(g) => {
                *g /= rhs;
            }
            Obj::Pg(pg) => {
                *pg /= rhs;
            }
            Obj::Sg(sg) => {
                *sg /= rhs;
            }
        }
    }
}

impl Scalable<f64> for Obj {}

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

impl Annotatable for Obj {
    fn annotate(&self, settings: &AnnotationSettings) -> Vec<(Obj, Style)> {
        match self {
            Obj::Pg(pg) => pg.annotate(settings),
            Obj::Ml(ml) => ml.annotate(settings),
            Obj::Group(g) => g.annotate(settings),
            Obj::Pt(_) | Obj::Sg(_) | Obj::CurveArc(_) | Obj::Txt(_) => vec![],
        }
    }
}
