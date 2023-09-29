//! The inner value of a Object2d, i.e. the enum which holds some geometric thingy.

use crate::styled_obj2::StyledObj2;

use {
    crate::{
        bounded::Bounded,
        crop::CropType,
        crop::{Croppable, PointLoc},
        group::Group,
        shapes::{curve::CurveArc, pg2::Pg2, pg2::PolygonKind, pt2::Pt2, sg2::Sg2, txt::Txt},
        traits::*,
    },
    derive_more::From,
    std::{fmt::Debug, ops::*},
};

/// Either a polygon or a segment.
#[derive(Debug, PartialEq, Clone, From)]
pub enum Obj2 {
    /// A point.
    Pt(Pt2),
    /// A polygon.
    Pg2(Pg2),
    /// A segment.
    Sg2(Sg2),
    /// An arc.
    CurveArc(CurveArc),
    /// A character to be printed in SVG, at a point.
    Txt(Txt),
    /// A group of other objects.
    Group(Group<StyledObj2>),
}

impl Obj2 {
    /// Returns true if the object is empty (i.e. zero points)
    pub fn is_empty(&self) -> bool {
        match self {
            Obj2::Pg2(p) => p.is_empty(),
            Obj2::Group(dois) => dois.is_empty(),
            Obj2::Pt(pt) => pt.is_empty(),
            Obj2::Sg2(sg) => sg.is_empty(),
            Obj2::Txt(ch) => ch.is_empty(),
            Obj2::CurveArc(ca) => ca.is_empty(),
        }
    }

    /// Casts each inner value to something which implements Bounded.
    pub fn bounded(&self) -> &dyn Bounded {
        match self {
            Obj2::Txt(ch) => ch,
            Obj2::CurveArc(arc) => arc,
            Obj2::Group(dos) => dos,
            Obj2::Pt(p) => p,
            Obj2::Pg2(pg) => pg,
            Obj2::Sg2(s) => s,
        }
    }

    /// Casts each inner value to something which implements YieldPoints.
    pub fn yield_points(&self) -> &dyn YieldPoints {
        match self {
            Obj2::Pt(p) => p,
            Obj2::Txt(ch) => ch,
            Obj2::CurveArc(ca) => ca,
            Obj2::Group(g) => g,
            Obj2::Pg2(pg) => pg,
            Obj2::Sg2(sg) => sg,
        }
    }

    /// Casts each inner value to something which implements YieldPointsMut.
    pub fn yield_points_mut(&mut self) -> &mut dyn YieldPointsMut {
        match self {
            Obj2::Pt(p) => p,
            Obj2::Txt(ch) => ch,
            Obj2::CurveArc(ca) => ca,
            Obj2::Group(g) => g,
            Obj2::Pg2(pg) => pg,
            Obj2::Sg2(sg) => sg,
        }
    }
    /// Cast to sg2, if possible
    pub fn to_sg2(&self) -> Option<&Sg2> {
        match self {
            Obj2::Sg2(x) => Some(x),
            _ => None,
        }
    }
    /// Cast to pg2, if possible
    pub fn to_pg2(&self) -> Option<&Pg2> {
        match self {
            Obj2::Pg2(x) => Some(x),
            _ => None,
        }
    }
}

impl YieldPoints for Obj2 {
    fn yield_pts(&self) -> Box<dyn Iterator<Item = &Pt2> + '_> {
        self.yield_points().yield_pts()
    }
}

impl YieldPointsMut for Obj2 {
    fn yield_pts_mut(&mut self) -> Box<dyn Iterator<Item = &mut Pt2> + '_> {
        self.yield_points_mut().yield_pts_mut()
    }
}

impl Mutable for Obj2 {}

impl Bounded for Obj2 {
    fn bounds(&self) -> crate::bounded::Bounds {
        self.bounded().bounds()
    }
}

impl RemAssign<Pt2> for Obj2 {
    fn rem_assign(&mut self, rhs: Pt2) {
        match self {
            Obj2::Pt(p) => {
                *p %= rhs;
            }
            Obj2::Txt(ch) => {
                *ch %= rhs;
            }
            Obj2::CurveArc(ca) => {
                *ca %= rhs;
            }
            Obj2::Group(g) => {
                *g %= rhs;
            }
            Obj2::Pg2(pg) => {
                *pg %= rhs;
            }
            Obj2::Sg2(sg) => {
                *sg %= rhs;
            }
        }
    }
}

impl Add<Pt2> for Obj2 {
    type Output = Obj2;
    fn add(self, rhs: Pt2) -> Self::Output {
        match self {
            Obj2::Pt(p) => Obj2::from(p + rhs),
            Obj2::Txt(ch) => Obj2::from(ch + rhs),
            Obj2::CurveArc(ca) => Obj2::from(ca + rhs),
            Obj2::Group(g) => Obj2::from(g + rhs),
            Obj2::Pg2(pg) => Obj2::from(pg + rhs),
            Obj2::Sg2(sg) => Obj2::from(sg + rhs),
        }
    }
}

impl Sub<Pt2> for Obj2 {
    type Output = Obj2;
    fn sub(self, rhs: Pt2) -> Self::Output {
        match self {
            Obj2::Pt(p) => Obj2::from(p - rhs),
            Obj2::Txt(ch) => Obj2::from(ch - rhs),
            Obj2::CurveArc(ca) => Obj2::from(ca - rhs),
            Obj2::Group(g) => Obj2::from(g - rhs),
            Obj2::Pg2(pg) => Obj2::from(pg - rhs),
            Obj2::Sg2(sg) => Obj2::from(sg - rhs),
        }
    }
}
impl Mul<f64> for Obj2 {
    type Output = Obj2;
    fn mul(self, rhs: f64) -> Self::Output {
        match self {
            Obj2::Pt(p) => Obj2::from(p * rhs),
            Obj2::Txt(ch) => Obj2::from(ch * rhs),
            Obj2::CurveArc(ca) => Obj2::from(ca * rhs),
            Obj2::Group(g) => Obj2::from(g * rhs),
            Obj2::Pg2(pg) => Obj2::from(pg * rhs),
            Obj2::Sg2(sg) => Obj2::from(sg * rhs),
        }
    }
}
impl Div<f64> for Obj2 {
    type Output = Obj2;
    fn div(self, rhs: f64) -> Self::Output {
        match self {
            Obj2::Pt(p) => Obj2::from(p / rhs),
            Obj2::Txt(ch) => Obj2::from(ch / rhs),
            Obj2::CurveArc(ca) => Obj2::from(ca / rhs),
            Obj2::Group(g) => Obj2::from(g / rhs),
            Obj2::Pg2(pg) => Obj2::from(pg / rhs),
            Obj2::Sg2(sg) => Obj2::from(sg / rhs),
        }
    }
}
impl AddAssign<Pt2> for Obj2 {
    fn add_assign(&mut self, rhs: Pt2) {
        match self {
            Obj2::Pt(p) => {
                *p += rhs;
            }
            Obj2::Txt(ch) => {
                *ch += rhs;
            }
            Obj2::CurveArc(ca) => {
                *ca += rhs;
            }
            Obj2::Group(g) => {
                *g += rhs;
            }
            Obj2::Pg2(pg) => {
                *pg += rhs;
            }
            Obj2::Sg2(sg) => {
                *sg += rhs;
            }
        }
    }
}
impl SubAssign<Pt2> for Obj2 {
    fn sub_assign(&mut self, rhs: Pt2) {
        match self {
            Obj2::Pt(p) => {
                *p -= rhs;
            }
            Obj2::Txt(ch) => {
                *ch -= rhs;
            }
            Obj2::CurveArc(ca) => {
                *ca -= rhs;
            }
            Obj2::Group(g) => {
                *g -= rhs;
            }
            Obj2::Pg2(pg) => {
                *pg -= rhs;
            }
            Obj2::Sg2(sg) => {
                *sg -= rhs;
            }
        }
    }
}

impl MulAssign<f64> for Obj2 {
    fn mul_assign(&mut self, rhs: f64) {
        match self {
            Obj2::Pt(p) => {
                *p *= rhs;
            }
            Obj2::Txt(ch) => {
                *ch *= rhs;
            }
            Obj2::CurveArc(ca) => {
                *ca *= rhs;
            }
            Obj2::Group(g) => {
                *g *= rhs;
            }
            Obj2::Pg2(pg) => {
                *pg *= rhs;
            }
            Obj2::Sg2(sg) => {
                *sg *= rhs;
            }
        }
    }
}

impl DivAssign<f64> for Obj2 {
    fn div_assign(&mut self, rhs: f64) {
        match self {
            Obj2::Pt(p) => {
                *p /= rhs;
            }
            Obj2::Txt(ch) => {
                *ch /= rhs;
            }
            Obj2::CurveArc(ca) => {
                *ca /= rhs;
            }
            Obj2::Group(g) => {
                *g /= rhs;
            }
            Obj2::Pg2(pg) => {
                *pg /= rhs;
            }
            Obj2::Sg2(sg) => {
                *sg /= rhs;
            }
        }
    }
}

impl Translatable for Obj2 {}
impl Scalable<f64> for Obj2 {}
impl ScalableAssign for Obj2 {}
impl TranslatableAssign for Obj2 {}

impl Croppable for Obj2 {
    type Output = Obj2;
    fn crop(&self, frame: &Pg2, crop_type: CropType) -> Vec<Self::Output> {
        match &self {
            Obj2::Pt(pt) => {
                assert_eq!(crop_type, CropType::Inclusive);
                if !matches!(frame.contains_pt(pt), PointLoc::Outside) {
                    vec![self.clone()]
                } else {
                    vec![]
                }
            }
            Obj2::Pg2(pg) => match pg.kind {
                PolygonKind::Open => pg
                    .to_segments()
                    .into_iter()
                    .flat_map(|sg| sg.crop(frame, crop_type))
                    .map(Obj2::from)
                    .collect::<Vec<_>>(),
                PolygonKind::Closed => pg
                    .crop(frame, crop_type)
                    .into_iter()
                    .map(Obj2::from)
                    .collect::<Vec<_>>(),
            },
            Obj2::Sg2(sg) => sg
                .crop(frame, crop_type)
                .into_iter()
                .map(Obj2::from)
                .collect::<Vec<_>>(),
            Obj2::CurveArc(ca) => ca
                .crop(frame, crop_type)
                .into_iter()
                .map(Obj2::from)
                .collect::<Vec<_>>(),
            Obj2::Txt(ch) => {
                assert_eq!(crop_type, CropType::Inclusive);
                if !matches!(frame.contains_pt(&ch.pt), PointLoc::Outside) {
                    vec![self.clone()]
                } else {
                    vec![]
                }
            }
            Obj2::Group(g) => g
                .crop(frame, crop_type)
                .into_iter()
                .map(Obj2::from)
                .collect::<Vec<_>>(),
        }
    }

    fn crop_excluding(&self, other: &Pg2) -> Vec<Self::Output>
    where
        Self: Sized,
    {
        match &self {
            Obj2::Pt(pt) => {
                if matches!(other.contains_pt(pt), PointLoc::Outside) {
                    vec![]
                } else {
                    vec![self.clone()]
                }
            }
            Obj2::Pg2(pg) => match pg.kind {
                PolygonKind::Open => pg
                    .to_segments()
                    .into_iter()
                    .flat_map(|sg| sg.crop_excluding(other))
                    .map(Obj2::from)
                    .collect::<Vec<_>>(),
                PolygonKind::Closed => pg
                    .crop_excluding(other)
                    .into_iter()
                    .map(Obj2::from)
                    .collect::<Vec<_>>(),
            },
            Obj2::Sg2(sg) => sg
                .crop_excluding(other)
                .into_iter()
                .map(Obj2::from)
                .collect::<Vec<_>>(),
            Obj2::CurveArc(ca) => ca
                .crop_excluding(other)
                .into_iter()
                .map(Obj2::from)
                .collect::<Vec<_>>(),
            Obj2::Txt(ch) => {
                if matches!(other.contains_pt(&ch.pt), PointLoc::Outside) {
                    vec![]
                } else {
                    vec![self.clone()]
                }
            }
            Obj2::Group(g) => g
                .crop_excluding(other)
                .into_iter()
                .map(Obj2::from)
                .collect::<Vec<_>>(),
        }
    }
}

impl Annotatable for Obj2 {
    fn annotate(&self, settings: &AnnotationSettings) -> Vec<crate::styled_obj2::StyledObj2> {
        match self {
            Obj2::Pg2(pg) => pg.annotate(settings),
            Obj2::Group(g) => g.annotate(settings),
            Obj2::Pt(_) | Obj2::Sg2(_) | Obj2::CurveArc(_) | Obj2::Txt(_) => vec![],
        }
    }
}
