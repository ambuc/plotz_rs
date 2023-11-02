//! A 2D polygon (or multi&line).
#![allow(missing_docs)]

mod annotated_isxn_result;
mod crop_graph;

use self::{annotated_isxn_result::*, crop_graph::*};
use crate::{
    bounded::{Bounded, Bounds},
    crop::{CropType, Croppable, PointLoc},
    intersection::IntersectionResult,
    obj::{Obj, ObjType},
    shapes::{
        pt::Pt,
        sg::{Contains, Sg},
        txt::Txt,
    },
    style::Style,
    *,
};
use anyhow::{anyhow, Context, Result};
use float_cmp::approx_eq;
use float_ord::FloatOrd;
use itertools::iproduct;
use std::{
    cmp::{Eq, PartialEq},
    fmt::Debug,
    iter::zip,
    ops::*,
};

#[derive(Debug, Clone)]
pub struct Pg {
    pub pts: Vec<Pt>,
}

impl PartialEq for Pg {
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

        self_new_pts == other_new_pts
    }
}

/// Constructor for polygons. Polygons must have inner area, so they must have
/// three or more points. Constructing a polygon from two or fewer points will
/// result in a PolygonConstructorError
#[allow(non_snake_case)]
pub fn Pg(a: impl IntoIterator<Item = impl Into<Pt>>) -> Result<Pg> {
    let mut pts: Vec<Pt> = a.into_iter().map(|x| x.into()).collect();
    if pts.len() <= 2 {
        return Err(anyhow!("two or fewer points"));
    }

    if pts[pts.len() - 1] == pts[0] {
        let _ = pts.pop();
    }

    let mut p = Pg { pts };
    if p.get_curve_orientation() == Some(CurveOrientation::Negative) {
        p.orient_curve_positively();
    }
    Ok(p)
}

/// Convenience constructor for rectangles.
#[allow(non_snake_case)]
pub fn Rect<T1, T2>(tl: impl Into<Pt>, (w, h): (T1, T2)) -> Result<Pg>
where
    f64: From<T1>,
    f64: From<T2>,
    T1: std::marker::Copy,
    T2: std::marker::Copy,
{
    let tl: Pt = tl.into();
    Pg([tl, tl + (w, 0), tl + (w, h), tl + (0, h)])
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

impl Pg {
    /// Returns the segments of a polygon, one at a time.
    pub fn to_segments(&self) -> Vec<Sg> {
        zip(self.pts.iter(), self.pts.iter().cycle().skip(1))
            .map(|(x, y)| Sg(*x, *y))
            .collect()
    }

    /// A rotation operation, for rotating one polygon about a point. Accepts a
    /// |by| argument in radians.
    pub fn rotate(&mut self, about: &Pt, by: f64) {
        self.pts
            .iter_mut()
            .for_each(|pt| pt.rotate_inplace(about, by))
    }

    /// Returns true if any line Sg from this polygon intersects any line
    /// segment from the other polygon.
    pub fn intersects(&self, other: &Pg) -> bool {
        self.intersects_detailed(other).count() != 0
    }

    /// Returns the detailed set of intersection outcomes between this polygon's
    /// segments and another polygon's segments.
    pub fn intersects_detailed(&self, other: &Pg) -> impl Iterator<Item = IntersectionResult> {
        iproduct!(self.to_segments(), other.to_segments()).flat_map(|(l1, l2)| l1.intersects(&l2))
    }

    fn annotated_intersects_detailed(&self, other: &Pg) -> Vec<AnnotatedIsxnResult> {
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
    pub fn intersects_segment(&self, other: &Sg) -> bool {
        self.to_segments()
            .iter()
            .any(|l| l.intersects(other).is_some())
    }

    /// Returns the detailed set of intersection outcomes between this polygon's
    /// segments and another segment.
    pub fn intersects_segment_detailed(&self, other: &Sg) -> Vec<IntersectionResult> {
        self.to_segments()
            .iter()
            .flat_map(|l| l.intersects(other))
            .collect::<Vec<_>>()
    }

    /// Calculates whether a point is within, without, or along a closed polygon
    /// using the https://en.wikipedia.org/wiki/Winding_number method.
    pub fn contains_pt(&self, other: &Pt) -> Result<PointLoc> {
        for (idx, pt) in self.pts.iter().enumerate() {
            if other == pt {
                return Ok(PointLoc::OnPoint(idx));
            }
        }
        for (idx, seg) in self.to_segments().iter().enumerate() {
            match seg.line_segment_contains_pt(other) {
                Some(Contains::Within) => {
                    return Ok(PointLoc::OnSegment(idx));
                }
                Some(Contains::AtStart | Contains::AtEnd) => {
                    return Err(anyhow!("not sure what is going on here"));
                }
                _ => {}
            }
        }

        let mut theta = 0_f64;
        for (i, j) in zip(self.pts.iter(), self.pts.iter().cycle().skip(1)) {
            theta += abp(other, i, j)
        }

        match approx_eq!(f64, theta, 0_f64, epsilon = 0.00001) {
            true => Ok(PointLoc::Outside),
            false => Ok(PointLoc::Inside),
        }
    }

    /// True if the area or points/edges of this polygon contain a point.
    pub fn point_is_inside_or_on_border(&self, other: &Pt) -> bool {
        matches!(
            self.contains_pt(other),
            Ok(PointLoc::Inside | PointLoc::OnPoint(_) | PointLoc::OnSegment(_))
        )
    }

    /// True if the area of this polygon contains a point.
    pub fn point_is_inside(&self, other: &Pt) -> bool {
        matches!(self.contains_pt(other), Ok(PointLoc::Inside))
    }

    /// True if the point is totally outside the polygon.
    pub fn point_is_outside(&self, pt: &Pt) -> bool {
        matches!(self.contains_pt(pt), Ok(PointLoc::Outside))
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
            .map(|segment| (segment.f.x - segment.i.x) * (segment.f.y + segment.i.y))
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
    pub fn average(&self) -> Pt {
        let num: f64 = self.pts.len() as f64;
        let sum_x: f64 = self.pts.iter().map(|pt| pt.x).sum();
        let sum_y: f64 = self.pts.iter().map(|pt| pt.y).sum();
        Pt(sum_x / num, sum_y / num)
    }

    // check if this polygon totally contains another.
    // assumes no intersections.
    fn totally_contains(&self, other: &Pg) -> Result<bool> {
        Ok(other
            .pts
            .iter()
            .all(|pt| !matches!(self.contains_pt(pt), Ok(PointLoc::Outside))))
    }

    // check if the other polygon isn't inside of or intersecting this one at all.
    // assumes no intersections.
    fn contains_not_at_all(&self, other: &Pg) -> Result<bool> {
        Ok(other
            .pts
            .iter()
            .all(|pt| matches!(self.contains_pt(pt), Ok(PointLoc::Outside))))
    }

    /// Iterator.
    pub fn iter(&self) -> impl Iterator<Item = &Pt> {
        self.pts.iter()
    }

    /// Mutable iterator.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Pt> {
        self.pts.iter_mut()
    }

    pub fn objtype(&self) -> ObjType {
        ObjType::Polygon
    }
}

impl Croppable for Pg {
    type Output = Pg;
    /// Crop this polygon to some frame (b). Returns a list of resultant polygons.
    /// Both polygons must already be closed and positively oriented.
    ///
    /// Known bug: If multiple resultant polygons are present, this will return
    /// only one.
    fn crop(&self, b: &Pg, crop_type: CropType) -> Result<Vec<Self::Output>> {
        // tracing::info!("Cropping self \n\t{:?} \n\tto b \n\t{:?}", self, b);
        let a: &Pg = self;

        if a == b {
            return Ok(vec![a.clone()]);
        }

        // scenario with no intersections.
        if Pg::annotated_intersects_detailed(a, b).is_empty() {
            match crop_type {
                CropType::Inclusive => {
                    // if inclusive, then we want the bit of |a| in |b|.
                    if a.totally_contains(b)? {
                        // if |a| totally contains |b|, just return |b|.
                        return Ok(vec![b.clone()]);
                    }
                    if b.totally_contains(a)? {
                        // if |b| totally contains |a|, just return |a|.
                        return Ok(vec![a.clone()]);
                    }
                    if b.contains_not_at_all(a)? {
                        // if |b| doesn't contain any part of |a| (and there are
                        // no intersections) then return nothing.
                        return Ok(vec![]);
                    }
                    return Err(anyhow!("I thought there were no intersections!"));
                }
                CropType::Exclusive => {
                    // if exclusive, then we want the bit of |a| _not_ in |b|.
                    if a.totally_contains(b)? {
                        // TODO(https://github.com/ambuc/plotz_rs/issues/4):
                        // must begin to support polygons with cavities !!!
                        return Err(anyhow!(
                            "we want a polygon with a cavity here - not yet supported. See https://github.com/ambuc/plotz_rs/issues/4."
                        ));
                    }
                    if b.totally_contains(a)? {
                        // if |b| totally contains |a|, then there's no part of
                        // |a| we want.
                        return Ok(vec![]);
                    }
                    if b.contains_not_at_all(a)? {
                        // if |b| doesn't contain any part of |a| (and there are
                        // no intersections) then return A unchanged.
                        return Ok(vec![a.clone()]);
                    }
                }
            }
        }

        let (resultant, _crop_graph) =
            CropGraph::run(a, b, crop_type).context("crop graph failed")?;
        Ok(resultant)
    }
}

/// Angle between points. Projects OI onto OJ and finds the angle IOJ.
pub fn abp(o: &Pt, i: &Pt, j: &Pt) -> f64 {
    let a: Pt = *i - *o;
    let b: Pt = *j - *o;
    let angle = f64::atan2(
        /*det=*/ a.x * b.y - a.y * b.x,
        /*dot=*/ a.x * b.x + a.y * b.y,
    );

    if approx_eq!(f64, angle, 0.0) {
        0.0
    } else {
        angle
    }
}

impl IntoIterator for Pg {
    type Item = Pt;
    type IntoIter = std::vec::IntoIter<Pt>;

    fn into_iter(self) -> Self::IntoIter {
        self.pts.into_iter()
    }
}

crate::ops_defaults_t!(Pg, Pt);

impl Bounded for Pg {
    fn bounds(&self) -> Result<Bounds> {
        Ok(Bounds {
            top_bound: self
                .pts
                .iter()
                .map(|p| FloatOrd(p.y))
                .max()
                .ok_or(anyhow!("not empty"))?
                .0,
            bottom_bound: self
                .pts
                .iter()
                .map(|p| FloatOrd(p.y))
                .min()
                .ok_or(anyhow!("not empty"))?
                .0,
            left_bound: self
                .pts
                .iter()
                .map(|p| FloatOrd(p.x))
                .min()
                .ok_or(anyhow!("not empty"))?
                .0,
            right_bound: self
                .pts
                .iter()
                .map(|p| FloatOrd(p.x))
                .max()
                .ok_or(anyhow!("not empty"))?
                .0,
        })
    }
}

impl Roundable for Pg {
    fn round_to_nearest(&mut self, f: f64) {
        self.pts.iter_mut().for_each(|pt| pt.round_to_nearest(f));
    }
}

impl Object for Pg {
    fn annotate(&self, settings: &AnnotationSettings) -> Vec<(Obj, Style)> {
        let mut a = vec![];

        let AnnotationSettings {
            font_size,
            precision,
        } = settings;
        for (_idx, pt) in self.pts.iter().enumerate() {
            let x = format!("{:.1$}", pt.x, precision);
            let y = format!("{:.1$}", pt.y, precision);
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

        a
    }

    fn is_empty(&self) -> bool {
        self.pts.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use float_eq::assert_float_eq;

    #[test]
    fn test_polygon_to_segments() -> Result<()> {
        assert!(Pg([(0, 0), (0, 1)]).is_err());

        assert_eq!(
            Pg([(0, 0), (0, 1), (0, 2)])?.to_segments(),
            [Sg((0, 0), (0, 1)), Sg((0, 1), (0, 2)), Sg((0, 2), (0, 0)),]
        );

        assert_eq!(
            Pg([(0, 0), (0, 1), (0, 2), (0, 3)])?.to_segments(),
            [
                Sg((0, 0), (0, 1)),
                Sg((0, 1), (0, 2)),
                Sg((0, 2), (0, 3)),
                Sg((0, 3), (0, 0)),
            ]
        );
        Ok(())
    }

    #[test]
    fn test_intersects() -> Result<()> {
        //   ^
        //   |
        //   A  B  C
        //   |
        //   D  E  F
        //   |
        // --G--H--I->
        //   |
        let a = Pt(0, 2);
        let b = Pt(1, 2);
        let c = Pt(2, 2);
        let d = Pt(0, 1);
        let e = Pt(1, 1);
        let f = Pt(2, 1);
        let g = Pt(0, 0);
        let h = Pt(1, 0);
        let i = Pt(2, 0);

        // Positive area intersection.
        assert!(Pg([a, c, i, g])?.intersects(&Pg([b, f, h, d])?));
        assert!(Pg([a, c, i, g])?.intersects(&Pg([a, b, e, d])?));
        assert!(Pg([a, c, i, g])?.intersects(&Pg([e, f, i, h])?));

        // Shares a corner.
        assert!(Pg([a, b, e, d])?.intersects(&Pg([e, f, i, h])?));
        assert!(Pg([a, b, e, d])?.intersects(&Pg([b, c, f, e])?));

        // No intersection.
        assert!(!Pg([a, b, d])?.intersects(&Pg([e, f, h])?));
        assert!(!Pg([a, b, d])?.intersects(&Pg([f, h, i])?));
        Ok(())
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
        let a = Pt(0, 2);
        let b = Pt(1, 2);
        let c = Pt(2, 2);
        let d = Pt(0, 1);
        let e = Pt(1, 1);
        let f = Pt(2, 1);
        let g = Pt(0, 0);
        let h = Pt(1, 0);
        let i = Pt(2, 0);

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
    fn test_contains_p2() -> Result<()> {
        //   ^
        //   |
        //   A  B  C
        //   |
        //   D  E  F
        //   |
        // --G--H--I->
        //   |
        let a = Pt(0, 2);
        let b = Pt(1, 2);
        let c = Pt(2, 2);
        let d = Pt(0, 1);
        let e = Pt(1, 1);
        let f = Pt(2, 1);
        let g = Pt(0, 0);
        let h = Pt(1, 0);
        let i = Pt(2, 0);

        // frame [a,c,i,g] should contain a, b, c, d, e, f, g, h, and i.
        let frame1 = Pg([a, c, i, g])?;
        {
            let p = e;
            assert_eq!(frame1.contains_pt(&p)?, PointLoc::Inside);
        }
        assert_eq!(frame1.contains_pt(&a)?, PointLoc::OnPoint(3));
        assert_eq!(frame1.contains_pt(&c)?, PointLoc::OnPoint(2));
        assert_eq!(frame1.contains_pt(&i)?, PointLoc::OnPoint(1));
        assert_eq!(frame1.contains_pt(&g)?, PointLoc::OnPoint(0));

        assert_eq!(frame1.contains_pt(&d)?, PointLoc::OnSegment(3));
        assert_eq!(frame1.contains_pt(&b)?, PointLoc::OnSegment(2));
        assert_eq!(frame1.contains_pt(&f)?, PointLoc::OnSegment(1));
        assert_eq!(frame1.contains_pt(&h)?, PointLoc::OnSegment(0));

        // frame [a,b,e,d] should contain a, b, d, e...
        let frame2 = Pg([a, b, e, d])?;
        assert_eq!(frame2.contains_pt(&a)?, PointLoc::OnPoint(3));
        assert_eq!(frame2.contains_pt(&b)?, PointLoc::OnPoint(2));
        assert_eq!(frame2.contains_pt(&e)?, PointLoc::OnPoint(1));
        assert_eq!(frame2.contains_pt(&d)?, PointLoc::OnPoint(0));
        for p in [c, f, i, h, g] {
            assert_eq!(frame2.contains_pt(&p)?, PointLoc::Outside);
        }

        let frame3 = Pg([b, f, h, d])?;
        assert_eq!(frame3.contains_pt(&b)?, PointLoc::OnPoint(3));
        assert_eq!(frame3.contains_pt(&f)?, PointLoc::OnPoint(2));
        assert_eq!(frame3.contains_pt(&h)?, PointLoc::OnPoint(1));
        assert_eq!(frame3.contains_pt(&d)?, PointLoc::OnPoint(0));
        assert_eq!(frame3.contains_pt(&e)?, PointLoc::Inside);
        for p in [a, c, g, i] {
            assert_eq!(frame3.contains_pt(&p)?, PointLoc::Outside);
        }
        Ok(())
    }

    #[test]
    fn test_contains_pt_regression() -> Result<()> {
        let frame = Pg([
            (228.17, 202.35),
            (231.21, 212.64),
            (232.45, 228.76),
            (231.67, 257.09),
            (230.63, 265.17),
            (263.66, 335.37),
            (261.85, 336.27),
            (295.65, 404.87),
            (298.24, 409.14),
            (302.39, 413.67),
            (305.92, 412.20),
            (309.33, 417.90),
            (311.03, 417.06),
            (312.99, 420.06),
            (318.55, 420.99),
            (322.66, 420.45),
            (325.57, 419.13),
            (343.70, 406.83),
            (336.17, 404.87),
            (230.61, 185.93),
            (228.83, 189.47),
            (227.19, 195.84),
            (228.17, 202.35),
        ])?;
        let suspicious_pt = Pt(228, 400);
        assert_eq!(frame.contains_pt(&suspicious_pt)?, PointLoc::Outside);
        Ok(())
    }

    #[test]
    fn test_crop_to_polygon_inner_equals_frame() -> Result<()> {
        // ⬆️ y
        // ⬜⬜⬜⬜⬜
        // ⬜🟧🟧🟧⬜
        // ⬜🟧🟧🟧⬜
        // ⬜🟧🟧🟧⬜
        // ⬜⬜⬜⬜⬜ ➡️ x
        let inner = Pg([(1, 1), (3, 1), (3, 3), (1, 3)])?; // 🟥
        let frame = Pg([(1, 1), (3, 1), (3, 3), (1, 3)])?; // 🟨
        assert_eq!(inner, frame);
        let crops = inner.crop_to(&frame)?; // 🟧
        assert_eq!(crops, vec![inner]);
        Ok(())
    }

    #[test]
    fn test_crop_to_polygon_inner_colinear_to_frame() -> Result<()> {
        // ⬆️ y
        // ⬜⬜⬜⬜⬜
        // 🟨🟧🟧🟧⬜
        // 🟨🟧🟧🟧⬜
        // 🟨🟧🟧🟧⬜
        // 🟨🟨🟨🟨⬜ ➡️ x
        let inner = Pg([(1, 1), (3, 1), (3, 3), (1, 3)])?; // 🟥
        let frame = Pg([(0, 0), (3, 0), (3, 3), (0, 3)])?; // 🟨
        assert_eq!(inner.crop_to(&frame)?[0], inner);

        // ⬆️ y
        // ⬜⬜⬜⬜⬜
        // ⬜🟧🟧🟧🟨
        // ⬜🟧🟧🟧🟨
        // ⬜🟧🟧🟧🟨
        // ⬜🟨🟨🟨🟨 ➡️ x
        assert_eq!(inner.crop_to(&(&frame + (1, 0)))?[0], inner,);

        // ⬆️ y
        // 🟨🟨🟨🟨⬜
        // 🟨🟧🟧🟧⬜
        // 🟨🟧🟧🟧⬜
        // 🟨🟧🟧🟧⬜
        // ⬜⬜⬜⬜⬜ ➡ x
        assert_eq!(inner.crop_to(&(&frame + (0, 1)))?[0], inner);

        // ⬆️ y
        // ⬜🟨🟨🟨🟨
        // ⬜🟧🟧🟧🟨
        // ⬜🟧🟧🟧🟨
        // ⬜🟧🟧🟧🟨
        // ⬜⬜⬜⬜⬜ ➡ x
        assert_eq!(inner.crop_to(&(&frame + (1, 1)))?[0], inner,);
        Ok(())
    }

    #[test]
    fn test_crop_to_polygon_inner_totally_within_frame() -> Result<()> {
        // ⬆️ y
        // 🟨🟨🟨🟨🟨
        // 🟨🟧🟧🟧🟨
        // 🟨🟧🟧🟧🟨
        // 🟨🟧🟧🟧🟨
        // 🟨🟨🟨🟨🟨 ➡️ x
        let inner = Pg([(1, 1), (3, 1), (3, 3), (1, 3)])?; // 🟥
        let frame = Pg([(0, 0), (4, 0), (4, 4), (0, 4)])?; // 🟨

        // inner /\ frame == inner
        let crops = inner.crop_to(&frame)?; // 🟧
        assert_eq!(crops, vec![inner.clone()]);
        Ok(())
    }

    #[test]
    fn test_crop_to_polygon_two_pivots() -> Result<()> {
        // ⬆️ y
        // ⬜⬜⬜⬜⬜
        // ⬜🟥🟥🟥⬜
        // 🟨🟧🟧🟥⬜
        // 🟨🟧🟧🟥⬜
        // 🟨🟨🟨⬜⬜ ➡️ x
        let inner = Pg([(1, 1), (4, 1), (4, 4), (1, 4)])?; // 🟥
        let frame = Pg([(0, 0), (3, 0), (3, 3), (0, 3)])?; // 🟨
        let expected = Pg([(1, 1), (3, 1), (3, 3), (1, 3)])?; // 🟧

        let crops = inner.crop_to(&frame)?;
        assert_eq!(crops, vec![expected.clone()]);
        Ok(())
    }

    #[test]
    fn test_crop_to_polygon_two_pivots_02() -> Result<()> {
        // ⬆️ y
        // ⬜⬜⬜⬜⬜
        // 🟨🟨🟨⬜⬜
        // 🟨🟧🟧🟥⬜
        // 🟨🟧🟧🟥⬜
        // ⬜🟥🟥🟥⬜ ➡️ x
        let inner = Pg([(1, 0), (4, 0), (4, 3), (1, 3)])?; // 🟥
        let frame = Pg([(0, 1), (3, 1), (3, 4), (0, 4)])?; // 🟨
        let expected = Pg([(1, 1), (3, 1), (3, 3), (1, 3)])?; // 🟧

        let crops = inner.crop_to(&frame)?;
        assert_eq!(crops, vec![expected.clone()]);
        Ok(())
    }

    #[test]
    fn test_crop_to_polygon_many_pivots_01() -> Result<()> {
        // ⬆️ y
        // ⬜🟥⬜🟥⬜
        // 🟨🟧🟨🟧🟨
        // 🟨🟧🟧🟧🟨
        // 🟨🟧🟨🟧🟨
        // ⬜🟥⬜🟥⬜
        let inner = Pg([
            (1, 0),
            (2, 0),
            (2, 2),
            (3, 2),
            (3, 0),
            (4, 0),
            (4, 5),
            (3, 5),
            (3, 3),
            (2, 3),
            (2, 5),
            (1, 5),
        ])?; // 🟥
        let frame = Pg([(0, 1), (5, 1), (5, 4), (0, 4)])?; // 🟨
        let expected = Pg([
            (1, 1),
            (2, 1),
            (2, 2),
            (3, 2),
            (3, 1),
            (4, 1),
            (4, 4),
            (3, 4),
            (3, 3),
            (2, 3),
            (2, 4),
            (1, 4),
        ])?; // 🟧

        let crops = inner.crop_to(&frame)?;
        assert_eq!(crops, vec![expected.clone()]);
        Ok(())
    }

    #[test]
    fn test_crop_to_polygon_many_pivots_02() -> Result<()> {
        // ⬆️ y
        // ⬜⬜⬜⬜⬜
        // 🟨🟧🟨🟧🟨
        // 🟨🟧🟧🟧🟨
        // 🟨🟧🟨🟧🟨
        // ⬜⬜⬜⬜⬜
        let inner = Pg([
            (1, 1),
            (2, 1),
            (2, 2),
            (3, 2),
            (3, 1),
            (4, 1),
            (4, 4),
            (3, 4),
            (3, 3),
            (2, 3),
            (2, 4),
            (1, 4),
        ])?; // 🟥
        let frame = Pg([(0, 1), (5, 1), (5, 4), (0, 4)])?; // 🟨
        let expected = inner.clone();
        let crops = inner.crop_to(&frame)?;
        assert_eq!(crops, vec![expected.clone()]);
        Ok(())
    }

    #[test]
    fn test_crop_to_polygon_many_pivots_03() -> Result<()> {
        // ⬆️ y
        // ⬜⬜⬜⬜⬜
        // ⬜🟧🟨🟧⬜
        // ⬜🟧🟧🟧⬜
        // ⬜🟧🟨🟧⬜
        // ⬜⬜⬜⬜⬜
        let inner = Pg([
            (1, 1),
            (2, 1),
            (2, 2),
            (3, 2),
            (3, 1),
            (4, 1),
            (4, 4),
            (3, 4),
            (3, 3),
            (2, 3),
            (2, 4),
            (1, 4),
        ])?; // 🟥
        let frame = Pg([(1, 1), (4, 1), (4, 4), (1, 4)])?; // 🟨
        let expected = inner.clone();
        let crops = inner.crop_to(&frame)?;
        assert_eq!(crops, vec![expected.clone()]);
        Ok(())
    }

    #[test]
    fn test_polygon_get_curve_orientation() -> Result<()> {
        //   ^
        //   |
        //   A  B  C
        //   |
        //   D  E  F
        //   |
        // --G--H--I->
        //   |
        let a = Pt(0, 2);
        let c = Pt(2, 2);
        let g = Pt(0, 0);
        let i = Pt(2, 0);

        assert_eq!(
            Pg([a, c, i, g])?.get_curve_orientation(),
            Some(CurveOrientation::Positive)
        );
        assert_eq!(
            Pg([a, g, i, c])?.get_curve_orientation(),
            Some(CurveOrientation::Positive)
        );
        Ok(())
    }

    #[test]
    #[ignore]
    fn test_polygon_orient_curve() -> Result<()> {
        //   ^
        //   |
        //   A  B  C
        //   |
        //   D  E  F
        //   |
        // --G--H--I->
        //   |
        let a = Pt(0, 2);
        let c = Pt(2, 2);
        let g = Pt(0, 0);
        let i = Pt(2, 0);
        let mut p = Pg([a, g, i, c])?;
        assert_eq!(p.get_curve_orientation(), Some(CurveOrientation::Positive));
        p.orient_curve_positively();
        assert_eq!(p.get_curve_orientation(), Some(CurveOrientation::Negative));
        Ok(())
    }

    #[test]
    fn test_add() -> Result<()> {
        assert_eq!(
            &Pg([(0, 0), (1, 1), (2, 2)])? + (1, 0),
            Pg([(1, 0), (2, 1), (3, 2)])?
        );
        Ok(())
    }

    #[test]
    fn test_sub() -> Result<()> {
        assert_eq!(
            &Pg([(0, 0), (1, 1), (2, 2)])? - (1, 0),
            Pg([(-1, 0), (0, 1), (1, 2)])?
        );
        Ok(())
    }

    #[test]
    fn test_bounded() -> Result<()> {
        //   ^
        //   |
        //   A  B  C
        //   |
        //   D  E  F
        //   |
        // --G--H--I->
        //   |
        let h = Pt(1, 0);
        let f = Pt(2, 1);
        let b = Pt(1, 2);
        let d = Pt(0, 1);
        let p = Pg([h, f, b, d])?;
        let bounds = p.bounds()?;
        assert_eq!(bounds.t(), 2.0);
        assert_eq!(bounds.b(), 0.0);
        assert_eq!(bounds.l(), 0.0);
        assert_eq!(bounds.r(), 2.0);
        assert_eq!(bounds.tl(), Pt(0, 2));
        assert_eq!(bounds.bl(), Pt(0, 0));
        assert_eq!(bounds.tr(), Pt(2, 2));
        assert_eq!(bounds.br(), Pt(2, 0));
        Ok(())
    }

    #[test]
    fn test_frame_to_segment_many_outputs() -> Result<()> {
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

        let frame = Pg([
            (0, 0),
            (1, 0),
            (1, 3),
            (2, 3),
            (2, 0),
            (5, 0),
            (5, 4),
            (4, 4),
            (4, 1),
            (3, 1),
            (3, 5),
            (0, 5),
        ])?;
        let segment = Sg((0, 2), (5, 2));
        assert_eq!(
            segment.crop_to(&frame)?,
            vec![Sg((0, 2), (1, 2)), Sg((2, 2), (3, 2)), Sg((4, 2), (5, 2)),]
        );
        Ok(())
    }

    #[test]
    fn test_frame_to_segment_crop() -> Result<()> {
        let frame = Pg([(1, 0), (2, 1), (1, 2), (0, 1)])?;
        assert_eq!(
            Sg((0, 2), (2, 0)).crop_to(&frame)?,
            vec![Sg((0.5, 1.5), (1.5, 0.5))]
        );
        Ok(())
    }
    #[test]
    fn test_frame_to_segment_crop_02() -> Result<()> {
        let frame = Pg([(1, 0), (2, 1), (1, 2), (0, 1)])?;
        assert_eq!(
            Sg((0, 0), (2, 2)).crop_to(&frame)?,
            vec![Sg((0.5, 0.5), (1.5, 1.5))]
        );
        Ok(())
    }
    #[test]
    fn test_frame_to_segment_crop_empty() -> Result<()> {
        let frame = Pg([(1, 0), (2, 1), (1, 2), (0, 1)])?;
        assert_eq!(Sg((0, 2), (2, 2)).crop_to(&frame)?, vec![]);
        Ok(())
    }
    #[test]
    fn test_frame_to_segment_crop_unchanged() -> Result<()> {
        let frame = Pg([(1, 0), (2, 1), (1, 2), (0, 1)])?;
        assert_eq!(
            Sg((0, 1), (2, 1)).crop_to(&frame)?,
            vec![Sg((0, 1), (2, 1))]
        );
        Ok(())
    }

    #[test]
    fn test_into_iter() -> Result<()> {
        let frame = Pg([(1, 0), (2, 1), (1, 2), (0, 1)])?;
        let pts: Vec<Pt> = frame.into_iter().collect();
        assert_eq!(pts, vec![Pt(1, 0), Pt(2, 1), Pt(1, 2), Pt(0, 1)]);
        Ok(())
    }

    #[test]
    fn test_iter() -> Result<()> {
        let src = vec![Pt(1, 0), Pt(2, 1), Pt(1, 2), Pt(0, 1)];
        let frame = Pg(src.clone())?;
        for (idx, p) in frame.iter().enumerate() {
            assert_eq!(src[idx], *p);
        }
        Ok(())
    }
}
