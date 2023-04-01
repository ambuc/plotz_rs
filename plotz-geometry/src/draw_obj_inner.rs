use crate::crop::{CropToPolygonError, Croppable, PointLoc};

use {
    crate::{
        bounded::Bounded, curve::CurveArc, point::Pt, polygon::Polygon, segment::Segment, traits::*,
    },
    crate::{char::Char, group::Group},
    derive_more::From,
    std::ops::*,
};

/// Either a polygon or a segment.
#[derive(Debug, PartialEq, Clone, From)]
pub enum DrawObjInner {
    /// A point.
    Point(Pt),
    /// A polygon.
    Polygon(Polygon),
    /// A segment.
    Segment(Segment),
    /// An arc.
    CurveArc(CurveArc),
    /// A character to be printed in SVG, at a point.
    Char(Char),
    /// A group of other drawobjects.
    Group(Group),
}

impl DrawObjInner {
    /// Returns true if the object is empty (i.e. zero points)
    pub fn is_empty(&self) -> bool {
        match self {
            DrawObjInner::Polygon(p) => p.pts.is_empty(),
            DrawObjInner::Group(dois) => dois.iter_dois().all(|doi| doi.is_empty()),
            DrawObjInner::Point(_)
            | DrawObjInner::Segment(_)
            | DrawObjInner::Char(_)
            | DrawObjInner::CurveArc(_) => false,
        }
    }

    pub fn inner_impl_bounded(&self) -> &dyn Bounded {
        match self {
            DrawObjInner::Char(ch) => ch,
            DrawObjInner::CurveArc(arc) => arc,
            DrawObjInner::Group(dos) => dos,
            DrawObjInner::Point(p) => p,
            DrawObjInner::Polygon(pg) => pg,
            DrawObjInner::Segment(s) => s,
        }
    }
    pub fn inner_impl_yield_points(&self) -> &dyn YieldPoints {
        match self {
            DrawObjInner::Point(p) => p,
            DrawObjInner::Char(ch) => ch,
            DrawObjInner::CurveArc(ca) => ca,
            DrawObjInner::Group(g) => g,
            DrawObjInner::Polygon(pg) => pg,
            DrawObjInner::Segment(sg) => sg,
        }
    }
    pub fn inner_impl_yield_points_mut(&mut self) -> &mut dyn YieldPointsMut {
        match self {
            DrawObjInner::Point(p) => p,
            DrawObjInner::Char(ch) => ch,
            DrawObjInner::CurveArc(ca) => ca,
            DrawObjInner::Group(g) => g,
            DrawObjInner::Polygon(pg) => pg,
            DrawObjInner::Segment(sg) => sg,
        }
    }
}

impl YieldPoints for DrawObjInner {
    fn yield_pts(&self) -> Option<Box<dyn Iterator<Item = &Pt> + '_>> {
        self.inner_impl_yield_points().yield_pts()
    }
}

impl YieldPointsMut for DrawObjInner {
    fn yield_pts_mut(&mut self) -> Option<Box<dyn Iterator<Item = &mut Pt> + '_>> {
        self.inner_impl_yield_points_mut().yield_pts_mut()
    }
}

impl Mutable for DrawObjInner {}

impl Bounded for DrawObjInner {
    fn right_bound(&self) -> f64 {
        self.inner_impl_bounded().right_bound()
    }

    fn left_bound(&self) -> f64 {
        self.inner_impl_bounded().left_bound()
    }

    fn top_bound(&self) -> f64 {
        self.inner_impl_bounded().top_bound()
    }

    fn bottom_bound(&self) -> f64 {
        self.inner_impl_bounded().bottom_bound()
    }
}

impl RemAssign<Pt> for DrawObjInner {
    fn rem_assign(&mut self, rhs: Pt) {
        match self {
            DrawObjInner::Point(p) => {
                *p %= rhs;
            }
            DrawObjInner::Char(ch) => {
                *ch %= rhs;
            }
            DrawObjInner::CurveArc(ca) => {
                *ca %= rhs;
            }
            DrawObjInner::Group(g) => {
                *g %= rhs;
            }
            DrawObjInner::Polygon(pg) => {
                *pg %= rhs;
            }
            DrawObjInner::Segment(sg) => {
                *sg %= rhs;
            }
        }
    }
}

impl Add<Pt> for DrawObjInner {
    type Output = DrawObjInner;
    fn add(self, rhs: Pt) -> Self::Output {
        match self {
            DrawObjInner::Point(p) => DrawObjInner::from(p + rhs),
            DrawObjInner::Char(ch) => DrawObjInner::from(ch + rhs),
            DrawObjInner::CurveArc(ca) => DrawObjInner::from(ca + rhs),
            DrawObjInner::Group(g) => DrawObjInner::from(g + rhs),
            DrawObjInner::Polygon(pg) => DrawObjInner::from(pg + rhs),
            DrawObjInner::Segment(sg) => DrawObjInner::from(sg + rhs),
        }
    }
}

impl Sub<Pt> for DrawObjInner {
    type Output = DrawObjInner;
    fn sub(self, rhs: Pt) -> Self::Output {
        match self {
            DrawObjInner::Point(p) => DrawObjInner::from(p - rhs),
            DrawObjInner::Char(ch) => DrawObjInner::from(ch - rhs),
            DrawObjInner::CurveArc(ca) => DrawObjInner::from(ca - rhs),
            DrawObjInner::Group(g) => DrawObjInner::from(g - rhs),
            DrawObjInner::Polygon(pg) => DrawObjInner::from(pg - rhs),
            DrawObjInner::Segment(sg) => DrawObjInner::from(sg - rhs),
        }
    }
}
impl Mul<f64> for DrawObjInner {
    type Output = DrawObjInner;
    fn mul(self, rhs: f64) -> Self::Output {
        match self {
            DrawObjInner::Point(p) => DrawObjInner::from(p * rhs),
            DrawObjInner::Char(ch) => DrawObjInner::from(ch * rhs),
            DrawObjInner::CurveArc(ca) => DrawObjInner::from(ca * rhs),
            DrawObjInner::Group(g) => DrawObjInner::from(g * rhs),
            DrawObjInner::Polygon(pg) => DrawObjInner::from(pg * rhs),
            DrawObjInner::Segment(sg) => DrawObjInner::from(sg * rhs),
        }
    }
}
impl Div<f64> for DrawObjInner {
    type Output = DrawObjInner;
    fn div(self, rhs: f64) -> Self::Output {
        match self {
            DrawObjInner::Point(p) => DrawObjInner::from(p / rhs),
            DrawObjInner::Char(ch) => DrawObjInner::from(ch / rhs),
            DrawObjInner::CurveArc(ca) => DrawObjInner::from(ca / rhs),
            DrawObjInner::Group(g) => DrawObjInner::from(g / rhs),
            DrawObjInner::Polygon(pg) => DrawObjInner::from(pg / rhs),
            DrawObjInner::Segment(sg) => DrawObjInner::from(sg / rhs),
        }
    }
}
impl AddAssign<Pt> for DrawObjInner {
    fn add_assign(&mut self, rhs: Pt) {
        match self {
            DrawObjInner::Point(p) => {
                *p += rhs;
            }
            DrawObjInner::Char(ch) => {
                *ch += rhs;
            }
            DrawObjInner::CurveArc(ca) => {
                *ca += rhs;
            }
            DrawObjInner::Group(g) => {
                *g += rhs;
            }
            DrawObjInner::Polygon(pg) => {
                *pg += rhs;
            }
            DrawObjInner::Segment(sg) => {
                *sg += rhs;
            }
        }
    }
}
impl SubAssign<Pt> for DrawObjInner {
    fn sub_assign(&mut self, rhs: Pt) {
        match self {
            DrawObjInner::Point(p) => {
                *p -= rhs;
            }
            DrawObjInner::Char(ch) => {
                *ch -= rhs;
            }
            DrawObjInner::CurveArc(ca) => {
                *ca -= rhs;
            }
            DrawObjInner::Group(g) => {
                *g -= rhs;
            }
            DrawObjInner::Polygon(pg) => {
                *pg -= rhs;
            }
            DrawObjInner::Segment(sg) => {
                *sg -= rhs;
            }
        }
    }
}

impl MulAssign<f64> for DrawObjInner {
    fn mul_assign(&mut self, rhs: f64) {
        match self {
            DrawObjInner::Point(p) => {
                *p *= rhs;
            }
            DrawObjInner::Char(ch) => {
                *ch *= rhs;
            }
            DrawObjInner::CurveArc(ca) => {
                *ca *= rhs;
            }
            DrawObjInner::Group(g) => {
                *g *= rhs;
            }
            DrawObjInner::Polygon(pg) => {
                *pg *= rhs;
            }
            DrawObjInner::Segment(sg) => {
                *sg *= rhs;
            }
        }
    }
}

impl DivAssign<f64> for DrawObjInner {
    fn div_assign(&mut self, rhs: f64) {
        match self {
            DrawObjInner::Point(p) => {
                *p /= rhs;
            }
            DrawObjInner::Char(ch) => {
                *ch /= rhs;
            }
            DrawObjInner::CurveArc(ca) => {
                *ca /= rhs;
            }
            DrawObjInner::Group(g) => {
                *g /= rhs;
            }
            DrawObjInner::Polygon(pg) => {
                *pg /= rhs;
            }
            DrawObjInner::Segment(sg) => {
                *sg /= rhs;
            }
        }
    }
}

impl Translatable for DrawObjInner {}
impl Scalable<f64> for DrawObjInner {}
impl ScalableAssign for DrawObjInner {}
impl TranslatableAssign for DrawObjInner {}

impl Croppable for DrawObjInner {
    type Output = DrawObjInner;
    fn crop_to(&self, frame: &Polygon) -> Result<Vec<Self::Output>, CropToPolygonError> {
        Ok(match &self {
            DrawObjInner::Point(pt) => {
                if !matches!(frame.contains_pt(pt), Ok(PointLoc::Outside)) {
                    vec![self.clone()]
                } else {
                    vec![]
                }
            }
            DrawObjInner::Polygon(pg) => pg
                .to_segments()
                .into_iter()
                .map(|sg| sg.crop_to(&frame).expect("crop segment to frame failed"))
                .flatten()
                .map(DrawObjInner::from)
                .collect::<Vec<_>>(),
            DrawObjInner::Segment(sg) => sg
                .crop_to(frame)?
                .into_iter()
                .map(DrawObjInner::from)
                .collect::<Vec<_>>(),
            DrawObjInner::CurveArc(ca) => ca
                .crop_to(frame)?
                .into_iter()
                .map(DrawObjInner::from)
                .collect::<Vec<_>>(),
            DrawObjInner::Char(ch) => {
                if !matches!(frame.contains_pt(&ch.pt), Ok(PointLoc::Outside)) {
                    vec![self.clone()]
                } else {
                    vec![]
                }
            }
            DrawObjInner::Group(g) => g
                .crop_to(frame)?
                .into_iter()
                .map(DrawObjInner::from)
                .collect::<Vec<_>>(),
        })
    }
}
