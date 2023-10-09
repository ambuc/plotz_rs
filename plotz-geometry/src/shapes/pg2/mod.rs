//! A 2D polygon (or multi&line).

use crate::{obj2::Obj2, style::Style};

mod annotated_isxn_result;
mod crop_graph;
pub mod multiline;

use {
    self::{annotated_isxn_result::*, crop_graph::*},
    crate::{
        bounded::{Bounded, Bounds},
        crop::CropType,
        crop::{CropToPolygonError, Croppable, PointLoc},
        isxn::IntersectionResult,
        shapes::{
            pt2::Pt2,
            sg2::{Contains, Sg2},
            txt::Txt,
        },
        traits::*,
    },
    float_cmp::approx_eq,
    itertools::iproduct,
    std::{
        cmp::{Eq, PartialEq},
        fmt::Debug,
        iter::zip,
        ops::*,
    },
    thiserror::Error,
};

/// Whether a polygon is open (there should be no line drawn between its last
/// and first points) or closed (a line should be drawn between its last and
/// first points).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolygonKind {
    /// A polygon is open.
    Open,
    /// A polygon is closed.
    Closed,
}

/// A multiline is a list of points rendered with connecting line segments.
/// If constructed with PolygonKind::Open, this is a multiline (unshaded).
/// If constructed with PolygonKind::Closed, this is a closed, shaded polygon.
#[derive(Clone)]
pub struct Pg2 {
    /// The points which describe a polygon or multiline.
    pub pts: Vec<Pt2>,
    /// Whether this polygon is open or closed.
    pub kind: PolygonKind,
}

impl Debug for Pg2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Pg2 { pts, kind } = self;
        match kind {
            PolygonKind::Open => write!(f, "Multiline({:?})", pts),
            PolygonKind::Closed => write!(f, "Pg2({:?})", pts),
        }
    }
}

impl PartialEq for Pg2 {
    fn eq(&self, other: &Self) -> bool {
        let self_idx_of_min = self
            .pts
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.cmp(b))
            .map(|(idx, _)| idx)
            .unwrap();
        let self_new_pts: Vec<_> = self
            .pts
            .iter()
            .cycle()
            .skip(self_idx_of_min)
            .take(self.pts.len())
            .collect();

        let other_idx_of_min = other
            .pts
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.cmp(b))
            .map(|(idx, _)| idx)
            .unwrap();
        let other_new_pts: Vec<_> = other
            .pts
            .iter()
            .cycle()
            .skip(other_idx_of_min)
            .take(other.pts.len())
            .collect();

        self_new_pts == other_new_pts && self.kind == other.kind
    }
}

/// A general error arising from trying to construct a Pg2.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PolygonConstructorError {
    /// It is not possible to construct a polygon from two or fewer points.
    #[error("It is not possible to construct a polygon from two or fewer points.")]
    TwoOrFewerPoints,
}

/// Constructor for polygons. Polygons must have inner area, so they must have
/// three or more points. Constructing a polygon from two or fewer points will
/// result in a PolygonConstructorErrorip
#[allow(non_snake_case)]
pub fn TryPolygon(a: impl IntoIterator<Item = Pt2>) -> Result<Pg2, PolygonConstructorError> {
    let mut pts: Vec<Pt2> = a.into_iter().collect();
    if pts.len() <= 2 {
        return Err(PolygonConstructorError::TwoOrFewerPoints);
    }

    if pts[pts.len() - 1] == pts[0] {
        let _ = pts.pop();
    }

    let mut p = Pg2 {
        pts,
        kind: PolygonKind::Closed,
    };
    if p.get_curve_orientation() == Some(CurveOrientation::Negative) {
        p.orient_curve_positively();
    }
    Ok(p)
}
/// Definitely makes a polygon. Trust me.
#[allow(non_snake_case)]
pub fn Pg2(a: impl IntoIterator<Item = Pt2>) -> Pg2 {
    TryPolygon(a).unwrap()
}

/// Convenience constructor for rectangles.
#[allow(non_snake_case)]
pub fn Rect(tl: Pt2, (w, h): (f64, f64)) -> Result<Pg2, PolygonConstructorError> {
    TryPolygon([tl, tl + Pt2(w, 0.0), tl + Pt2(w, h), tl + Pt2(0.0, h)])
}

/// Whether a curve is positively or negatively oriented (whether its points are
/// listed in clockwise or counter-clockwise order).
#[derive(Debug, PartialEq, Eq)]
pub enum CurveOrientation {
    /// Negatively oriented, i.e. points listed in clockwise order.
    Negative,
    /// Positively oriented, i.e. points listed in counter-clockwise order.
    Positive,
}

impl Pg2 {
    /// Returns the segments of a polygon, one at a time.
    ///
    /// If this is an open polygon, we return only the line segments without the
    /// final closure.
    ///
    /// If this is a closed polygon, we also generate the final closure.
    ///
    /// See test_multiline_to_segments() and test_polygon_to_segments() for
    /// examples.
    pub fn to_segments(&self) -> Vec<Sg2> {
        match self.kind {
            PolygonKind::Open => zip(self.pts.iter(), self.pts.iter().skip(1))
                .map(|(x, y)| Sg2(*x, *y))
                .collect(),
            PolygonKind::Closed => zip(self.pts.iter(), self.pts.iter().cycle().skip(1))
                .map(|(x, y)| Sg2(*x, *y))
                .collect(),
        }
    }

    /// A rotation operation, for rotating one polygon about a point. Accepts a
    /// |by| argument in radians.
    pub fn rotate(&mut self, about: &Pt2, by: f64) {
        self.pts
            .iter_mut()
            .for_each(|pt| pt.rotate_inplace(about, by))
    }

    /// Returns true if any line Sg2 from this polygon intersects any line
    /// segment from the other polygon.
    pub fn intersects(&self, other: &Pg2) -> bool {
        self.intersects_detailed(other).count() != 0
    }

    /// Returns the detailed set of intersection outcomes between this polygon's
    /// segments and another polygon's segments.
    pub fn intersects_detailed(&self, other: &Pg2) -> impl Iterator<Item = IntersectionResult> {
        iproduct!(self.to_segments(), other.to_segments()).flat_map(|(l1, l2)| l1.intersects(&l2))
    }

    fn annotated_intersects_detailed(&self, other: &Pg2) -> Vec<AnnotatedIsxnResult> {
        iproduct!(
            self.to_segments().iter().enumerate(),
            other.to_segments().iter().enumerate()
        )
        .flat_map(|((a_segment_idx, a), (b_segment_idx, b))| {
            a.intersects(b).map(|isxn_result| AnnotatedIsxnResult {
                isxn_result,
                a_segment_idx,
                b_segment_idx,
            })
        })
        .collect()
    }

    /// Returns true if any line segment from this polygon intersects other.
    pub fn intersects_segment(&self, other: &Sg2) -> bool {
        self.to_segments()
            .iter()
            .any(|l| l.intersects(other).is_some())
    }

    /// Returns the detailed set of intersection outcomes between this polygon's
    /// segments and another segment.
    pub fn intersects_segment_detailed(&self, other: &Sg2) -> Vec<IntersectionResult> {
        self.to_segments()
            .iter()
            .flat_map(|l| l.intersects(other))
            .collect::<Vec<_>>()
    }

    /// Calculates whether a point is within, without, or along a closed polygon
    /// using the https://en.wikipedia.org/wiki/Winding_number method.
    pub fn contains_pt(&self, other: &Pt2) -> PointLoc {
        // If |self| is open, error out.
        if self.kind == PolygonKind::Open {
            panic!("Pg2 is open.");
        }

        for (idx, pt) in self.pts.iter().enumerate() {
            if other == pt {
                return PointLoc::OnPoint(idx);
            }
        }
        for (idx, seg) in self.to_segments().iter().enumerate() {
            match seg.line_segment_contains_pt(other) {
                Some(Contains::Within) => {
                    return PointLoc::OnSegment(idx);
                }
                Some(Contains::AtStart | Contains::AtEnd) => {
                    panic!("?");
                }
                _ => {}
            }
        }

        let mut theta = 0_f64;
        for (i, j) in zip(self.pts.iter(), self.pts.iter().cycle().skip(1)) {
            theta += abp(other, i, j)
        }

        match approx_eq!(f64, theta, 0_f64, epsilon = 0.00001) {
            true => PointLoc::Outside,
            false => PointLoc::Inside,
        }
    }

    /// True if the area or points/edges of this polygon contain a point.
    pub fn point_is_inside_or_on_border(&self, other: &Pt2) -> bool {
        matches!(
            self.contains_pt(other),
            PointLoc::Inside | PointLoc::OnPoint(_) | PointLoc::OnSegment(_)
        )
    }

    /// True if the area of this polygon contains a point.
    pub fn point_is_inside(&self, other: &Pt2) -> bool {
        matches!(self.contains_pt(other), PointLoc::Inside)
    }

    /// True if the point is totally outside the polygon.
    pub fn point_is_outside(&self, pt: &Pt2) -> bool {
        matches!(self.contains_pt(pt), PointLoc::Outside)
    }

    /// Which curve orientation a polygon has. Curve orientation refers to
    /// whether or not the points in the polygon are stored in clockwise or
    /// counterclockwise order.
    ///
    /// If there is no internal area, returns None.
    pub fn get_curve_orientation(&self) -> Option<CurveOrientation> {
        let o = self
            .to_segments()
            .iter()
            .map(|segment| (segment.f.x.0 - segment.i.x.0) * (segment.f.y.0 + segment.i.y.0))
            .sum::<f64>();

        match o {
            o if approx_eq!(f64, o, 0.0) => None,
            o if o >= 0_f64 => Some(CurveOrientation::Negative),
            _ => Some(CurveOrientation::Positive),
        }
    }

    /// Orients a polygon in-place such that it has a positive orientation.
    pub fn orient_curve_positively(&mut self) {
        if let Some(CurveOrientation::Negative) = self.get_curve_orientation() {
            self.pts.reverse();
        }
    }

    /// Returns the average point across all points in the polygon. NB: Not the
    /// same as the center or centroid or whatever.
    pub fn average(&self) -> Pt2 {
        let num: f64 = self.pts.len() as f64;
        let sum_x: f64 = self.pts.iter().map(|pt| pt.x.0).sum();
        let sum_y: f64 = self.pts.iter().map(|pt| pt.y.0).sum();
        Pt2(sum_x / num, sum_y / num)
    }

    // check that this and the other are both closed and positively oriented.
    fn crop_check_prerequisites(&self, b: &Pg2) -> Result<(), CropToPolygonError> {
        if self.kind != PolygonKind::Closed {
            return Err(CropToPolygonError::ThisPolygonNotClosed);
        }

        // frame actually MUST be closed.
        if b.kind != PolygonKind::Closed {
            return Err(CropToPolygonError::ThatPolygonNotClosed);
        }

        Ok(())
    }

    // check if this polygon totally contains another.
    // assumes no intersections.
    fn totally_contains(&self, other: &Pg2) -> bool {
        other
            .pts
            .iter()
            .all(|pt| !matches!(self.contains_pt(pt), PointLoc::Outside))
    }

    // check if the other polygon isn't inside of or intersecting this one at all.
    // assumes no intersections.
    fn contains_not_at_all(&self, other: &Pg2) -> bool {
        other
            .pts
            .iter()
            .all(|pt| matches!(self.contains_pt(pt), PointLoc::Outside))
    }
}

impl Croppable for Pg2 {
    type Output = Pg2;
    /// Crop this polygon to some frame (b). Returns a list of resultant polygons.
    /// Both polygons must already be closed and positively oriented.
    ///
    /// Known bug: If multiple resultant polygons are present, this will return
    /// only one.
    fn crop(&self, b: &Pg2, crop_type: CropType) -> Vec<Self::Output> {
        // tracing::info!("Cropping self \n\t{:?} \n\tto b \n\t{:?}", self, b);
        let a: &Pg2 = self;

        if a == b {
            return vec![a.clone()];
        }

        Pg2::crop_check_prerequisites(a, b).expect("failed prerequisites");

        // scenario with no intersections.
        if Pg2::annotated_intersects_detailed(a, b).is_empty() {
            match crop_type {
                CropType::Inclusive => {
                    // if inclusive, then we want the bit of |a| in |b|.
                    if a.totally_contains(b) {
                        // if |a| totally contains |b|, just return |b|.
                        return vec![b.clone()];
                    }
                    if b.totally_contains(a) {
                        // if |b| totally contains |a|, just return |a|.
                        return vec![a.clone()];
                    }
                    if b.contains_not_at_all(a) {
                        // if |b| doesn't contain any part of |a| (and there are
                        // no intersections) then return nothing.
                        return vec![];
                    }
                    panic!("I thought there were no intersections.");
                }
                CropType::Exclusive => {
                    // if exclusive, then we want the bit of |a| _not_ in |b|.
                    if a.totally_contains(b) {
                        panic!("we want a polygon with a cavity here -- not yet supported.");
                    }
                    if b.totally_contains(a) {
                        // if |b| totally contains |a|, then there's no part of
                        // |a| we want.
                        return vec![];
                    }
                    if b.contains_not_at_all(a) {
                        // if |b| doesn't contain any part of |a| (and there are
                        // no intersections) then return A unchanged.
                        return vec![a.clone()];
                    }
                }
            }
        }

        let (resultant, _crop_graph) = CropGraph::run(a, b, crop_type);
        resultant
    }
}

/// Angle between points. Projects OI onto OJ and finds the angle IOJ.
pub fn abp(o: &Pt2, i: &Pt2, j: &Pt2) -> f64 {
    let a: Pt2 = *i - *o;
    let b: Pt2 = *j - *o;
    let angle = f64::atan2(
        /*det=*/ a.x.0 * b.y.0 - a.y.0 * b.x.0,
        /*dot=*/ a.x.0 * b.x.0 + a.y.0 * b.y.0,
    );

    if approx_eq!(f64, angle, 0.0) {
        0.0
    } else {
        angle
    }
}

impl Add<Pt2> for &Pg2 {
    type Output = Pg2;
    fn add(self, rhs: Pt2) -> Self::Output {
        Pg2(self.pts.iter().map(|p| *p + rhs))
    }
}
impl Add<Pt2> for Pg2 {
    type Output = Pg2;
    fn add(self, rhs: Pt2) -> Self::Output {
        &self + rhs
    }
}
impl AddAssign<Pt2> for Pg2 {
    fn add_assign(&mut self, rhs: Pt2) {
        self.pts.iter_mut().for_each(|p| *p += rhs);
    }
}
impl Div<Pt2> for Pg2 {
    type Output = Pg2;
    fn div(self, rhs: Pt2) -> Self::Output {
        Pg2(self.pts.iter().map(|p| *p / rhs))
    }
}
impl Div<f64> for Pg2 {
    type Output = Pg2;
    fn div(self, rhs: f64) -> Self::Output {
        Pg2(self.pts.iter().map(|p| *p / rhs))
    }
}
impl DivAssign<Pt2> for Pg2 {
    fn div_assign(&mut self, rhs: Pt2) {
        self.pts.iter_mut().for_each(|p| *p /= rhs);
    }
}
impl DivAssign<f64> for Pg2 {
    fn div_assign(&mut self, rhs: f64) {
        self.pts.iter_mut().for_each(|p| *p /= rhs);
    }
}
impl Mul<Pt2> for Pg2 {
    type Output = Pg2;
    fn mul(self, rhs: Pt2) -> Pg2 {
        Pg2(self.pts.iter().map(|p| *p * rhs))
    }
}
impl Mul<f64> for Pg2 {
    type Output = Pg2;
    fn mul(mut self, rhs: f64) -> Pg2 {
        self *= rhs;
        self
    }
}
impl MulAssign<Pt2> for Pg2 {
    fn mul_assign(&mut self, rhs: Pt2) {
        self.pts.iter_mut().for_each(|p| *p *= rhs);
    }
}
impl MulAssign<f64> for Pg2 {
    fn mul_assign(&mut self, rhs: f64) {
        self.pts.iter_mut().for_each(|p| *p *= rhs);
    }
}
impl Sub<Pt2> for &Pg2 {
    type Output = Pg2;
    fn sub(self, rhs: Pt2) -> Self::Output {
        Pg2(self.pts.iter().map(|p| *p - rhs))
    }
}
impl Sub<Pt2> for Pg2 {
    type Output = Pg2;
    fn sub(self, rhs: Pt2) -> Self::Output {
        Pg2(self.pts.iter().map(|p| *p - rhs))
    }
}
impl SubAssign<Pt2> for Pg2 {
    fn sub_assign(&mut self, rhs: Pt2) {
        self.pts.iter_mut().for_each(|p| *p -= rhs);
    }
}
impl RemAssign<Pt2> for Pg2 {
    fn rem_assign(&mut self, rhs: Pt2) {
        self.pts.iter_mut().for_each(|p| *p %= rhs);
    }
}

impl Bounded for Pg2 {
    fn bounds(&self) -> crate::bounded::Bounds {
        Bounds {
            top_bound: self.pts.iter().map(|p| p.y).max().expect("not empty").0,
            bottom_bound: self.pts.iter().map(|p| p.y).min().expect("not empty").0,
            left_bound: self.pts.iter().map(|p| p.x).min().expect("not empty").0,
            right_bound: self.pts.iter().map(|p| p.x).max().expect("not empty").0,
        }
    }
}
impl YieldPoints for Pg2 {
    fn yield_pts(&self) -> Box<dyn Iterator<Item = &Pt2> + '_> {
        Box::new(self.pts.iter())
    }
}
impl YieldPointsMut for Pg2 {
    fn yield_pts_mut(&mut self) -> Box<dyn Iterator<Item = &mut Pt2> + '_> {
        Box::new(self.pts.iter_mut())
    }
}
impl Mutable for Pg2 {}

impl Translatable for Pg2 {}
impl Scalable<Pt2> for Pg2 {}
impl Scalable<f64> for Pg2 {}

impl Roundable for Pg2 {
    fn round_to_nearest(&mut self, f: f64) {
        self.pts.iter_mut().for_each(|pt| pt.round_to_nearest(f));
    }
}

impl Nullable for Pg2 {
    fn is_empty(&self) -> bool {
        self.pts.is_empty()
    }
}

impl Annotatable for Pg2 {
    fn annotate(&self, settings: &AnnotationSettings) -> Vec<(Obj2, Style)> {
        let mut a = vec![];

        let AnnotationSettings {
            font_size,
            precision,
        } = settings;
        for (_idx, pt) in self.pts.iter().enumerate() {
            let x = format!("{:.1$}", pt.x.0, precision);
            let y = format!("{:.1$}", pt.y.0, precision);
            a.push((
                Txt {
                    pt: *pt,
                    inner: format!("({}, {})", x, y),
                    font_size: *font_size,
                }
                .into(),
                Style::default(),
            ));
        }

        // for (idx, sg) in self.to_segments().iter().enumerate() {
        //     a.push(Object2d::new(Txt {
        //         pt: sg.i.avg(&sg.f),
        //         inner: format!("s{}", idx.to_string()),
        //     }));
        // }

        a
    }
}

#[cfg(test)]
mod tests {
    use crate::shapes::pg2::multiline::{Multiline, MultilineConstructorError};

    use super::*;
    use float_eq::assert_float_eq;

    #[test]
    fn test_multiline_to_segments() {
        assert_eq!(
            Multiline([Pt2(0, 0)]).unwrap_err(),
            MultilineConstructorError::OneOrFewerPoints
        );
        assert_eq!(
            Multiline([Pt2(0, 0), Pt2(0, 1)]).unwrap().to_segments(),
            [Sg2(Pt2(0, 0), Pt2(0, 1)),]
        );
        assert_eq!(
            Multiline([Pt2(0, 0), Pt2(0, 1), Pt2(0, 2)])
                .unwrap()
                .to_segments(),
            [Sg2(Pt2(0, 0), Pt2(0, 1)), Sg2(Pt2(0, 1), Pt2(0, 2)),]
        );
        assert_eq!(
            Multiline([Pt2(0, 0), Pt2(0, 1), Pt2(0, 2), Pt2(0, 3)])
                .unwrap()
                .to_segments(),
            [
                Sg2(Pt2(0, 0), Pt2(0, 1)),
                Sg2(Pt2(0, 1), Pt2(0, 2)),
                Sg2(Pt2(0, 2), Pt2(0, 3)),
            ]
        );
    }

    #[test]
    fn test_polygon_to_segments() {
        assert_eq!(
            TryPolygon([Pt2(0, 0), Pt2(0, 1)]).unwrap_err(),
            PolygonConstructorError::TwoOrFewerPoints,
        );

        assert_eq!(
            Pg2([Pt2(0, 0), Pt2(0, 1), Pt2(0, 2)]).to_segments(),
            [
                Sg2(Pt2(0, 0), Pt2(0, 1)),
                Sg2(Pt2(0, 1), Pt2(0, 2)),
                Sg2(Pt2(0, 2), Pt2(0, 0)),
            ]
        );

        assert_eq!(
            Pg2([Pt2(0, 0), Pt2(0, 1), Pt2(0, 2), Pt2(0, 3)]).to_segments(),
            [
                Sg2(Pt2(0, 0), Pt2(0, 1)),
                Sg2(Pt2(0, 1), Pt2(0, 2)),
                Sg2(Pt2(0, 2), Pt2(0, 3)),
                Sg2(Pt2(0, 3), Pt2(0, 0)),
            ]
        );
    }

    #[test]
    fn test_intersects() {
        //   ^
        //   |
        //   A  B  C
        //   |
        //   D  E  F
        //   |
        // --G--H--I->
        //   |
        let a = Pt2(0, 2);
        let b = Pt2(1, 2);
        let c = Pt2(2, 2);
        let d = Pt2(0, 1);
        let e = Pt2(1, 1);
        let f = Pt2(2, 1);
        let g = Pt2(0, 0);
        let h = Pt2(1, 0);
        let i = Pt2(2, 0);

        // Positive area intersection.
        assert!(Pg2([a, c, i, g]).intersects(&Pg2([b, f, h, d])));
        assert!(Pg2([a, c, i, g]).intersects(&Pg2([a, b, e, d])));
        assert!(Pg2([a, c, i, g]).intersects(&Pg2([e, f, i, h])));

        // Shares a corner.
        assert!(Pg2([a, b, e, d]).intersects(&Pg2([e, f, i, h])));
        assert!(Pg2([a, b, e, d]).intersects(&Pg2([b, c, f, e])));

        // No intersection.
        assert!(!Pg2([a, b, d]).intersects(&Pg2([e, f, h])));
        assert!(!Pg2([a, b, d]).intersects(&Pg2([f, h, i])));
    }

    #[test]
    fn test_angle_between_points() {
        use std::f64::consts::PI;
        //   ^
        //   |
        //   A  B  C
        //   |
        //   D  E  F
        //   |
        // --G--H--I->
        //   |
        let a = Pt2(0, 2);
        let b = Pt2(1, 2);
        let c = Pt2(2, 2);
        let d = Pt2(0, 1);
        let e = Pt2(1, 1);
        let f = Pt2(2, 1);
        let g = Pt2(0, 0);
        let h = Pt2(1, 0);
        let i = Pt2(2, 0);

        // circle around E. (quadrants 1, 2, 3, 4)
        assert_float_eq!(abp(&e, &f, &b), PI / 2.0, ulps <= 10);
        assert_float_eq!(abp(&e, &f, &d), PI, ulps <= 10);
        assert_float_eq!(abp(&e, &f, &h), -1.0 * PI / 2.0, ulps <= 10);
        assert_float_eq!(abp(&e, &f, &f), 0.0, ulps <= 10);

        // circle around E, inverse. (quadrants 1, 2, 3, 4)
        assert_float_eq!(abp(&e, &f, &h), -1.0 * PI / 2.0, ulps <= 10);
        assert_float_eq!(abp(&e, &f, &d), PI, ulps <= 10);
        assert_float_eq!(abp(&e, &f, &b), PI / 2.0, ulps <= 10);
        assert_float_eq!(abp(&e, &f, &f), 0.0, ulps <= 10);

        // circle around G. (quadrant 1)
        assert_float_eq!(abp(&g, &i, &i), 0.0, ulps <= 10);
        assert_float_eq!(abp(&g, &i, &h), 0.0, ulps <= 10);
        assert_float_eq!(abp(&g, &i, &f), 0.5_f64.atan(), ulps <= 10);
        assert_float_eq!(abp(&g, &i, &e), 1.0_f64.atan(), ulps <= 10);
        assert_float_eq!(abp(&g, &i, &c), 1.0_f64.atan(), ulps <= 10);
        assert_float_eq!(abp(&g, &i, &b), 2.0_f64.atan(), ulps <= 10);
        assert_float_eq!(abp(&g, &i, &d), PI / 2.0, ulps <= 10);
        assert_float_eq!(abp(&g, &i, &a), PI / 2.0, ulps <= 10);

        // circle around H (quadrants 1, 2)
        assert_float_eq!(abp(&h, &i, &i), 0.0, ulps <= 10);
        assert_float_eq!(abp(&h, &i, &b), PI / 2.0, ulps <= 10);
        assert_float_eq!(abp(&h, &i, &a), PI / 2.0 + 0.5_f64.atan(), ulps <= 10);
        assert_float_eq!(abp(&h, &i, &d), PI / 2.0 + 1.0_f64.atan(), ulps <= 10);
        assert_float_eq!(abp(&h, &i, &g), PI, ulps <= 10);

        // circle around B (quadrants 3, 4)
        assert_float_eq!(abp(&b, &c, &c), 0.0, ulps <= 10);
        assert_float_eq!(abp(&b, &c, &f), -1.0_f64.atan(), ulps <= 10);
        assert_float_eq!(abp(&b, &c, &i), -2.0_f64.atan(), ulps <= 10);
        assert_float_eq!(abp(&b, &c, &e), -1.0 * PI / 2.0, ulps <= 10);
        assert_float_eq!(abp(&b, &c, &h), -1.0 * PI / 2.0, ulps <= 10);
        assert_float_eq!(
            abp(&b, &c, &g),
            -1.0 * PI / 2.0 - 0.5_f64.atan(),
            ulps <= 10
        );
        assert_float_eq!(abp(&b, &c, &d), -3.0 * PI / 4.0, ulps <= 10);
    }

    #[test]
    fn test_contains_p2() {
        //   ^
        //   |
        //   A  B  C
        //   |
        //   D  E  F
        //   |
        // --G--H--I->
        //   |
        let a = Pt2(0, 2);
        let b = Pt2(1, 2);
        let c = Pt2(2, 2);
        let d = Pt2(0, 1);
        let e = Pt2(1, 1);
        let f = Pt2(2, 1);
        let g = Pt2(0, 0);
        let h = Pt2(1, 0);
        let i = Pt2(2, 0);

        // frame [a,c,i,g] should contain a, b, c, d, e, f, g, h, and i.
        let frame1 = Pg2([a, c, i, g]);
        {
            let p = e;
            assert_eq!(frame1.contains_pt(&p), PointLoc::Inside);
        }
        assert_eq!(frame1.contains_pt(&a), PointLoc::OnPoint(3));
        assert_eq!(frame1.contains_pt(&c), PointLoc::OnPoint(2));
        assert_eq!(frame1.contains_pt(&i), PointLoc::OnPoint(1));
        assert_eq!(frame1.contains_pt(&g), PointLoc::OnPoint(0));

        assert_eq!(frame1.contains_pt(&d), PointLoc::OnSegment(3));
        assert_eq!(frame1.contains_pt(&b), PointLoc::OnSegment(2));
        assert_eq!(frame1.contains_pt(&f), PointLoc::OnSegment(1));
        assert_eq!(frame1.contains_pt(&h), PointLoc::OnSegment(0));

        // frame [a,b,e,d] should contain a, b, d, e...
        let frame2 = Pg2([a, b, e, d]);
        assert_eq!(frame2.contains_pt(&a), PointLoc::OnPoint(3));
        assert_eq!(frame2.contains_pt(&b), PointLoc::OnPoint(2));
        assert_eq!(frame2.contains_pt(&e), PointLoc::OnPoint(1));
        assert_eq!(frame2.contains_pt(&d), PointLoc::OnPoint(0));
        for p in [c, f, i, h, g] {
            assert_eq!(frame2.contains_pt(&p), PointLoc::Outside);
        }

        let frame3 = Pg2([b, f, h, d]);
        assert_eq!(frame3.contains_pt(&b), PointLoc::OnPoint(3));
        assert_eq!(frame3.contains_pt(&f), PointLoc::OnPoint(2));
        assert_eq!(frame3.contains_pt(&h), PointLoc::OnPoint(1));
        assert_eq!(frame3.contains_pt(&d), PointLoc::OnPoint(0));
        assert_eq!(frame3.contains_pt(&e), PointLoc::Inside);
        for p in [a, c, g, i] {
            assert_eq!(frame3.contains_pt(&p), PointLoc::Outside);
        }
    }

    #[test]
    fn test_contains_pt_regression() {
        let frame = Pg2([
            Pt2(228.17, 202.35),
            Pt2(231.21, 212.64),
            Pt2(232.45, 228.76),
            Pt2(231.67, 257.09),
            Pt2(230.63, 265.17),
            Pt2(263.66, 335.37),
            Pt2(261.85, 336.27),
            Pt2(295.65, 404.87),
            Pt2(298.24, 409.14),
            Pt2(302.39, 413.67),
            Pt2(305.92, 412.20),
            Pt2(309.33, 417.90),
            Pt2(311.03, 417.06),
            Pt2(312.99, 420.06),
            Pt2(318.55, 420.99),
            Pt2(322.66, 420.45),
            Pt2(325.57, 419.13),
            Pt2(343.70, 406.83),
            Pt2(336.17, 404.87),
            Pt2(230.61, 185.93),
            Pt2(228.83, 189.47),
            Pt2(227.19, 195.84),
            Pt2(228.17, 202.35),
        ]);
        let suspicious_pt = Pt2(228, 400);
        assert_eq!(frame.contains_pt(&suspicious_pt), PointLoc::Outside);
    }

    #[test]
    #[should_panic]

    fn test_crop_to_polygon_this_not_closed() {
        let _ = Multiline([Pt2(1, 1), Pt2(3, 1), Pt2(3, 3), Pt2(1, 3)])
            .unwrap()
            .crop_to(&Rect(Pt2(0., 0.), (4., 4.)).unwrap());
    }

    #[test]
    #[should_panic]
    fn test_crop_to_polygon_that_not_closed() {
        let _ = Rect(Pt2(1., 1.), (2., 2.))
            .unwrap()
            .crop_to(&Multiline([Pt2(0, 0), Pt2(4, 0), Pt2(4, 4), Pt2(0, 4)]).unwrap());
    }

    #[test]
    fn test_crop_to_polygon_inner_equals_frame() {
        // ⬆️ y
        // ⬜⬜⬜⬜⬜
        // ⬜🟧🟧🟧⬜
        // ⬜🟧🟧🟧⬜
        // ⬜🟧🟧🟧⬜
        // ⬜⬜⬜⬜⬜ ➡️ x
        let inner = Pg2([Pt2(1, 1), Pt2(3, 1), Pt2(3, 3), Pt2(1, 3)]); // 🟥
        let frame = Pg2([Pt2(1, 1), Pt2(3, 1), Pt2(3, 3), Pt2(1, 3)]); // 🟨
        assert_eq!(inner, frame);
        let crops = inner.crop_to(&frame); // 🟧
        assert_eq!(crops, vec![inner]);
    }

    #[test]
    fn test_crop_to_polygon_inner_colinear_to_frame() {
        // ⬆️ y
        // ⬜⬜⬜⬜⬜
        // 🟨🟧🟧🟧⬜
        // 🟨🟧🟧🟧⬜
        // 🟨🟧🟧🟧⬜
        // 🟨🟨🟨🟨⬜ ➡️ x
        let inner = Pg2([Pt2(1, 1), Pt2(3, 1), Pt2(3, 3), Pt2(1, 3)]); // 🟥
        let frame = Pg2([Pt2(0, 0), Pt2(3, 0), Pt2(3, 3), Pt2(0, 3)]); // 🟨
        assert_eq!(inner.crop_to(&frame)[0], inner);

        // ⬆️ y
        // ⬜⬜⬜⬜⬜
        // ⬜🟧🟧🟧🟨
        // ⬜🟧🟧🟧🟨
        // ⬜🟧🟧🟧🟨
        // ⬜🟨🟨🟨🟨 ➡️ x
        assert_eq!(inner.crop_to(&(&frame + Pt2(1, 0)))[0], inner,);

        // ⬆️ y
        // 🟨🟨🟨🟨⬜
        // 🟨🟧🟧🟧⬜
        // 🟨🟧🟧🟧⬜
        // 🟨🟧🟧🟧⬜
        // ⬜⬜⬜⬜⬜ ➡ x
        assert_eq!(inner.crop_to(&(&frame + Pt2(0, 1)))[0], inner);

        // ⬆️ y
        // ⬜🟨🟨🟨🟨
        // ⬜🟧🟧🟧🟨
        // ⬜🟧🟧🟧🟨
        // ⬜🟧🟧🟧🟨
        // ⬜⬜⬜⬜⬜ ➡ x
        assert_eq!(inner.crop_to(&(&frame + Pt2(1, 1)))[0], inner,);
    }

    #[test]
    fn test_crop_to_polygon_inner_totally_within_frame() {
        // ⬆️ y
        // 🟨🟨🟨🟨🟨
        // 🟨🟧🟧🟧🟨
        // 🟨🟧🟧🟧🟨
        // 🟨🟧🟧🟧🟨
        // 🟨🟨🟨🟨🟨 ➡️ x
        let inner = Pg2([Pt2(1, 1), Pt2(3, 1), Pt2(3, 3), Pt2(1, 3)]); // 🟥
        let frame = Pg2([Pt2(0, 0), Pt2(4, 0), Pt2(4, 4), Pt2(0, 4)]); // 🟨

        // inner /\ frame == inner
        let crops = inner.crop_to(&frame); // 🟧
        assert_eq!(crops, vec![inner.clone()]);
    }

    #[test]
    fn test_crop_to_polygon_two_pivots() {
        // ⬆️ y
        // ⬜⬜⬜⬜⬜
        // ⬜🟥🟥🟥⬜
        // 🟨🟧🟧🟥⬜
        // 🟨🟧🟧🟥⬜
        // 🟨🟨🟨⬜⬜ ➡️ x
        let inner = Pg2([Pt2(1, 1), Pt2(4, 1), Pt2(4, 4), Pt2(1, 4)]); // 🟥
        let frame = Pg2([Pt2(0, 0), Pt2(3, 0), Pt2(3, 3), Pt2(0, 3)]); // 🟨
        let expected = Pg2([Pt2(1, 1), Pt2(3, 1), Pt2(3, 3), Pt2(1, 3)]); // 🟧

        let crops = inner.crop_to(&frame);
        assert_eq!(crops, vec![expected.clone()]);
    }

    #[test]
    fn test_crop_to_polygon_two_pivots_02() {
        // ⬆️ y
        // ⬜⬜⬜⬜⬜
        // 🟨🟨🟨⬜⬜
        // 🟨🟧🟧🟥⬜
        // 🟨🟧🟧🟥⬜
        // ⬜🟥🟥🟥⬜ ➡️ x
        let inner = Pg2([Pt2(1, 0), Pt2(4, 0), Pt2(4, 3), Pt2(1, 3)]); // 🟥
        let frame = Pg2([Pt2(0, 1), Pt2(3, 1), Pt2(3, 4), Pt2(0, 4)]); // 🟨
        let expected = Pg2([Pt2(1, 1), Pt2(3, 1), Pt2(3, 3), Pt2(1, 3)]); // 🟧

        let crops = inner.crop_to(&frame);
        assert_eq!(crops, vec![expected.clone()]);
    }

    #[test]
    fn test_crop_to_polygon_many_pivots_01() {
        // ⬆️ y
        // ⬜🟥⬜🟥⬜
        // 🟨🟧🟨🟧🟨
        // 🟨🟧🟧🟧🟨
        // 🟨🟧🟨🟧🟨
        // ⬜🟥⬜🟥⬜
        let inner = Pg2([
            Pt2(1, 0),
            Pt2(2, 0),
            Pt2(2, 2),
            Pt2(3, 2),
            Pt2(3, 0),
            Pt2(4, 0),
            Pt2(4, 5),
            Pt2(3, 5),
            Pt2(3, 3),
            Pt2(2, 3),
            Pt2(2, 5),
            Pt2(1, 5),
        ]); // 🟥
        let frame = Pg2([Pt2(0, 1), Pt2(5, 1), Pt2(5, 4), Pt2(0, 4)]); // 🟨
        let expected = Pg2([
            Pt2(1, 1),
            Pt2(2, 1),
            Pt2(2, 2),
            Pt2(3, 2),
            Pt2(3, 1),
            Pt2(4, 1),
            Pt2(4, 4),
            Pt2(3, 4),
            Pt2(3, 3),
            Pt2(2, 3),
            Pt2(2, 4),
            Pt2(1, 4),
        ]); // 🟧

        let crops = inner.crop_to(&frame);
        assert_eq!(crops, vec![expected.clone()]);
    }

    #[test]
    fn test_crop_to_polygon_many_pivots_02() {
        // ⬆️ y
        // ⬜⬜⬜⬜⬜
        // 🟨🟧🟨🟧🟨
        // 🟨🟧🟧🟧🟨
        // 🟨🟧🟨🟧🟨
        // ⬜⬜⬜⬜⬜
        let inner = Pg2([
            Pt2(1, 1),
            Pt2(2, 1),
            Pt2(2, 2),
            Pt2(3, 2),
            Pt2(3, 1),
            Pt2(4, 1),
            Pt2(4, 4),
            Pt2(3, 4),
            Pt2(3, 3),
            Pt2(2, 3),
            Pt2(2, 4),
            Pt2(1, 4),
        ]); // 🟥
        let frame = Pg2([Pt2(0, 1), Pt2(5, 1), Pt2(5, 4), Pt2(0, 4)]); // 🟨
        let expected = inner.clone();
        let crops = inner.crop_to(&frame);
        assert_eq!(crops, vec![expected.clone()]);
    }

    #[test]
    fn test_crop_to_polygon_many_pivots_03() {
        // ⬆️ y
        // ⬜⬜⬜⬜⬜
        // ⬜🟧🟨🟧⬜
        // ⬜🟧🟧🟧⬜
        // ⬜🟧🟨🟧⬜
        // ⬜⬜⬜⬜⬜
        let inner = Pg2([
            Pt2(1, 1),
            Pt2(2, 1),
            Pt2(2, 2),
            Pt2(3, 2),
            Pt2(3, 1),
            Pt2(4, 1),
            Pt2(4, 4),
            Pt2(3, 4),
            Pt2(3, 3),
            Pt2(2, 3),
            Pt2(2, 4),
            Pt2(1, 4),
        ]); // 🟥
        let frame = Pg2([Pt2(1, 1), Pt2(4, 1), Pt2(4, 4), Pt2(1, 4)]); // 🟨
        let expected = inner.clone();
        let crops = inner.crop_to(&frame);
        assert_eq!(crops, vec![expected.clone()]);
    }

    // #[test]
    // #[ignore]
    // fn test_crop_to_polygon_concavities_01() {
    //     // ⬆️ y
    //     // ⬜🟨🟨🟨⬜
    //     // ⬜🟨⬜🟨⬜
    //     // 🟥🟧🟥🟧🟥
    //     // 🟥🟧🟥🟧🟥
    //     // ⬜🟨⬜🟨⬜
    //     let inner = Pg2([
    //         Pt2(1, 0),
    //         Pt2(2, 0),
    //         Pt2(2, 4),
    //         Pt2(3, 4),
    //         Pt2(3, 0),
    //         Pt2(4, 0),
    //         Pt2(4, 5),
    //         Pt2(1, 5),
    //     ])
    //     .unwrap();
    //     let frame = Pg2([Pt2(0, 1), Pt2(5, 1), Pt2(5, 3), Pt2(0, 3)]).unwrap();
    //     let expected = vec![
    //         Pg2([Pt2(1, 1), Pt2(2, 1), Pt2(2, 3), Pt2(1, 3)]).unwrap(),
    //         Pg2([Pt2(3, 1), Pt2(4, 1), Pt2(4, 3), Pt2(3, 3)]).unwrap(),
    //     ];
    //     let crops = inner.crop_to(&frame).unwrap();
    //     assert_eq!(crops.len(), 2);
    //     assert_eq!(crops[0], expected[0]);
    //     assert_eq!(crops[1], expected[1]);
    // }

    #[test]
    fn test_polygon_get_curve_orientation() {
        //   ^
        //   |
        //   A  B  C
        //   |
        //   D  E  F
        //   |
        // --G--H--I->
        //   |
        let a = Pt2(0, 2);
        let c = Pt2(2, 2);
        let g = Pt2(0, 0);
        let i = Pt2(2, 0);

        assert_eq!(
            Pg2([a, c, i, g]).get_curve_orientation(),
            Some(CurveOrientation::Positive)
        );
        assert_eq!(
            Pg2([a, g, i, c]).get_curve_orientation(),
            Some(CurveOrientation::Positive)
        );
    }

    #[test]
    #[ignore]
    fn test_polygon_orient_curve() {
        //   ^
        //   |
        //   A  B  C
        //   |
        //   D  E  F
        //   |
        // --G--H--I->
        //   |
        let a = Pt2(0, 2);
        let c = Pt2(2, 2);
        let g = Pt2(0, 0);
        let i = Pt2(2, 0);
        let mut p = Pg2([a, g, i, c]);
        assert_eq!(p.get_curve_orientation(), Some(CurveOrientation::Positive));
        p.orient_curve_positively();
        assert_eq!(p.get_curve_orientation(), Some(CurveOrientation::Negative));
    }

    #[test]
    fn test_add() {
        assert_eq!(
            &Pg2([Pt2(0, 0), Pt2(1, 1), Pt2(2, 2)]) + Pt2(1, 0),
            Pg2([Pt2(1, 0), Pt2(2, 1), Pt2(3, 2)])
        );
    }

    #[test]
    fn test_sub() {
        assert_eq!(
            &Pg2([Pt2(0, 0), Pt2(1, 1), Pt2(2, 2)]) - Pt2(1, 0),
            Pg2([Pt2(-1, 0), Pt2(0, 1), Pt2(1, 2)])
        );
    }

    #[test]
    fn test_bounded() {
        //   ^
        //   |
        //   A  B  C
        //   |
        //   D  E  F
        //   |
        // --G--H--I->
        //   |
        let h = Pt2(1, 0);
        let f = Pt2(2, 1);
        let b = Pt2(1, 2);
        let d = Pt2(0, 1);
        let p = Pg2([h, f, b, d]);
        assert_eq!(p.top_bound(), 2.0);
        assert_eq!(p.bottom_bound(), 0.0);
        assert_eq!(p.left_bound(), 0.0);
        assert_eq!(p.right_bound(), 2.0);
        assert_eq!(p.tl_bound(), Pt2(0, 2));
        assert_eq!(p.bl_bound(), Pt2(0, 0));
        assert_eq!(p.tr_bound(), Pt2(2, 2));
        assert_eq!(p.br_bound(), Pt2(2, 0));
    }

    #[test]
    fn test_frame_to_segment_many_outputs() {
        // ^ y
        // |
        // 4 - - + - - + - - + - - + - - +
        // |xxxxx|xxxxx|xxxxx| .   |xxxxx|
        // |xxxxx|xxxxx|xxxxx| .   |xxxxx|
        // 3 - - + - - + - - + - - + - - +
        // |xxxxx| .   |xxxxx| .   |xxxxx|
        // |xxxxx| .   |xxxxx| .   |xxxxx|
        // 2OOOOOOOOOOOOOOOOOOOOOOOOOOOOOO
        // |xxxxx| .   |xxxxx| .   |xxxxx|
        // |xxxxx| .   |xxxxx| .   |xxxxx|
        // 1 - - + - - + - - + - - + - - +
        // |xxxxx| .   |xxxxx|xxxxx|xxxxx|
        // |xxxxx| .   |xxxxx|xxxxx|xxxxx|
        // 0 - - 1 - - 2 - - 3 - - 4 - - 5 -> x

        let frame = Pg2([
            Pt2(0, 0),
            Pt2(1, 0),
            Pt2(1, 3),
            Pt2(2, 3),
            Pt2(2, 0),
            Pt2(5, 0),
            Pt2(5, 4),
            Pt2(4, 4),
            Pt2(4, 1),
            Pt2(3, 1),
            Pt2(3, 5),
            Pt2(0, 5),
        ]);
        let segment = Sg2(Pt2(0, 2), Pt2(5, 2));
        assert_eq!(
            segment.crop_to(&frame),
            vec![
                Sg2(Pt2(0, 2), Pt2(1, 2)),
                Sg2(Pt2(2, 2), Pt2(3, 2)),
                Sg2(Pt2(4, 2), Pt2(5, 2)),
            ]
        );
    }

    #[test]
    fn test_frame_to_segment_crop() {
        let frame = Pg2([Pt2(1, 0), Pt2(2, 1), Pt2(1, 2), Pt2(0, 1)]);
        assert_eq!(
            Sg2(Pt2(0, 2), Pt2(2, 0)).crop_to(&frame),
            vec![Sg2(Pt2(0.5, 1.5), Pt2(1.5, 0.5))]
        );
    }
    #[test]
    fn test_frame_to_segment_crop_02() {
        let frame = Pg2([Pt2(1, 0), Pt2(2, 1), Pt2(1, 2), Pt2(0, 1)]);
        assert_eq!(
            Sg2(Pt2(0, 0), Pt2(2, 2)).crop_to(&frame),
            vec![Sg2(Pt2(0.5, 0.5), Pt2(1.5, 1.5))]
        );
    }
    #[test]
    fn test_frame_to_segment_crop_empty() {
        let frame = Pg2([Pt2(1, 0), Pt2(2, 1), Pt2(1, 2), Pt2(0, 1)]);
        assert_eq!(Sg2(Pt2(0, 2), Pt2(2, 2)).crop_to(&frame), vec![]);
    }
    #[test]
    fn test_frame_to_segment_crop_unchanged() {
        let frame = Pg2([Pt2(1, 0), Pt2(2, 1), Pt2(1, 2), Pt2(0, 1)]);
        assert_eq!(
            Sg2(Pt2(0, 1), Pt2(2, 1)).crop_to(&frame),
            vec![Sg2(Pt2(0, 1), Pt2(2, 1))]
        );
    }
}
