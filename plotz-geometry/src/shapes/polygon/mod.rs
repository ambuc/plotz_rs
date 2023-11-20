//! A 2D polygon (or multi&line).
#![allow(missing_docs)]

mod annotated_isxn_result;
mod crop_graph;

use self::{annotated_isxn_result::*, crop_graph::*};
use crate::{
    bounded::{Bounded, Bounds},
    crop::{CropType, Croppable, PointLocation},
    intersection::IntersectionResult,
    obj2::ObjType2d,
    overlaps::{opinion::PolygonOp, polygon_overlaps_point},
    shapes::{point::Point, segment::Segment},
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

#[derive(Debug, Clone, PartialOrd, Ord)]
pub struct Polygon {
    pub pts: Vec<Point>,
}

impl PartialEq for Polygon {
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
impl Eq for Polygon {}

/// Constructor for polygons. Polygons must have inner area, so they must have
/// three or more points. Constructing a polygon from two or fewer points will
/// result in a PolygonConstructorError
#[allow(non_snake_case)]
pub fn Polygon(a: impl IntoIterator<Item = impl Into<Point>>) -> Result<Polygon> {
    let mut pts: Vec<Point> = a.into_iter().map(|x| x.into()).collect();
    if pts.len() <= 2 {
        return Err(anyhow!("two or fewer points"));
    }

    if pts[pts.len() - 1] == pts[0] {
        let _ = pts.pop();
    }

    let mut p = Polygon { pts };
    if p.get_curve_orientation() == Some(PointListOrientation::Clockwise) {
        p.orient_curve_positively();
    }
    Ok(p)
}

/// Convenience constructor for rectangles.
#[allow(non_snake_case)]
pub fn Rect<T1, T2>(tl: impl Into<Point>, (w, h): (T1, T2)) -> Result<Polygon>
where
    f64: From<T1>,
    f64: From<T2>,
    T1: std::marker::Copy,
    T2: std::marker::Copy,
{
    let tl: Point = tl.into();
    Polygon([tl, tl + (w, 0), tl + (w, h), tl + (0, h)])
}

#[derive(Debug, PartialEq, Eq)]
pub enum PointListOrientation {
    Clockwise,
    CounterClockwise,
}

impl Polygon {
    /// Returns the segments of a polygon, one at a time.
    pub fn to_segments(&self) -> Vec<Segment> {
        zip(self.pts.iter(), self.pts.iter().cycle().skip(1))
            .map(|(x, y)| Segment(*x, *y))
            .collect()
    }

    /// A rotation operation, for rotating one polygon about a point. Accepts a
    /// |by| argument in radians.
    pub fn rotate(&mut self, about: &Point, by: f64) {
        self.pts
            .iter_mut()
            .for_each(|pt| pt.rotate_inplace(about, by))
    }

    /// Returns true if any line Sg from this polygon intersects any line
    /// segment from the other polygon.
    pub fn intersects(&self, other: &Polygon) -> bool {
        self.intersects_detailed(other).count() != 0
    }

    /// Returns the detailed set of intersection outcomes between this polygon's
    /// segments and another polygon's segments.
    pub fn intersects_detailed(&self, other: &Polygon) -> impl Iterator<Item = IntersectionResult> {
        iproduct!(self.to_segments(), other.to_segments()).flat_map(|(l1, l2)| l1.intersects(&l2))
    }

    fn annotated_intersects_detailed(&self, other: &Polygon) -> Vec<AnnotatedIsxnResult> {
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

    /// Returns the detailed set of intersection outcomes between this polygon's
    /// segments and another segment.
    fn intersects_segment_detailed(&self, other: &Segment) -> Vec<IntersectionResult> {
        self.to_segments()
            .iter()
            .flat_map(|l| l.intersects(other))
            .collect::<Vec<_>>()
    }

    /// Calculates whether a point is within, without, or along a closed polygon
    /// using the https://en.wikipedia.org/wiki/Winding_number method.
    pub fn contains_pt_deprecated(&self, other: &Point) -> Result<PointLocation> {
        match polygon_overlaps_point(self, other)? {
            None => Ok(PointLocation::Outside),
            Some((PolygonOp::WithinArea, _)) => Ok(PointLocation::Inside),
            Some((PolygonOp::Point(index, _), _)) => Ok(PointLocation::OnPoint(index)),
            Some((PolygonOp::PointAlongEdge(index, ..), _)) => Ok(PointLocation::OnSegment(index)),
            _ => Err(anyhow!("how did we get here?")),
        }
    }

    /// True if the area or points/edges of this polygon contain a point.
    pub fn point_is_inside_or_on_border_deprecated(&self, other: &Point) -> bool {
        matches!(
            polygon_overlaps_point(self, other).unwrap(),
            Some((PolygonOp::WithinArea, _))
                | Some((PolygonOp::Point(..), _))
                | Some((PolygonOp::PointAlongEdge(..), _))
        )
    }

    /// True if the area of this polygon contains a point.
    pub fn point_is_inside_deprecated(&self, other: &Point) -> bool {
        matches!(
            polygon_overlaps_point(self, other).unwrap(),
            Some((PolygonOp::WithinArea, _))
        )
    }

    /// Which curve orientation a polygon has. Curve orientation refers to
    /// whether or not the points in the polygon are stored in clockwise or
    /// counterclockwise order.
    ///
    /// If there is no internal area, returns None.
    pub fn get_curve_orientation(&self) -> Option<PointListOrientation> {
        let o = self
            .to_segments()
            .iter()
            .map(|segment| (segment.f.x - segment.i.x) * (segment.f.y + segment.i.y))
            .sum::<f64>();

        match o {
            o if approx_eq!(f64, o, 0.0) => None,
            o if o >= 0_f64 => Some(PointListOrientation::Clockwise),
            _ => Some(PointListOrientation::CounterClockwise),
        }
    }

    /// Orients a polygon in-place such that it has a positive orientation.
    pub fn orient_curve_positively(&mut self) {
        if let Some(PointListOrientation::Clockwise) = self.get_curve_orientation() {
            self.pts.reverse();
        }
    }

    /// Returns the average point across all points in the polygon. NB: Not the
    /// same as the center or centroid or whatever.
    pub fn average(&self) -> Point {
        let num: f64 = self.pts.len() as f64;
        let sum_x: f64 = self.pts.iter().map(|pt| pt.x).sum();
        let sum_y: f64 = self.pts.iter().map(|pt| pt.y).sum();
        Point(sum_x / num, sum_y / num)
    }

    // check if this polygon totally contains another.
    // assumes no intersections.
    fn totally_contains(&self, other: &Polygon) -> Result<bool> {
        Ok(other
            .pts
            .iter()
            .all(|pt| !matches!(self.contains_pt_deprecated(pt), Ok(PointLocation::Outside))))
    }

    // check if the other polygon isn't inside of or intersecting this one at all.
    // assumes no intersections.
    fn contains_not_at_all(&self, other: &Polygon) -> Result<bool> {
        Ok(other
            .pts
            .iter()
            .all(|pt| matches!(self.contains_pt_deprecated(pt), Ok(PointLocation::Outside))))
    }
}

impl Croppable for Polygon {
    type Output = Polygon;
    /// Crop this polygon to some frame (b). Returns a list of resultant polygons.
    /// Both polygons must already be closed and positively oriented.
    ///
    /// Known bug: If multiple resultant polygons are present, this will return
    /// only one.
    fn crop(&self, b: &Polygon, crop_type: CropType) -> Result<Vec<Self::Output>> {
        // tracing::info!("Cropping self \n\t{:?} \n\tto b \n\t{:?}", self, b);
        let a: &Polygon = self;

        if a == b {
            return Ok(vec![a.clone()]);
        }

        // scenario with no intersections.
        if Polygon::annotated_intersects_detailed(a, b).is_empty() {
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
pub fn abp(o: &Point, i: &Point, j: &Point) -> f64 {
    let a: Point = *i - *o;
    let b: Point = *j - *o;
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

impl IntoIterator for Polygon {
    type Item = Point;
    type IntoIter = std::vec::IntoIter<Point>;

    fn into_iter(self) -> Self::IntoIter {
        self.pts.into_iter()
    }
}

crate::ops_defaults_t!(Polygon, Point);

impl Bounded for Polygon {
    fn bounds(&self) -> Result<Bounds> {
        Ok(Bounds {
            y_max: self
                .pts
                .iter()
                .map(|p| FloatOrd(p.y))
                .max()
                .ok_or(anyhow!("not empty"))?
                .0,
            y_min: self
                .pts
                .iter()
                .map(|p| FloatOrd(p.y))
                .min()
                .ok_or(anyhow!("not empty"))?
                .0,
            x_min: self
                .pts
                .iter()
                .map(|p| FloatOrd(p.x))
                .min()
                .ok_or(anyhow!("not empty"))?
                .0,
            x_max: self
                .pts
                .iter()
                .map(|p| FloatOrd(p.x))
                .max()
                .ok_or(anyhow!("not empty"))?
                .0,
        })
    }
}

impl Object for Polygon {
    fn objtype(&self) -> ObjType2d {
        ObjType2d::Polygon2d
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &Point> + '_> {
        Box::new(self.pts.iter())
    }

    fn iter_mut(&mut self) -> Box<dyn Iterator<Item = &mut Point> + '_> {
        Box::new(self.pts.iter_mut())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use float_eq::assert_float_eq;

    #[test]
    fn test_polygon_to_segments() -> Result<()> {
        assert!(Polygon([(0, 0), (0, 1)]).is_err());

        assert_eq!(
            Polygon([(0, 0), (0, 1), (0, 2)])?.to_segments(),
            [
                Segment((0, 0), (0, 1)),
                Segment((0, 1), (0, 2)),
                Segment((0, 2), (0, 0)),
            ]
        );

        assert_eq!(
            Polygon([(0, 0), (0, 1), (0, 2), (0, 3)])?.to_segments(),
            [
                Segment((0, 0), (0, 1)),
                Segment((0, 1), (0, 2)),
                Segment((0, 2), (0, 3)),
                Segment((0, 3), (0, 0)),
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
        let a = Point(0, 2);
        let b = Point(1, 2);
        let c = Point(2, 2);
        let d = Point(0, 1);
        let e = Point(1, 1);
        let f = Point(2, 1);
        let g = Point(0, 0);
        let h = Point(1, 0);
        let i = Point(2, 0);

        // Positive area intersection.
        assert!(Polygon([a, c, i, g])?.intersects(&Polygon([b, f, h, d])?));
        assert!(Polygon([a, c, i, g])?.intersects(&Polygon([a, b, e, d])?));
        assert!(Polygon([a, c, i, g])?.intersects(&Polygon([e, f, i, h])?));

        // Shares a corner.
        assert!(Polygon([a, b, e, d])?.intersects(&Polygon([e, f, i, h])?));
        assert!(Polygon([a, b, e, d])?.intersects(&Polygon([b, c, f, e])?));

        // No intersection.
        assert!(!Polygon([a, b, d])?.intersects(&Polygon([e, f, h])?));
        assert!(!Polygon([a, b, d])?.intersects(&Polygon([f, h, i])?));
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
        let a = Point(0, 2);
        let b = Point(1, 2);
        let c = Point(2, 2);
        let d = Point(0, 1);
        let e = Point(1, 1);
        let f = Point(2, 1);
        let g = Point(0, 0);
        let h = Point(1, 0);
        let i = Point(2, 0);

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
        let a = Point(0, 2);
        let b = Point(1, 2);
        let c = Point(2, 2);
        let d = Point(0, 1);
        let e = Point(1, 1);
        let f = Point(2, 1);
        let g = Point(0, 0);
        let h = Point(1, 0);
        let i = Point(2, 0);

        // frame [a,c,i,g] should contain a, b, c, d, e, f, g, h, and i.
        let frame1 = Polygon([a, c, i, g])?;
        {
            let p = e;
            assert_eq!(frame1.contains_pt_deprecated(&p)?, PointLocation::Inside);
        }
        assert_eq!(
            frame1.contains_pt_deprecated(&a)?,
            PointLocation::OnPoint(3)
        );
        assert_eq!(
            frame1.contains_pt_deprecated(&c)?,
            PointLocation::OnPoint(2)
        );
        assert_eq!(
            frame1.contains_pt_deprecated(&i)?,
            PointLocation::OnPoint(1)
        );
        assert_eq!(
            frame1.contains_pt_deprecated(&g)?,
            PointLocation::OnPoint(0)
        );

        assert_eq!(
            frame1.contains_pt_deprecated(&d)?,
            PointLocation::OnSegment(3)
        );
        assert_eq!(
            frame1.contains_pt_deprecated(&b)?,
            PointLocation::OnSegment(2)
        );
        assert_eq!(
            frame1.contains_pt_deprecated(&f)?,
            PointLocation::OnSegment(1)
        );
        assert_eq!(
            frame1.contains_pt_deprecated(&h)?,
            PointLocation::OnSegment(0)
        );

        // frame [a,b,e,d] should contain a, b, d, e...
        let frame2 = Polygon([a, b, e, d])?;
        assert_eq!(
            frame2.contains_pt_deprecated(&a)?,
            PointLocation::OnPoint(3)
        );
        assert_eq!(
            frame2.contains_pt_deprecated(&b)?,
            PointLocation::OnPoint(2)
        );
        assert_eq!(
            frame2.contains_pt_deprecated(&e)?,
            PointLocation::OnPoint(1)
        );
        assert_eq!(
            frame2.contains_pt_deprecated(&d)?,
            PointLocation::OnPoint(0)
        );
        for p in [c, f, i, h, g] {
            assert_eq!(frame2.contains_pt_deprecated(&p)?, PointLocation::Outside);
        }

        let frame3 = Polygon([b, f, h, d])?;
        assert_eq!(
            frame3.contains_pt_deprecated(&b)?,
            PointLocation::OnPoint(3)
        );
        assert_eq!(
            frame3.contains_pt_deprecated(&f)?,
            PointLocation::OnPoint(2)
        );
        assert_eq!(
            frame3.contains_pt_deprecated(&h)?,
            PointLocation::OnPoint(1)
        );
        assert_eq!(
            frame3.contains_pt_deprecated(&d)?,
            PointLocation::OnPoint(0)
        );
        assert_eq!(frame3.contains_pt_deprecated(&e)?, PointLocation::Inside);
        for p in [a, c, g, i] {
            assert_eq!(frame3.contains_pt_deprecated(&p)?, PointLocation::Outside);
        }
        Ok(())
    }

    #[test]
    fn test_contains_pt_regression() -> Result<()> {
        let frame = Polygon([
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
        let suspicious_pt = Point(228, 400);
        assert_eq!(
            frame.contains_pt_deprecated(&suspicious_pt)?,
            PointLocation::Outside
        );
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
        let inner = Polygon([(1, 1), (3, 1), (3, 3), (1, 3)])?; // 🟥
        let frame = Polygon([(1, 1), (3, 1), (3, 3), (1, 3)])?; // 🟨
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
        let inner = Polygon([(1, 1), (3, 1), (3, 3), (1, 3)])?; // 🟥
        let frame = Polygon([(0, 0), (3, 0), (3, 3), (0, 3)])?; // 🟨
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
        let inner = Polygon([(1, 1), (3, 1), (3, 3), (1, 3)])?; // 🟥
        let frame = Polygon([(0, 0), (4, 0), (4, 4), (0, 4)])?; // 🟨

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
        let inner = Polygon([(1, 1), (4, 1), (4, 4), (1, 4)])?; // 🟥
        let frame = Polygon([(0, 0), (3, 0), (3, 3), (0, 3)])?; // 🟨
        let expected = Polygon([(1, 1), (3, 1), (3, 3), (1, 3)])?; // 🟧

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
        let inner = Polygon([(1, 0), (4, 0), (4, 3), (1, 3)])?; // 🟥
        let frame = Polygon([(0, 1), (3, 1), (3, 4), (0, 4)])?; // 🟨
        let expected = Polygon([(1, 1), (3, 1), (3, 3), (1, 3)])?; // 🟧

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
        let inner = Polygon([
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
        let frame = Polygon([(0, 1), (5, 1), (5, 4), (0, 4)])?; // 🟨
        let expected = Polygon([
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
        let inner = Polygon([
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
        let frame = Polygon([(0, 1), (5, 1), (5, 4), (0, 4)])?; // 🟨
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
        let inner = Polygon([
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
        let frame = Polygon([(1, 1), (4, 1), (4, 4), (1, 4)])?; // 🟨
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
        let a = Point(0, 2);
        let c = Point(2, 2);
        let g = Point(0, 0);
        let i = Point(2, 0);

        assert_eq!(
            Polygon([a, c, i, g])?.get_curve_orientation(),
            Some(PointListOrientation::CounterClockwise)
        );
        assert_eq!(
            Polygon([a, g, i, c])?.get_curve_orientation(),
            Some(PointListOrientation::CounterClockwise)
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
        let a = Point(0, 2);
        let c = Point(2, 2);
        let g = Point(0, 0);
        let i = Point(2, 0);
        let mut p = Polygon([a, g, i, c])?;
        assert_eq!(
            p.get_curve_orientation(),
            Some(PointListOrientation::CounterClockwise)
        );
        p.orient_curve_positively();
        assert_eq!(
            p.get_curve_orientation(),
            Some(PointListOrientation::Clockwise)
        );
        Ok(())
    }

    #[test]
    fn test_add() -> Result<()> {
        assert_eq!(
            &Polygon([(0, 0), (1, 1), (2, 2)])? + (1, 0),
            Polygon([(1, 0), (2, 1), (3, 2)])?
        );
        Ok(())
    }

    #[test]
    fn test_sub() -> Result<()> {
        assert_eq!(
            &Polygon([(0, 0), (1, 1), (2, 2)])? - (1, 0),
            Polygon([(-1, 0), (0, 1), (1, 2)])?
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
        let h = Point(1, 0);
        let f = Point(2, 1);
        let b = Point(1, 2);
        let d = Point(0, 1);
        let p = Polygon([h, f, b, d])?;
        let bounds = p.bounds()?;
        assert_eq!(bounds.y_max, 2.0);
        assert_eq!(bounds.y_min, 0.0);
        assert_eq!(bounds.x_min, 0.0);
        assert_eq!(bounds.x_max, 2.0);
        assert_eq!(bounds.x_min_y_max(), Point(0, 2));
        assert_eq!(bounds.x_min_y_min(), Point(0, 0));
        assert_eq!(bounds.x_max_y_max(), Point(2, 2));
        assert_eq!(bounds.x_max_y_min(), Point(2, 0));
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

        let frame = Polygon([
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
        let segment = Segment((0, 2), (5, 2));
        assert_eq!(
            segment.crop_to(&frame)?,
            vec![
                Segment((0, 2), (1, 2)),
                Segment((2, 2), (3, 2)),
                Segment((4, 2), (5, 2)),
            ]
        );
        Ok(())
    }

    #[test]
    fn test_frame_to_segment_crop() -> Result<()> {
        let frame = Polygon([(1, 0), (2, 1), (1, 2), (0, 1)])?;
        assert_eq!(
            Segment((0, 2), (2, 0)).crop_to(&frame)?,
            vec![Segment((0.5, 1.5), (1.5, 0.5))]
        );
        Ok(())
    }
    #[test]
    fn test_frame_to_segment_crop_02() -> Result<()> {
        let frame = Polygon([(1, 0), (2, 1), (1, 2), (0, 1)])?;
        assert_eq!(
            Segment((0, 0), (2, 2)).crop_to(&frame)?,
            vec![Segment((0.5, 0.5), (1.5, 1.5))]
        );
        Ok(())
    }
    #[test]
    fn test_frame_to_segment_crop_empty() -> Result<()> {
        let frame = Polygon([(1, 0), (2, 1), (1, 2), (0, 1)])?;
        assert_eq!(Segment((0, 2), (2, 2)).crop_to(&frame)?, vec![]);
        Ok(())
    }
    #[test]
    #[ignore]
    fn test_frame_to_segment_crop_unchanged() -> Result<()> {
        let frame = Polygon([(1, 0), (2, 1), (1, 2), (0, 1)])?;
        assert_eq!(
            Segment((0, 1), (2, 1)).crop_to(&frame)?,
            vec![Segment((0, 1), (2, 1))]
        );
        Ok(())
    }

    #[test]
    fn test_into_iter() -> Result<()> {
        let frame = Polygon([(1, 0), (2, 1), (1, 2), (0, 1)])?;
        let pts: Vec<Point> = frame.into_iter().collect();
        assert_eq!(
            pts,
            vec![Point(1, 0), Point(2, 1), Point(1, 2), Point(0, 1)]
        );
        Ok(())
    }

    #[test]
    fn test_iter() -> Result<()> {
        let src = vec![Point(1, 0), Point(2, 1), Point(1, 2), Point(0, 1)];
        let frame = Polygon(src.clone())?;
        for (idx, p) in frame.iter().enumerate() {
            assert_eq!(src[idx], *p);
        }
        Ok(())
    }
}
