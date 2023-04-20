//! The inner value of a Object2d, i.e. the enum which holds some geometric thingy.

use std::fmt::Debug;

use crate::{crop::CropType, polygon::PolygonKind};

use {
    crate::{
        bounded::Bounded,
        crop::{Croppable, PointLoc},
        curve::CurveArc,
        group::Group,
        point::Pt,
        polygon::Polygon,
        segment::Segment,
        traits::*,
        txt::Txt,
    },
    derive_more::From,
    std::ops::*,
};

/// Either a polygon or a segment.
#[derive(Debug, PartialEq, Clone, From)]
pub enum Object2dInner {
    /// A point.
    Point(Pt),
    /// A polygon.
    Polygon(Polygon),
    /// A segment.
    Segment(Segment),
    /// An arc.
    CurveArc(CurveArc),
    /// A character to be printed in SVG, at a point.
    Char(Txt),
    /// A group of other objects.
    Group(Group),
}

impl Object2dInner {
    /// Returns true if the object is empty (i.e. zero points)
    pub fn is_empty(&self) -> bool {
        match self {
            Object2dInner::Polygon(p) => p.is_empty(),
            Object2dInner::Group(dois) => dois.is_empty(),
            Object2dInner::Point(pt) => pt.is_empty(),
            Object2dInner::Segment(sg) => sg.is_empty(),
            Object2dInner::Char(ch) => ch.is_empty(),
            Object2dInner::CurveArc(ca) => ca.is_empty(),
        }
    }

    /// Casts each inner value to something which implements Bounded.
    pub fn inner_impl_bounded(&self) -> &dyn Bounded {
        match self {
            Object2dInner::Char(ch) => ch,
            Object2dInner::CurveArc(arc) => arc,
            Object2dInner::Group(dos) => dos,
            Object2dInner::Point(p) => p,
            Object2dInner::Polygon(pg) => pg,
            Object2dInner::Segment(s) => s,
        }
    }

    /// Casts each inner value to something which implements YieldPoints.
    pub fn inner_impl_yield_points(&self) -> &dyn YieldPoints {
        match self {
            Object2dInner::Point(p) => p,
            Object2dInner::Char(ch) => ch,
            Object2dInner::CurveArc(ca) => ca,
            Object2dInner::Group(g) => g,
            Object2dInner::Polygon(pg) => pg,
            Object2dInner::Segment(sg) => sg,
        }
    }

    /// Casts each inner value to something which implements YieldPointsMut.
    pub fn inner_impl_yield_points_mut(&mut self) -> &mut dyn YieldPointsMut {
        match self {
            Object2dInner::Point(p) => p,
            Object2dInner::Char(ch) => ch,
            Object2dInner::CurveArc(ca) => ca,
            Object2dInner::Group(g) => g,
            Object2dInner::Polygon(pg) => pg,
            Object2dInner::Segment(sg) => sg,
        }
    }
}

impl YieldPoints for Object2dInner {
    fn yield_pts(&self) -> Option<Box<dyn Iterator<Item = &Pt> + '_>> {
        self.inner_impl_yield_points().yield_pts()
    }
}

impl YieldPointsMut for Object2dInner {
    fn yield_pts_mut(&mut self) -> Option<Box<dyn Iterator<Item = &mut Pt> + '_>> {
        self.inner_impl_yield_points_mut().yield_pts_mut()
    }
}

impl Mutable for Object2dInner {}

impl Bounded for Object2dInner {
    fn bounds(&self) -> crate::bounded::Bounds {
        self.inner_impl_bounded().bounds()
    }
}

impl RemAssign<Pt> for Object2dInner {
    fn rem_assign(&mut self, rhs: Pt) {
        match self {
            Object2dInner::Point(p) => {
                *p %= rhs;
            }
            Object2dInner::Char(ch) => {
                *ch %= rhs;
            }
            Object2dInner::CurveArc(ca) => {
                *ca %= rhs;
            }
            Object2dInner::Group(g) => {
                *g %= rhs;
            }
            Object2dInner::Polygon(pg) => {
                *pg %= rhs;
            }
            Object2dInner::Segment(sg) => {
                *sg %= rhs;
            }
        }
    }
}

impl Add<Pt> for Object2dInner {
    type Output = Object2dInner;
    fn add(self, rhs: Pt) -> Self::Output {
        match self {
            Object2dInner::Point(p) => Object2dInner::from(p + rhs),
            Object2dInner::Char(ch) => Object2dInner::from(ch + rhs),
            Object2dInner::CurveArc(ca) => Object2dInner::from(ca + rhs),
            Object2dInner::Group(g) => Object2dInner::from(g + rhs),
            Object2dInner::Polygon(pg) => Object2dInner::from(pg + rhs),
            Object2dInner::Segment(sg) => Object2dInner::from(sg + rhs),
        }
    }
}

impl Sub<Pt> for Object2dInner {
    type Output = Object2dInner;
    fn sub(self, rhs: Pt) -> Self::Output {
        match self {
            Object2dInner::Point(p) => Object2dInner::from(p - rhs),
            Object2dInner::Char(ch) => Object2dInner::from(ch - rhs),
            Object2dInner::CurveArc(ca) => Object2dInner::from(ca - rhs),
            Object2dInner::Group(g) => Object2dInner::from(g - rhs),
            Object2dInner::Polygon(pg) => Object2dInner::from(pg - rhs),
            Object2dInner::Segment(sg) => Object2dInner::from(sg - rhs),
        }
    }
}
impl Mul<f64> for Object2dInner {
    type Output = Object2dInner;
    fn mul(self, rhs: f64) -> Self::Output {
        match self {
            Object2dInner::Point(p) => Object2dInner::from(p * rhs),
            Object2dInner::Char(ch) => Object2dInner::from(ch * rhs),
            Object2dInner::CurveArc(ca) => Object2dInner::from(ca * rhs),
            Object2dInner::Group(g) => Object2dInner::from(g * rhs),
            Object2dInner::Polygon(pg) => Object2dInner::from(pg * rhs),
            Object2dInner::Segment(sg) => Object2dInner::from(sg * rhs),
        }
    }
}
impl Div<f64> for Object2dInner {
    type Output = Object2dInner;
    fn div(self, rhs: f64) -> Self::Output {
        match self {
            Object2dInner::Point(p) => Object2dInner::from(p / rhs),
            Object2dInner::Char(ch) => Object2dInner::from(ch / rhs),
            Object2dInner::CurveArc(ca) => Object2dInner::from(ca / rhs),
            Object2dInner::Group(g) => Object2dInner::from(g / rhs),
            Object2dInner::Polygon(pg) => Object2dInner::from(pg / rhs),
            Object2dInner::Segment(sg) => Object2dInner::from(sg / rhs),
        }
    }
}
impl AddAssign<Pt> for Object2dInner {
    fn add_assign(&mut self, rhs: Pt) {
        match self {
            Object2dInner::Point(p) => {
                *p += rhs;
            }
            Object2dInner::Char(ch) => {
                *ch += rhs;
            }
            Object2dInner::CurveArc(ca) => {
                *ca += rhs;
            }
            Object2dInner::Group(g) => {
                *g += rhs;
            }
            Object2dInner::Polygon(pg) => {
                *pg += rhs;
            }
            Object2dInner::Segment(sg) => {
                *sg += rhs;
            }
        }
    }
}
impl SubAssign<Pt> for Object2dInner {
    fn sub_assign(&mut self, rhs: Pt) {
        match self {
            Object2dInner::Point(p) => {
                *p -= rhs;
            }
            Object2dInner::Char(ch) => {
                *ch -= rhs;
            }
            Object2dInner::CurveArc(ca) => {
                *ca -= rhs;
            }
            Object2dInner::Group(g) => {
                *g -= rhs;
            }
            Object2dInner::Polygon(pg) => {
                *pg -= rhs;
            }
            Object2dInner::Segment(sg) => {
                *sg -= rhs;
            }
        }
    }
}

impl MulAssign<f64> for Object2dInner {
    fn mul_assign(&mut self, rhs: f64) {
        match self {
            Object2dInner::Point(p) => {
                *p *= rhs;
            }
            Object2dInner::Char(ch) => {
                *ch *= rhs;
            }
            Object2dInner::CurveArc(ca) => {
                *ca *= rhs;
            }
            Object2dInner::Group(g) => {
                *g *= rhs;
            }
            Object2dInner::Polygon(pg) => {
                *pg *= rhs;
            }
            Object2dInner::Segment(sg) => {
                *sg *= rhs;
            }
        }
    }
}

impl DivAssign<f64> for Object2dInner {
    fn div_assign(&mut self, rhs: f64) {
        match self {
            Object2dInner::Point(p) => {
                *p /= rhs;
            }
            Object2dInner::Char(ch) => {
                *ch /= rhs;
            }
            Object2dInner::CurveArc(ca) => {
                *ca /= rhs;
            }
            Object2dInner::Group(g) => {
                *g /= rhs;
            }
            Object2dInner::Polygon(pg) => {
                *pg /= rhs;
            }
            Object2dInner::Segment(sg) => {
                *sg /= rhs;
            }
        }
    }
}

impl Translatable for Object2dInner {}
impl Scalable<f64> for Object2dInner {}
impl ScalableAssign for Object2dInner {}
impl TranslatableAssign for Object2dInner {}

impl Croppable for Object2dInner {
    type Output = Object2dInner;
    fn crop(&self, frame: &Polygon, crop_type: CropType) -> Vec<Self::Output> {
        match &self {
            Object2dInner::Point(pt) => {
                assert_eq!(crop_type, CropType::Inclusive);
                if !matches!(frame.contains_pt(pt), PointLoc::Outside) {
                    vec![self.clone()]
                } else {
                    vec![]
                }
            }
            Object2dInner::Polygon(pg) => match pg.kind {
                PolygonKind::Open => pg
                    .to_segments()
                    .into_iter()
                    .flat_map(|sg| sg.crop(frame, crop_type))
                    .into_iter()
                    .map(Object2dInner::from)
                    .collect::<Vec<_>>(),
                PolygonKind::Closed => pg
                    .crop(frame, crop_type)
                    .into_iter()
                    .map(Object2dInner::from)
                    .collect::<Vec<_>>(),
            },
            Object2dInner::Segment(sg) => sg
                .crop(frame, crop_type)
                .into_iter()
                .map(Object2dInner::from)
                .collect::<Vec<_>>(),
            Object2dInner::CurveArc(ca) => ca
                .crop(frame, crop_type)
                .into_iter()
                .map(Object2dInner::from)
                .collect::<Vec<_>>(),
            Object2dInner::Char(ch) => {
                assert_eq!(crop_type, CropType::Inclusive);
                if !matches!(frame.contains_pt(&ch.pt), PointLoc::Outside) {
                    vec![self.clone()]
                } else {
                    vec![]
                }
            }
            Object2dInner::Group(g) => g
                .crop(frame, crop_type)
                .into_iter()
                .map(Object2dInner::from)
                .collect::<Vec<_>>(),
        }
    }

    fn crop_excluding(&self, other: &Polygon) -> Vec<Self::Output>
    where
        Self: Sized,
    {
        match &self {
            Object2dInner::Point(pt) => {
                if matches!(other.contains_pt(pt), PointLoc::Outside) {
                    vec![]
                } else {
                    vec![self.clone()]
                }
            }
            Object2dInner::Polygon(pg) => match pg.kind {
                PolygonKind::Open => pg
                    .to_segments()
                    .into_iter()
                    .flat_map(|sg| sg.crop_excluding(other))
                    .into_iter()
                    .map(Object2dInner::from)
                    .collect::<Vec<_>>(),
                PolygonKind::Closed => pg
                    .crop_excluding(other)
                    .into_iter()
                    .map(Object2dInner::from)
                    .collect::<Vec<_>>(),
            },
            Object2dInner::Segment(sg) => sg
                .crop_excluding(other)
                .into_iter()
                .map(Object2dInner::from)
                .collect::<Vec<_>>(),
            Object2dInner::CurveArc(ca) => ca
                .crop_excluding(other)
                .into_iter()
                .map(Object2dInner::from)
                .collect::<Vec<_>>(),
            Object2dInner::Char(ch) => {
                if matches!(other.contains_pt(&ch.pt), PointLoc::Outside) {
                    vec![]
                } else {
                    vec![self.clone()]
                }
            }
            Object2dInner::Group(g) => g
                .crop_excluding(other)
                .into_iter()
                .map(Object2dInner::from)
                .collect::<Vec<_>>(),
        }
    }
}

impl Annotatable for Object2dInner {
    fn annotate(&self, settings: &AnnotationSettings) -> Vec<crate::object2d::Object2d> {
        match self {
            Object2dInner::Polygon(pg) => pg.annotate(settings),
            Object2dInner::Group(g) => g.annotate(settings),
            Object2dInner::Point(_)
            | Object2dInner::Segment(_)
            | Object2dInner::CurveArc(_)
            | Object2dInner::Char(_) => vec![],
        }
    }
}
