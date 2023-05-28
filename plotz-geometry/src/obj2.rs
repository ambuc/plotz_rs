//! The inner value of a Object2d, i.e. the enum which holds some geometric thingy.

use {
    crate::{
        bounded::Bounded,
        crop::CropType,
        crop::{Croppable, PointLoc},
        group::Group,
        shapes::{
            curve::CurveArc, point::Pt, polygon::Polygon, polygon::PolygonKind, segment::Sg2,
            txt::Txt,
        },
        traits::*,
    },
    derive_more::From,
    std::{fmt::Debug, ops::*},
};

/// Either a polygon or a segment.
#[derive(Debug, PartialEq, Clone, From)]
pub enum Obj2 {
    /// A point.
    Point(Pt),
    /// A polygon.
    Polygon(Polygon),
    /// A segment.
    Segment(Sg2),
    /// An arc.
    CurveArc(CurveArc),
    /// A character to be printed in SVG, at a point.
    Char(Txt),
    /// A group of other objects.
    Group(Group),
}

impl Obj2 {
    /// Returns true if the object is empty (i.e. zero points)
    pub fn is_empty(&self) -> bool {
        match self {
            Obj2::Polygon(p) => p.is_empty(),
            Obj2::Group(dois) => dois.is_empty(),
            Obj2::Point(pt) => pt.is_empty(),
            Obj2::Segment(sg) => sg.is_empty(),
            Obj2::Char(ch) => ch.is_empty(),
            Obj2::CurveArc(ca) => ca.is_empty(),
        }
    }

    /// Casts each inner value to something which implements Bounded.
    pub fn inner_impl_bounded(&self) -> &dyn Bounded {
        match self {
            Obj2::Char(ch) => ch,
            Obj2::CurveArc(arc) => arc,
            Obj2::Group(dos) => dos,
            Obj2::Point(p) => p,
            Obj2::Polygon(pg) => pg,
            Obj2::Segment(s) => s,
        }
    }

    /// Casts each inner value to something which implements YieldPoints.
    pub fn inner_impl_yield_points(&self) -> &dyn YieldPoints {
        match self {
            Obj2::Point(p) => p,
            Obj2::Char(ch) => ch,
            Obj2::CurveArc(ca) => ca,
            Obj2::Group(g) => g,
            Obj2::Polygon(pg) => pg,
            Obj2::Segment(sg) => sg,
        }
    }

    /// Casts each inner value to something which implements YieldPointsMut.
    pub fn inner_impl_yield_points_mut(&mut self) -> &mut dyn YieldPointsMut {
        match self {
            Obj2::Point(p) => p,
            Obj2::Char(ch) => ch,
            Obj2::CurveArc(ca) => ca,
            Obj2::Group(g) => g,
            Obj2::Polygon(pg) => pg,
            Obj2::Segment(sg) => sg,
        }
    }
}

impl YieldPoints for Obj2 {
    fn yield_pts(&self) -> Box<dyn Iterator<Item = &Pt> + '_> {
        self.inner_impl_yield_points().yield_pts()
    }
}

impl YieldPointsMut for Obj2 {
    fn yield_pts_mut(&mut self) -> Box<dyn Iterator<Item = &mut Pt> + '_> {
        self.inner_impl_yield_points_mut().yield_pts_mut()
    }
}

impl Mutable for Obj2 {}

impl Bounded for Obj2 {
    fn bounds(&self) -> crate::bounded::Bounds {
        self.inner_impl_bounded().bounds()
    }
}

impl RemAssign<Pt> for Obj2 {
    fn rem_assign(&mut self, rhs: Pt) {
        match self {
            Obj2::Point(p) => {
                *p %= rhs;
            }
            Obj2::Char(ch) => {
                *ch %= rhs;
            }
            Obj2::CurveArc(ca) => {
                *ca %= rhs;
            }
            Obj2::Group(g) => {
                *g %= rhs;
            }
            Obj2::Polygon(pg) => {
                *pg %= rhs;
            }
            Obj2::Segment(sg) => {
                *sg %= rhs;
            }
        }
    }
}

impl Add<Pt> for Obj2 {
    type Output = Obj2;
    fn add(self, rhs: Pt) -> Self::Output {
        match self {
            Obj2::Point(p) => Obj2::from(p + rhs),
            Obj2::Char(ch) => Obj2::from(ch + rhs),
            Obj2::CurveArc(ca) => Obj2::from(ca + rhs),
            Obj2::Group(g) => Obj2::from(g + rhs),
            Obj2::Polygon(pg) => Obj2::from(pg + rhs),
            Obj2::Segment(sg) => Obj2::from(sg + rhs),
        }
    }
}

impl Sub<Pt> for Obj2 {
    type Output = Obj2;
    fn sub(self, rhs: Pt) -> Self::Output {
        match self {
            Obj2::Point(p) => Obj2::from(p - rhs),
            Obj2::Char(ch) => Obj2::from(ch - rhs),
            Obj2::CurveArc(ca) => Obj2::from(ca - rhs),
            Obj2::Group(g) => Obj2::from(g - rhs),
            Obj2::Polygon(pg) => Obj2::from(pg - rhs),
            Obj2::Segment(sg) => Obj2::from(sg - rhs),
        }
    }
}
impl Mul<f64> for Obj2 {
    type Output = Obj2;
    fn mul(self, rhs: f64) -> Self::Output {
        match self {
            Obj2::Point(p) => Obj2::from(p * rhs),
            Obj2::Char(ch) => Obj2::from(ch * rhs),
            Obj2::CurveArc(ca) => Obj2::from(ca * rhs),
            Obj2::Group(g) => Obj2::from(g * rhs),
            Obj2::Polygon(pg) => Obj2::from(pg * rhs),
            Obj2::Segment(sg) => Obj2::from(sg * rhs),
        }
    }
}
impl Div<f64> for Obj2 {
    type Output = Obj2;
    fn div(self, rhs: f64) -> Self::Output {
        match self {
            Obj2::Point(p) => Obj2::from(p / rhs),
            Obj2::Char(ch) => Obj2::from(ch / rhs),
            Obj2::CurveArc(ca) => Obj2::from(ca / rhs),
            Obj2::Group(g) => Obj2::from(g / rhs),
            Obj2::Polygon(pg) => Obj2::from(pg / rhs),
            Obj2::Segment(sg) => Obj2::from(sg / rhs),
        }
    }
}
impl AddAssign<Pt> for Obj2 {
    fn add_assign(&mut self, rhs: Pt) {
        match self {
            Obj2::Point(p) => {
                *p += rhs;
            }
            Obj2::Char(ch) => {
                *ch += rhs;
            }
            Obj2::CurveArc(ca) => {
                *ca += rhs;
            }
            Obj2::Group(g) => {
                *g += rhs;
            }
            Obj2::Polygon(pg) => {
                *pg += rhs;
            }
            Obj2::Segment(sg) => {
                *sg += rhs;
            }
        }
    }
}
impl SubAssign<Pt> for Obj2 {
    fn sub_assign(&mut self, rhs: Pt) {
        match self {
            Obj2::Point(p) => {
                *p -= rhs;
            }
            Obj2::Char(ch) => {
                *ch -= rhs;
            }
            Obj2::CurveArc(ca) => {
                *ca -= rhs;
            }
            Obj2::Group(g) => {
                *g -= rhs;
            }
            Obj2::Polygon(pg) => {
                *pg -= rhs;
            }
            Obj2::Segment(sg) => {
                *sg -= rhs;
            }
        }
    }
}

impl MulAssign<f64> for Obj2 {
    fn mul_assign(&mut self, rhs: f64) {
        match self {
            Obj2::Point(p) => {
                *p *= rhs;
            }
            Obj2::Char(ch) => {
                *ch *= rhs;
            }
            Obj2::CurveArc(ca) => {
                *ca *= rhs;
            }
            Obj2::Group(g) => {
                *g *= rhs;
            }
            Obj2::Polygon(pg) => {
                *pg *= rhs;
            }
            Obj2::Segment(sg) => {
                *sg *= rhs;
            }
        }
    }
}

impl DivAssign<f64> for Obj2 {
    fn div_assign(&mut self, rhs: f64) {
        match self {
            Obj2::Point(p) => {
                *p /= rhs;
            }
            Obj2::Char(ch) => {
                *ch /= rhs;
            }
            Obj2::CurveArc(ca) => {
                *ca /= rhs;
            }
            Obj2::Group(g) => {
                *g /= rhs;
            }
            Obj2::Polygon(pg) => {
                *pg /= rhs;
            }
            Obj2::Segment(sg) => {
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
    fn crop(&self, frame: &Polygon, crop_type: CropType) -> Vec<Self::Output> {
        match &self {
            Obj2::Point(pt) => {
                assert_eq!(crop_type, CropType::Inclusive);
                if !matches!(frame.contains_pt(pt), PointLoc::Outside) {
                    vec![self.clone()]
                } else {
                    vec![]
                }
            }
            Obj2::Polygon(pg) => match pg.kind {
                PolygonKind::Open => pg
                    .to_segments()
                    .into_iter()
                    .flat_map(|sg| sg.crop(frame, crop_type))
                    .into_iter()
                    .map(Obj2::from)
                    .collect::<Vec<_>>(),
                PolygonKind::Closed => pg
                    .crop(frame, crop_type)
                    .into_iter()
                    .map(Obj2::from)
                    .collect::<Vec<_>>(),
            },
            Obj2::Segment(sg) => sg
                .crop(frame, crop_type)
                .into_iter()
                .map(Obj2::from)
                .collect::<Vec<_>>(),
            Obj2::CurveArc(ca) => ca
                .crop(frame, crop_type)
                .into_iter()
                .map(Obj2::from)
                .collect::<Vec<_>>(),
            Obj2::Char(ch) => {
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

    fn crop_excluding(&self, other: &Polygon) -> Vec<Self::Output>
    where
        Self: Sized,
    {
        match &self {
            Obj2::Point(pt) => {
                if matches!(other.contains_pt(pt), PointLoc::Outside) {
                    vec![]
                } else {
                    vec![self.clone()]
                }
            }
            Obj2::Polygon(pg) => match pg.kind {
                PolygonKind::Open => pg
                    .to_segments()
                    .into_iter()
                    .flat_map(|sg| sg.crop_excluding(other))
                    .into_iter()
                    .map(Obj2::from)
                    .collect::<Vec<_>>(),
                PolygonKind::Closed => pg
                    .crop_excluding(other)
                    .into_iter()
                    .map(Obj2::from)
                    .collect::<Vec<_>>(),
            },
            Obj2::Segment(sg) => sg
                .crop_excluding(other)
                .into_iter()
                .map(Obj2::from)
                .collect::<Vec<_>>(),
            Obj2::CurveArc(ca) => ca
                .crop_excluding(other)
                .into_iter()
                .map(Obj2::from)
                .collect::<Vec<_>>(),
            Obj2::Char(ch) => {
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
            Obj2::Polygon(pg) => pg.annotate(settings),
            Obj2::Group(g) => g.annotate(settings),
            Obj2::Point(_) | Obj2::Segment(_) | Obj2::CurveArc(_) | Obj2::Char(_) => vec![],
        }
    }
}
