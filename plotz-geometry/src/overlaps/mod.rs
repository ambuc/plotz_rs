#![allow(missing_docs)]

pub mod opinion;

use crate::{
    interpolate::interpolate_2d_checked,
    obj2::Obj2,
    overlaps::opinion::{
        multiline_opinion::{MultilineOp, MultilineOpSet},
        polygon_opinion::{PolygonOp, PolygonOpSet},
        segment_opinion::{SegmentOp, SegmentOpSet},
    },
    shapes::{
        multiline::Multiline,
        point::Point,
        polygon::{abp, Polygon},
        segment::Segment,
    },
    utils::Percent::{One, Zero},
};
use anyhow::{anyhow, Result};
use float_cmp::approx_eq;
use nonempty::{nonempty, NonEmpty};

//           || pt | sg | ml | ca |
// ==========++====+====+====+====+==
//     point || ✔️  | \  | \  | \  |
//   segment || ✔️  | ✔️  | \  | \  |
// multiline || ✔️  | ✔️  |️ ✔️  | \  |
//  curvearc || -️  | -️  |️ -  | -  |
// ==========++====+====+====+====+==
pub fn totally_covers(o1: &Obj2, o2: &Obj2) -> Result<bool> {
    match (o1, o2) {
        // if {Text, CurveArc, Group, PolygonWithCavities} is o1 or o2,
        // then we haven't implemented this yet.
        (
            Obj2::Text(_) | Obj2::CurveArc(_) | Obj2::Group(_) | Obj2::PolygonWithCavities(_),
            _,
        )
        | (
            _,
            Obj2::Text(_) | Obj2::CurveArc(_) | Obj2::Group(_) | Obj2::PolygonWithCavities(_),
        ) => Err(anyhow!("unimplemented!")),

        // obviously false:

        // A point cannot totally cover a segment, multiline, or polygon.
        (Obj2::Point(_), Obj2::Segment(_) | Obj2::Multiline(_) | Obj2::Polygon(_))
        // A segment cannot totally cover a polygon.
        | (Obj2::Segment(_), Obj2::Polygon(_))
        // A segment cannot totally cover a multiline.
        | (Obj2::Segment(_), Obj2::Multiline(_))
        // A multiline cannot totally cover a polygon.
        | (Obj2::Multiline(_), Obj2::Polygon(_)) => Ok(false),

        // need to evaluate:

        // p1 totally covers p2 i.f.f. it is the same.
        (Obj2::Point(p1), Obj2::Point(p2)) => Ok(point_overlaps_point(p1, p2)?.is_some()),

        // s totally covers p i.f.f. they overlap at all.
        (Obj2::Segment(s), Obj2::Point(p)) => Ok(segment_overlaps_point(s, p)?.is_some()),

        (Obj2::Segment(s1), Obj2::Segment(s2)) => {
            Ok(matches!(segment_overlaps_segment(s1, s2)?, Some((_, SegmentOp::Entire))))
        },

        (Obj2::Multiline(ml), Obj2::Point(p)) => {
            Ok(multiline_overlaps_point(ml, p)?.is_some())
        },
        (Obj2::Multiline(ml), Obj2::Segment(sg)) => {
            if let Some((_, sg_ops)) = multiline_overlaps_segment(ml, sg)? {
                Ok(sg_ops.head == SegmentOp::Entire && sg_ops.tail.is_empty())
            } else {
                Ok(false)
            }
        }
        (Obj2::Multiline(ml1), Obj2::Multiline(ml2)) => {
            if let Some((_, ml2_ops)) = multiline_overlaps_multiline(ml1, ml2)? {
                Ok(ml2_ops.head == MultilineOp::Entire && ml2_ops.tail.is_empty())
            } else {
                Ok(false)
            }
        }
        (Obj2::Polygon(pg), Obj2::Point(p)) => {
            Ok(polygon_overlaps_point(pg, p)?.is_some())
        }
        (Obj2::Polygon(pg), Obj2::Segment(sg)) => {
            if let Some((_, sg_ops)) = polygon_overlaps_segment(pg, sg)? {
                Ok(sg_ops.head == SegmentOp::Entire && sg_ops.tail.is_empty())
            } else {
                Ok(false)
            }
        }
        (Obj2::Polygon(_pg), Obj2::Multiline(_ml)) => {
            unimplemented!()
        }
        (Obj2::Polygon(_pg1), Obj2::Polygon(_pg2)) => {
            unimplemented!()
        }
    }
}

pub fn point_overlaps_point(a: &Point, b: &Point) -> Result<Option<Point>> {
    if a == b {
        Ok(Some(*a))
    } else {
        Ok(None)
    }
}

pub fn segment_overlaps_point(s: &Segment, p: &Point) -> Result<Option<(SegmentOp, Point)>> {
    if s.i == *p {
        Ok(Some((SegmentOp::Point(*p, Zero), *p)))
    } else if s.f == *p {
        Ok(Some((SegmentOp::Point(*p, One), *p)))
    } else if approx_eq!(
        f64,
        s.length(),
        Segment(s.i, *p).length() + Segment(*p, s.f).length()
    ) {
        Ok(Some((
            SegmentOp::Point(*p, interpolate_2d_checked(s.i, s.f, *p)?),
            *p,
        )))
    } else {
        Ok(None)
    }
}

pub fn segment_overlaps_segment(
    sa: &Segment,
    sb: &Segment,
) -> Result<Option<(SegmentOp, SegmentOp)>> {
    // NB: sa and sb are _not_ guaranteed to point the same way.
    if approx_eq!(f64, sa.slope(), sb.slope()) || approx_eq!(f64, sa.slope(), sb.flip().slope()) {
        let isxn_segment: Option<Segment> = match (
            segment_overlaps_point(sb, &sa.i)?,
            segment_overlaps_point(sb, &sa.f)?,
            segment_overlaps_point(sa, &sb.i)?,
            segment_overlaps_point(sa, &sb.f)?,
        ) {
            // No collision.
            (None, None, None, None) => None,

            // ERR: same
            (Some(_), Some(_), Some(_), Some(_)) => {
                return Ok(Some((SegmentOp::Entire, SegmentOp::Entire)));
            }

            // |-->|  //  |-->| //  |-->|
            // |--->| // |--->| // |---->|
            (Some(_), Some(_), _, _) => Some(*sa),

            // |---->| // |---->| // |--->|
            // |-->|   //  |-->|  //  |-->|
            (_, _, Some(_), Some(_)) => Some(*sb),

            //     |-->|
            // |-->|
            (
                Some((z1 @ SegmentOp::Point(_, One), _)),
                None,
                None,
                Some((z2 @ SegmentOp::Point(_, Zero), _)),
            ) => {
                return Ok(Some((z2, z1)));
            }

            //    |--->|
            // |--->|
            (Some(_), None, None, Some(_)) => Some(Segment(sa.i, sb.f)),

            // |-->|
            //     |-->|
            (
                None,
                Some((z1 @ SegmentOp::Point(_, Zero), _)),
                Some((z2 @ SegmentOp::Point(_, One), _)),
                None,
            ) => {
                return Ok(Some((z2, z1)));
            }

            // |--->|
            //    |--->|
            (None, Some(_), Some(_), None) => Some(Segment(sb.i, sa.f)),

            // |<--|
            //     |-->|
            (
                Some((z1 @ SegmentOp::Point(_, Zero), _)),
                None,
                Some((z2 @ SegmentOp::Point(_, Zero), _)),
                None,
            ) => {
                return Ok(Some((z1, z2)));
            }

            // |<---|
            //    |--->|
            (Some(_), None, Some(_), None) => Some(Segment(sa.i, sb.i)),

            // Tail-to-tail collision.
            //     |<--|
            // |-->|
            (
                None,
                Some((z1 @ SegmentOp::Point(_, One), _)),
                None,
                Some((z2 @ SegmentOp::Point(_, One), _)),
            ) => {
                return Ok(Some((z1, z2)));
            }

            //   |<--|
            // |-->|
            (None, Some(_), None, Some(_)) => Some(Segment(sa.f, sb.f)),

            _ => {
                return Err(anyhow!("this should not be possible."));
            }
        };

        if let Some(isxn_segment) = isxn_segment {
            if isxn_segment == *sa {
                return Ok(Some((
                    SegmentOp::Entire,
                    SegmentOp::Subsegment(isxn_segment),
                )));
            } else if isxn_segment == *sb {
                return Ok(Some((
                    SegmentOp::Subsegment(isxn_segment),
                    SegmentOp::Entire,
                )));
            } else {
                return Ok(Some((
                    SegmentOp::Subsegment(
                        // why dot/flip here? if the resultant segment doesn't
                        // run 'along' the input |sa|, |sb|, we have to flip it
                        // so that its subsegment is correctly oriented with
                        // respect to the input subsegment.
                        if isxn_segment.dot(sa) < 0.0 {
                            isxn_segment.flip()
                        } else {
                            isxn_segment
                        },
                    ),
                    SegmentOp::Subsegment(if isxn_segment.dot(sb) < 0.0 {
                        isxn_segment.flip()
                    } else {
                        isxn_segment
                    }),
                )));
            }
        }
    }

    let (p0_x, p0_y): (f64, f64) = sa.i.into();
    let (p1_x, p1_y): (f64, f64) = sa.f.into();
    let (p2_x, p2_y): (f64, f64) = sb.i.into();
    let (p3_x, p3_y): (f64, f64) = sb.f.into();

    let s1_x = p1_x - p0_x;
    let s1_y = p1_y - p0_y;
    let s2_x = p3_x - p2_x;
    let s2_y = p3_y - p2_y;

    let s = (-s1_y * (p0_x - p2_x) + s1_x * (p0_y - p2_y)) / (-s2_x * s1_y + s1_x * s2_y);
    let t = (s2_x * (p0_y - p2_y) - s2_y * (p0_x - p2_x)) / (-s2_x * s1_y + s1_x * s2_y);

    if (0_f64..=1_f64).contains(&s) && (0_f64..=1_f64).contains(&t) {
        let pt = Point(p0_x + (t * s1_x), p0_y + (t * s1_y));
        return Ok(Some((
            SegmentOp::Point(pt, interpolate_2d_checked(sa.i, sa.f, pt)?),
            SegmentOp::Point(pt, interpolate_2d_checked(sb.i, sb.f, pt)?),
        )));
    }

    Ok(None)
}

pub fn multiline_overlaps_point(
    ml: &Multiline,
    p: &Point,
) -> Result<Option<(NonEmpty<MultilineOp>, Point)>> {
    let mut ml_op_set = MultilineOpSet::new(ml);
    for (index, sg) in ml.to_segments().iter().enumerate() {
        if let Some((segment_opinion, _)) = segment_overlaps_point(sg, p)? {
            ml_op_set.add(MultilineOp::from_segment_opinion(index, segment_opinion))?;
        }
    }
    match ml_op_set.to_nonempty() {
        None => Ok(None),
        Some(u) => Ok(Some((u, *p))),
    }
}

pub fn multiline_overlaps_segment(
    ml: &Multiline,
    sg: &Segment,
) -> Result<Option<(NonEmpty<MultilineOp>, NonEmpty<SegmentOp>)>> {
    let mut ml_op_set = MultilineOpSet::new(/*original=*/ ml);
    let mut sg_op_set = SegmentOpSet::new(/*original=*/ sg);

    for (ml_sg_idx, ml_sg) in ml.to_segments().iter().enumerate() {
        if let Some((ml_sg_op, sg_op)) = segment_overlaps_segment(ml_sg, sg)? {
            ml_op_set.add(MultilineOp::from_segment_opinion(ml_sg_idx, ml_sg_op))?;
            sg_op_set.add(sg_op)?;
        }
    }

    if let (Some(ml_ops), Some(sg_ops)) = (ml_op_set.to_nonempty(), sg_op_set.to_nonempty()) {
        Ok(Some((ml_ops, sg_ops)))
    } else {
        Ok(None)
    }
}

pub fn multiline_overlaps_multiline(
    ml1: &Multiline,
    ml2: &Multiline,
) -> Result<Option<(NonEmpty<MultilineOp>, NonEmpty<MultilineOp>)>> {
    let mut ml1_op_set = MultilineOpSet::new(/*original=*/ ml1);
    let mut ml2_op_set = MultilineOpSet::new(/*original=*/ ml2);

    for (ml_sg1_idx, ml_sg1) in ml1.to_segments().iter().enumerate() {
        for (ml_sg2_idx, ml_sg2) in ml2.to_segments().iter().enumerate() {
            if let Some((ml_sg1_op, ml_sg2_op)) = segment_overlaps_segment(ml_sg1, ml_sg2)? {
                ml1_op_set.add(MultilineOp::from_segment_opinion(ml_sg1_idx, ml_sg1_op))?;
                ml2_op_set.add(MultilineOp::from_segment_opinion(ml_sg2_idx, ml_sg2_op))?;
            }
        }
    }

    if let (Some(ml1_ops), Some(ml2_ops)) = (ml1_op_set.to_nonempty(), ml2_op_set.to_nonempty()) {
        Ok(Some((ml1_ops, ml2_ops)))
    } else {
        Ok(None)
    }
}

pub fn polygon_overlaps_point(
    polygon: &Polygon,
    point: &Point,
) -> Result<Option<(PolygonOp, Point)>> {
    // PolygonOp::OnPoint special case.
    if let Some(idx) = polygon.pts.iter().position(|x| x == point) {
        return Ok(Some((PolygonOp::Point(idx, *point), *point)));
    }

    // PolygonOp::PointAlongEdge special case.
    for (index, pg_sg) in polygon.to_segments().iter().enumerate() {
        if let Some((SegmentOp::Point(at_point, percent_along), _)) =
            segment_overlaps_point(pg_sg, point)?
        {
            return Ok(Some((
                PolygonOp::EdgePoint(index, at_point, percent_along),
                *point,
            )));
        }
    }

    // PolygonOp::PointWithinArea or None.
    // https://en.wikipedia.org/wiki/Point_in_polygon#Winding_number_algorithm
    let theta: f64 = (polygon.pts.iter())
        .zip(polygon.pts.iter().cycle().skip(1))
        .map(|(i, j)| abp(point, i, j))
        .sum();

    if approx_eq!(f64, theta, 0_f64, epsilon = 0.00001) {
        Ok(None)
    } else {
        Ok(Some((PolygonOp::AreaPoint(*point), *point)))
    }
}

pub fn polygon_overlaps_segment(
    polygon: &Polygon,
    segment: &Segment,
) -> Result<Option<(NonEmpty<PolygonOp>, NonEmpty<SegmentOp>)>> {
    let mut pg_op_set = PolygonOpSet::new(/*original=*/ polygon);
    let mut sg_op_set = SegmentOpSet::new(/*original=*/ segment);
    for (pg_sg_idx, pg_sg) in polygon.to_segments().iter().enumerate() {
        if let Some((pg_sg_op, sg_op)) = segment_overlaps_segment(pg_sg, segment)? {
            pg_op_set.add(PolygonOp::from_segment_opinion(pg_sg_idx, pg_sg_op))?;
            sg_op_set.add(sg_op)?;
        }
    }

    // or we need to look at the subsegment(s) of our original segment (i.e.,
    // how it has been segmented by intersections) and for each segment, if it
    // is totally within the polygon, add it to sg_op_set. maybe even modify
    // pg_op_set here too.
    // ideally this also covers the special-case below, where there are no
    // intersections but the segment is totally contained within the polygon.

    {
        let cuts = sg_op_set.to_cuts()?;
        let segments: Vec<Segment> = (cuts.iter())
            .zip(cuts.iter().skip(1))
            .map(|((p1, _), (p2, _))| Segment(*p1, *p2))
            .collect();

        for s in segments {
            // if this segment actually goes through the polygon (i.e., if its
            // midpoint is within), then
            if polygon_overlaps_point(polygon, &s.midpoint())?.is_none() {
                continue;
            }

            // (a) add it to the segment ops set.
            sg_op_set.add(SegmentOp::Subsegment(s))?;

            // (b) add it to the polygon ops set.
            match (
                polygon_overlaps_point(polygon, &s.i)?,
                polygon_overlaps_point(polygon, &s.f)?,
            ) {
                // but there's a catch -- if these points are together along the same edge,
                (
                    Some((PolygonOp::EdgePoint(idx, _, _), _)),
                    Some((PolygonOp::EdgePoint(jdx, _, _), _)),
                ) if idx == jdx => {
                    // then we need to add the type PolygonOp::SubsegmentOfEdge instead.
                    pg_op_set.add(PolygonOp::EdgeSubsegment(idx, s))?;
                }
                _ => {
                    pg_op_set.add(PolygonOp::AreaSegment(s))?;
                }
            }
        }
    }

    if let (Some(pg_ops), Some(sg_ops)) = (pg_op_set.to_nonempty(), sg_op_set.to_nonempty()) {
        Ok(Some((pg_ops, sg_ops)))
    } else {
        Ok(None)
    }
}

pub fn polygon_overlaps_multiline(
    polygon: &Polygon,
    multiline: &Multiline,
) -> Result<Option<(NonEmpty<PolygonOp>, NonEmpty<MultilineOp>)>> {
    let mut pg_op_set = PolygonOpSet::new(/*original=*/ polygon);
    let mut ml_op_set = MultilineOpSet::new(/*original=*/ multiline);
    for (ml_sg_idx, ml_sg) in multiline.to_segments().iter().enumerate() {
        if let Some((pg_ops, sg_ops)) = polygon_overlaps_segment(polygon, ml_sg)? {
            for pg_op in pg_ops.into_iter() {
                pg_op_set.add(pg_op)?;
            }
            for sg_op in sg_ops.into_iter() {
                ml_op_set.add(MultilineOp::from_segment_opinion(ml_sg_idx, sg_op))?;
            }
        }
    }

    if let (Some(pg_ops), Some(ml_ops)) = (pg_op_set.to_nonempty(), ml_op_set.to_nonempty()) {
        Ok(Some((pg_ops, ml_ops)))
    } else {
        Ok(None)
    }
}

pub fn polygon_overlaps_polygon(
    pg1: &Polygon,
    pg2: &Polygon,
) -> Result<Option<(NonEmpty<PolygonOp>, NonEmpty<PolygonOp>)>> {
    if pg1 == pg2 {
        return Ok(Some((
            nonempty![PolygonOp::Entire],
            nonempty![PolygonOp::Entire],
        )));
    }

    let mut pg1_ops_set = PolygonOpSet::new(/*original=*/ pg1);
    let mut pg2_ops_set = PolygonOpSet::new(/*original=*/ pg2);

    let ml1: Multiline = (*pg1).clone().into();
    let ml2: Multiline = (*pg2).clone().into();

    if let Some((ml1_ops, ml2_ops)) = multiline_overlaps_multiline(&ml1, &ml2)? {
        for ml1_op in ml1_ops.into_iter() {
            pg1_ops_set.add(PolygonOp::from_multiline_opinion(ml1_op))?;
        }
        for ml2_op in ml2_ops.into_iter() {
            pg2_ops_set.add(PolygonOp::from_multiline_opinion(ml2_op))?;
        }
    }

    if let (Some(pg1_ops), Some(pg2_ops)) = (pg1_ops_set.to_nonempty(), pg2_ops_set.to_nonempty()) {
        Ok(Some((pg1_ops, pg2_ops)))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::Percent::Val;
    use lazy_static::lazy_static;
    use nonempty::nonempty as ne;
    use pretty_assertions::assert_eq as pretty_assert_eq;
    use test_case::{test_case, test_matrix};

    //           ^ (y)
    //           |
    //   a . b . c . d . e
    //           |
    //   f . g . h . i . j
    //           |
    // <-k---l---m---n---o-> (x)
    //           |
    //   p . q . r . s . t
    //           |
    //   u . v . w . x . y
    //           |
    //           v
    lazy_static! {
        static ref A: Point = Point(-2, 2);
        static ref B: Point = Point(-1, 2);
        static ref C: Point = Point(0, 2);
        static ref D: Point = Point(1, 2);
        static ref E: Point = Point(2, 2);
        static ref F: Point = Point(-2, 1);
        static ref G: Point = Point(-1, 1);
        static ref H: Point = Point(0, 1);
        static ref I: Point = Point(1, 1);
        static ref J: Point = Point(2, 1);
        static ref K: Point = Point(-2, 0);
        static ref L: Point = Point(-1, 0);
        static ref M: Point = Point(0, 0);
        static ref N: Point = Point(1, 0);
        static ref O: Point = Point(2, 0);
        static ref P: Point = Point(-2, -1);
        static ref Q: Point = Point(-1, -1);
        static ref R: Point = Point(0, -1);
        static ref S: Point = Point(1, -1);
        static ref T: Point = Point(2, -1);
        static ref U: Point = Point(-2, -2);
        static ref V: Point = Point(-1, -2);
        static ref W: Point = Point(0, -2);
        static ref X: Point = Point(1, -2);
        static ref Y: Point = Point(2, -2);
    }

    #[test_case(*C, *C, Some(*C))]
    #[test_case(*D, *D, Some(*D))]
    #[test_case(*D, *H, None)]
    #[test_case(*A, *B, None)]
    fn test_point_overlaps_point(a: Point, b: Point, expectation: Option<Point>) -> Result<()> {
        pretty_assert_eq!(point_overlaps_point(&a, &b)?, expectation);
        Ok(())
    }

    #[test_case((*C, *D), *C, Some((SegmentOp::Point(*C, Zero), *C)); "at start 00")]
    #[test_case((*C, *D), *D, Some((SegmentOp::Point(*D, One), *D)); "at end 00")]
    #[test_case((*C, *I), *C, Some((SegmentOp::Point(*C, Zero), *C)); "at start 01")]
    #[test_case((*C, *I), *I, Some((SegmentOp::Point(*I, One), *I)); "at end 01")]
    #[test_case((*C, *E), *D, Some((SegmentOp::Point(*D, Val(0.5)), *D)); "halfway along 01")]
    #[test_case((*C, *O), *I, Some((SegmentOp::Point(*I, Val(0.5)), *I)); "halfway along 02")]
    #[test_case((*C, *W), *M, Some((SegmentOp::Point(*M, Val(0.5)), *M)); "halfway along 03")]
    fn test_segment_overlaps_point(
        segment: impl Into<Segment>,
        point: Point,
        expectation: Option<(SegmentOp, Point)>,
    ) -> Result<()> {
        pretty_assert_eq!(
            segment_overlaps_point(&segment.into(), &point)?,
            expectation
        );
        Ok(())
    }

    #[test_case((*C, *D), (*C, *D), Some((SegmentOp::Entire, SegmentOp::Entire)); "same 00")]
    #[test_case((*C, *M), (*C, *M), Some((SegmentOp::Entire, SegmentOp::Entire)); "same 01")]
    #[test_case((*C, *M), (*M, *C), Some((SegmentOp::Entire, SegmentOp::Entire)); "same reverse 00")]
    #[test_case((*Y, *M), (*M, *Y), Some((SegmentOp::Entire, SegmentOp::Entire)); "same reverse 01")]
    #[test_case((*B, *E), (*C, *D), Some((SegmentOp::Subsegment(Segment(*C, *D)), SegmentOp::Entire)); "total collision")]
    #[test_case((*B, *E), (*D, *C), Some((SegmentOp::Subsegment(Segment(*D, *C)), SegmentOp::Entire)); "total collision, flip")]
    #[test_case((*B, *D), (*C, *E), Some((SegmentOp::Subsegment(Segment(*C, *D)), SegmentOp::Subsegment(Segment(*C, *D)))); "partial collision")]
    #[test_case((*B, *D), (*E, *C), Some((SegmentOp::Subsegment(Segment(*C, *D)), SegmentOp::Subsegment(Segment(*D, *C)))); "partial collision, flip")]
    #[test_case((*A, *C), (*C, *E), Some((SegmentOp::Point(*C, One), SegmentOp::Point(*C, Zero))); "at point end to start")]
    #[test_case((*A, *C), (*E, *C), Some((SegmentOp::Point(*C, One), SegmentOp::Point(*C, One))); "at point end to end")]
    #[test_case((*C, *A), (*E, *C), Some((SegmentOp::Point(*C, Zero), SegmentOp::Point(*C, One))); "at point head to end")]
    #[test_case((*C, *A), (*C, *E), Some((SegmentOp::Point(*C, Zero), SegmentOp::Point(*C, Zero))); "at point head to head")]
    #[test_case((*A, *E), (*A, *C), Some((SegmentOp::Subsegment(Segment(*A, *C)), SegmentOp::Entire)); "subsegment 00")]
    #[test_case((*A, *C), (*A, *E), Some((SegmentOp::Entire, SegmentOp::Subsegment(Segment(*A, *C)))); "subsegment 00, flip")]
    #[test_case((*A, *E), (*B, *D), Some((SegmentOp::Subsegment(Segment(*B, *D)), SegmentOp::Entire)); "subsegment 01")]
    #[test_case((*A, *E), (*C, *E), Some((SegmentOp::Subsegment(Segment(*C, *E)), SegmentOp::Entire)); "subsegment 02")]
    #[test_case((*C, *O), (*E, *M), Some((SegmentOp::Point(*I, Val(0.5)), SegmentOp::Point(*I, Val(0.5)))); "crosshairs 00")]
    #[test_case((*O, *C), (*M, *E), Some((SegmentOp::Point(*I, Val(0.5)), SegmentOp::Point(*I, Val(0.5)))); "crosshairs 01")]
    #[test_case((*C, *O), (*M, *E), Some((SegmentOp::Point(*I, Val(0.5)), SegmentOp::Point(*I, Val(0.5)))); "crosshairs 02")]
    #[test_case((*O, *C), (*E, *M), Some((SegmentOp::Point(*I, Val(0.5)), SegmentOp::Point(*I, Val(0.5)))); "crosshairs 03")]
    fn test_segment_overlaps_segment(
        a: impl Into<Segment>,
        b: impl Into<Segment>,
        expectation: Option<(SegmentOp, SegmentOp)>,
    ) -> Result<()> {
        let a = a.into();
        let b = b.into();
        if let Some((o1, o2)) = expectation {
            pretty_assert_eq!(segment_overlaps_segment(&a, &b)?, Some((o1, o2)));
            pretty_assert_eq!(segment_overlaps_segment(&b, &a)?, Some((o2, o1)));
        } else {
            pretty_assert_eq!(segment_overlaps_segment(&a, &b)?, expectation);
        }
        Ok(())
    }

    #[test_case(Multiline([*A, *C, *E, *O, *Y]), *A, Some((ne![MultilineOp::Point(0, *A)], *A)); "multiline point at index 0")]
    #[test_case(Multiline([*A, *C, *E, *O, *Y]), *E, Some((ne![MultilineOp::Point(2, *E)], *E)); "multiline point at index 2")]
    #[test_case(Multiline([*A, *C, *E, *O, *Y]), *Y, Some((ne![MultilineOp::Point(4, *Y)], *Y)); "multiline point at index 4")]
    #[test_case(Multiline([*A, *C, *E, *O, *Y]), *B, Some((ne![MultilineOp::SegmentPoint(0, *B, Val(0.5))], *B)); "multiline point along segment 0")]
    #[test_case(Multiline([*A, *C, *E, *O, *Y]), *D, Some((ne![MultilineOp::SegmentPoint(1, *D, Val(0.5))], *D)); "multiline point along segment 1")]
    #[test_case(Multiline([*A, *C, *E, *O, *Y]), *J, Some((ne![MultilineOp::SegmentPoint(2, *J, Val(0.5))], *J)); "multiline point along segment 2")]
    #[test_case(Multiline([*A, *C, *E, *O, *Y]), *T, Some((ne![MultilineOp::SegmentPoint(3, *T, Val(0.5))], *T)); "multiline point along segment 3")]
    #[test_case(Multiline([*A, *C, *E, *O, *Y]), *M, None; "unrelated")]
    fn test_multiline_overlaps_point(
        multiline: Multiline,
        point: Point,
        expectation: Option<(NonEmpty<MultilineOp>, Point)>,
    ) -> Result<()> {
        pretty_assert_eq!(multiline_overlaps_point(&multiline, &point)?, expectation);
        Ok(())
    }

    #[test_case(&[*C, *E, *O], (*M, *N), None; "none 00")]
    #[test_case(&[*C, *E, *J], (*M, *N), None; "none 01")]
    #[test_case(&[*C, *E, *O], (*H, *N), None; "none 02")]
    #[test_case(&[*C, *I, *O], (*D, *J), None; "none 03")]
    #[test_case(&[*C, *E, *O], (*C, *M), Some((ne![MultilineOp::Point(0, *C)], ne![SegmentOp::Point(*C, Zero)])); "at point at point 00")]
    #[test_case(&[*E, *O, *M], (*E, *C), Some((ne![MultilineOp::Point(0, *E)], ne![SegmentOp::Point(*E, Zero)])); "at point at point 01")]
    #[test_case(&[*O, *M, *C], (*O, *E), Some((ne![MultilineOp::Point(0, *O)], ne![SegmentOp::Point(*O, Zero)])); "at point at point 02")]
    #[test_case(&[*C, *I, *O], (*C, *M), Some((ne![MultilineOp::Point(0, *C)], ne![SegmentOp::Point(*C, Zero)])); "at point at point 03")]
    #[test_case(&[*C, *E, *O], (*M, *C), Some((ne![MultilineOp::Point(0, *C)], ne![SegmentOp::Point(*C, One)])); "at point at point 04")]
    #[test_case(&[*E, *O, *M], (*C, *E), Some((ne![MultilineOp::Point(0, *E)], ne![SegmentOp::Point(*E, One)])); "at point at point 05")]
    #[test_case(&[*O, *M, *C], (*E, *O), Some((ne![MultilineOp::Point(0, *O)], ne![SegmentOp::Point(*O, One)])); "at point at point 06")]
    #[test_case(&[*C, *I, *O], (*M, *C), Some((ne![MultilineOp::Point(0, *C)], ne![SegmentOp::Point(*C, One)])); "at point at point 07")]
    #[test_case(&[*D, *I, *N], (*C, *E), Some((ne![MultilineOp::Point(0, *D)], ne![SegmentOp::Point(*D, Val(0.5))])); "at point at point 08")]
    #[test_case(&[*H, *I, *J], (*M, *C), Some((ne![MultilineOp::Point(0, *H)], ne![SegmentOp::Point(*H, Val(0.5))])); "at point at point 09")]
    #[test_case(&[*D, *I, *N], (*I, *J), Some((ne![MultilineOp::Point(1, *I)], ne![SegmentOp::Point(*I, Zero)])); "at point at point 10")]
    #[test_case(&[*D, *I, *N], (*H, *I), Some((ne![MultilineOp::Point(1, *I)], ne![SegmentOp::Point(*I, One)])); "at point at point 11")]
    #[test_case(&[*D, *I, *N], (*H, *J), Some((ne![MultilineOp::Point(1, *I)], ne![SegmentOp::Point(*I, Val(0.5))])); "at point at point 12")]
    #[test_case(&[*D, *I, *N], (*N, *O), Some((ne![MultilineOp::Point(2, *N)], ne![SegmentOp::Point(*N, Zero)])); "at point at point 13")]
    #[test_case(&[*D, *I, *N], (*M, *N), Some((ne![MultilineOp::Point(2, *N)], ne![SegmentOp::Point(*N, One)])); "at point at point 14")]
    #[test_case(&[*D, *I, *N], (*M, *O), Some((ne![MultilineOp::Point(2, *N)], ne![SegmentOp::Point(*N, Val(0.5))])); "at point at point 15")]
    #[test_case(&[*C, *E, *O], (*C, *O), Some((ne![MultilineOp::Point(0, *C), MultilineOp::Point(2, *O) ], ne![ SegmentOp::Point(*C, Zero), SegmentOp::Point(*O, One)])); "segment bookends 1")]
    #[test_case(&[*C, *E, *O], (*D, *J), Some((ne![MultilineOp::SegmentPoint(0, *D, Val(0.5)), MultilineOp::SegmentPoint(1, *J, Val(0.5)) ], ne![ SegmentOp::Point(*D, Zero), SegmentOp::Point(*J, One)])); "segment bookends 2")]
    #[test_case(&[*C, *D, *E], (*C, *D), Some((ne![MultilineOp::Segment(0)], ne![SegmentOp::Entire])); "partial collision")]
    #[test_case(&[*C, *D, *E], (*D, *C), Some((ne![MultilineOp::Segment(0)], ne![SegmentOp::Entire])); "partial collision 02")]
    #[test_case(&[*C, *D, *E], (*D, *E), Some((ne![MultilineOp::Segment(1)], ne![SegmentOp::Entire])); "partial collision 03")]
    #[test_case(&[*C, *D, *E], (*E, *D), Some((ne![MultilineOp::Segment(1)], ne![SegmentOp::Entire])); "partial collision 04")]
    #[test_case(&[*C, *D, *E], (*C, *E), Some((ne![MultilineOp::Entire], ne![ SegmentOp::Entire ])); "total collision 01")]
    #[test_case(&[*C, *D, *E], (*E, *C), Some((ne![MultilineOp::Entire], ne![ SegmentOp::Entire ])); "total collision 01 flip")]
    #[test_case(&[*C, *D, *E], (Point(0.5,2), Point(1.5,2)), Some(( ne![ MultilineOp::Subsegment(0, Segment(Point(0.5,2),Point(1,2))), MultilineOp::Subsegment(1, Segment(Point(1,2), Point(1.5,2))) ], ne![SegmentOp::Entire])); "total collision half shift 01")]
    #[test_case(&[*C, *D, *E], (Point(1.5,2), Point(0.5,2)), Some(( ne![ MultilineOp::Subsegment(0, Segment(Point(0.5,2),Point(1,2))), MultilineOp::Subsegment(1, Segment(Point(1,2), Point(1.5,2))) ], ne![SegmentOp::Entire])); "total collision half shift 01 flip")]
    #[test_case(&[*H, *J, *O], (*D, *N), Some((ne![MultilineOp::SegmentPoint(0, *I, Val(0.5))], ne![SegmentOp::Point(*I, Val(0.5))])); "at point on segment at point on segment 00")]
    #[test_case(&[*H, *J, *O], (*I, *N), Some((ne![MultilineOp::SegmentPoint(0, *I, Val(0.5))], ne![SegmentOp::Point(*I, Zero)])); "at point on segment at point on segment 01")]
    #[test_case(&[*H, *J, *O], (*D, *I), Some((ne![MultilineOp::SegmentPoint(0, *I, Val(0.5))], ne![SegmentOp::Point(*I, One)])); "at point on segment at point on segment 02")]
    #[test_case(&[*M, *H, *J], (*D, *N), Some((ne![MultilineOp::SegmentPoint(1, *I, Val(0.5))], ne![SegmentOp::Point(*I, Val(0.5))])); "at point on segment at point on segment 03")]
    #[test_case(&[*M, *H, *J], (*I, *N), Some((ne![MultilineOp::SegmentPoint(1, *I, Val(0.5))], ne![SegmentOp::Point(*I, Zero)])); "at point on segment at point on segment 04")]
    #[test_case(&[*M, *H, *J], (*D, *I), Some((ne![MultilineOp::SegmentPoint(1, *I, Val(0.5))], ne![SegmentOp::Point(*I, One)])); "at point on segment at point on segment 05")]
    fn test_multiline_overlaps_segment(
        ml: &[Point],
        sg: impl Into<Segment>,
        expectation: Option<(NonEmpty<MultilineOp>, NonEmpty<SegmentOp>)>,
    ) -> Result<()> {
        let ml: Multiline = ml.try_into()?;
        pretty_assert_eq!(multiline_overlaps_segment(&ml, &sg.into())?, expectation);
        Ok(())
    }

    #[test_case(Multiline([*C, *D, *E]), Multiline([*H, *I, *J]) => None; "none 01")]
    #[test_case(Multiline([*C, *D, *E]), Multiline([*M, *N, *O]) => None; "none 02")]
    #[test_case(Multiline([*C, *I, *O]), Multiline([*D, *J]) => None; "none diagonal")]
    #[test_case(Multiline([*C, *D, *E]), Multiline([*C, *H, *M]) => Some((ne![MultilineOp::Point(0, *C)], ne![MultilineOp::Point(0, *C)])); "AtPoint 0, AtPoint 0")]
    #[test_case(Multiline([*C, *D, *E]), Multiline([*M, *H, *C]) => Some((ne![MultilineOp::Point(0, *C)], ne![MultilineOp::Point(2, *C)])); "AtPoint 0, AtPoint 2")]
    #[test_case(Multiline([*E, *D, *C]), Multiline([*M, *H, *C]) => Some((ne![MultilineOp::Point(2, *C)], ne![MultilineOp::Point(2, *C)])); "AtPoint 2, AtPoint 2")]
    #[test_case(Multiline([*C, *I, *O]), Multiline([*M, *I, *E]) => Some((ne![MultilineOp::Point(1, *I)], ne![MultilineOp::Point(1, *I)])); "AtPoint 1, AtPoint 1")]
    #[test_case(Multiline([*C, *O]), Multiline([*E, *M]) => Some((ne![MultilineOp::SegmentPoint(0, *I, Val(0.5))], ne![MultilineOp::SegmentPoint(0, *I, Val(0.5))])); "crosshairs")]
    #[test_case(Multiline([*C, *D, *E]), Multiline([*C, *D, *I]) => Some((ne![MultilineOp::Segment(0)], ne![MultilineOp::Segment(0)])); "partial collision, entire subsegment 0 0")]
    #[test_case(Multiline([*E, *D, *C]), Multiline([*I, *D, *C]) => Some((ne![MultilineOp::Segment(1)], ne![MultilineOp::Segment(1)])); "partial collision, entire subsegment 1 1")]
    #[test_case(Multiline([*C, *D, *E]), Multiline([*D, *E, *J]) => Some((ne![MultilineOp::Segment(1)], ne![MultilineOp::Segment(0)])); "partial collision, entire subsegment 1 0")]
    #[test_case(Multiline([*C, *D, *E]), Multiline([*E, *D, *C]) => Some((ne![MultilineOp::Entire], ne![MultilineOp::Entire])); "partial collision, entire subsegment 01 01 flipped")]
    #[test_case(Multiline([*C, *D, *E, *J, *O]), Multiline([*C, *D, *I, *J, *O]) => Some((ne![MultilineOp::Segment(0), MultilineOp::Segment(3)], ne![MultilineOp::Segment(0), MultilineOp::Segment(3)])); "shared segment, then diversion, then another shared segment")]
    #[test_case(Multiline([*C, *D, *E, *J, *O]), Multiline([*C, *D, *I, *J]) => Some((ne![MultilineOp::Point(3, *J), MultilineOp::Segment(0)], ne![MultilineOp::Point(3, *J), MultilineOp::Segment(0)])); "shared segment, then diversion, then atpoint")]
    #[test_case( Multiline([*A, *B, *C]), Multiline([*A, *C, *E]) => Some(( ne![ MultilineOp::Entire, ], ne![ MultilineOp::Segment(0), ])))]
    fn test_multiline_overlaps_multiline(
        ml1: Multiline,
        ml2: Multiline,
    ) -> Option<(NonEmpty<MultilineOp>, NonEmpty<MultilineOp>)> {
        multiline_overlaps_multiline(&ml1, &ml2).unwrap()
    }

    #[test_case(Polygon([*D, *H, *N, *J]), &C => None; "point not in polygon 00")]
    #[test_case(Polygon([*D, *H, *N, *J]), &E => None; "point not in polygon 01")]
    #[test_case(Polygon([*D, *H, *N, *J]), &I => Some((PolygonOp::AreaPoint(*I), *I)); "point in polygon")]
    #[test_case(Polygon([*D, *H, *N, *J]), &D => Some((PolygonOp::Point(0, *D), *D)); "point at point of polygon 00")]
    #[test_case(Polygon([*D, *H, *N, *J]), &H => Some((PolygonOp::Point(1, *H), *H)); "point at point of polygon 01")]
    #[test_case(Polygon([*D, *H, *N, *J]), &N => Some((PolygonOp::Point(2, *N), *N)); "point at point of polygon 02")]
    #[test_case(Polygon([*D, *H, *N, *J]), &J => Some((PolygonOp::Point(3, *J), *J)); "point at point of polygon 03")]
    #[test_case(Polygon([*C, *M, *O, *E]), &H => Some((PolygonOp::EdgePoint(0, *H, Val(0.5)), *H)); "point at edge of polygon 00")]
    #[test_case(Polygon([*C, *M, *O, *E]), &N => Some((PolygonOp::EdgePoint(1, *N, Val(0.5)), *N)); "point at edge of polygon 01")]
    #[test_case(Polygon([*C, *M, *O, *E]), &J => Some((PolygonOp::EdgePoint(2, *J, Val(0.5)), *J)); "point at edge of polygon 02")]
    #[test_case(Polygon([*C, *M, *O, *E]), &D => Some((PolygonOp::EdgePoint(3, *D, Val(0.5)), *D)); "point at edge of polygon 03")]
    fn test_polygon_overlaps_point(pg: Result<Polygon>, pt: &Point) -> Option<(PolygonOp, Point)> {
        polygon_overlaps_point(&pg.unwrap(), pt).unwrap()
    }

    // segment begins outside and ends outside and does not pass through
    #[test_matrix([Polygon([*G, *Q, *S, *I])], [(*A, *E), (*E, *A), (*B, *F), (*T, *X), (*O, *J)] => None)]
    // segment begins outside and ends outside and does pass through at a point
    #[test_matrix([Polygon([*G, *Q, *S, *I])], [(*C, *K), (*K, *C)] => Some((ne![PolygonOp::Point(0, *G)], ne![SegmentOp::Point(*G, Val(0.5))])))]
    // segment begins outside and ends outside and does pass through at two points
    #[test_case(Polygon([*I, *M, *G, *K, *O]), (*J, *F) => Some((ne![ PolygonOp::Point(0, *I), PolygonOp::Point(2, *G)], ne![SegmentOp::Point(*G, Val(0.75)), SegmentOp::Point(*I, Val(0.25))])))]
    // segment begins outside and ends outside and does pass through along two edges
    #[test_case(Polygon([*T, *N, *M, *H, *B, *F, *X]), (*T, *B) => Some((ne![PolygonOp::Edge(0), PolygonOp::Edge(3)], ne![SegmentOp::Subsegment(Segment(*H, *B)), SegmentOp::Subsegment(Segment(*T, *N))])))]
    // segment begins outside and ends outside and does pass through along an edge
    #[test_case(Polygon([*I, *G, *K, *O]), (*F, *J) => Some((ne![PolygonOp::Edge(0)], ne![SegmentOp::Subsegment(Segment(*I, *G))])))]
    #[test_case(Polygon([*I, *G, *K, *O]), (*I, *G) => Some((ne![PolygonOp::Edge(0)], ne![SegmentOp::Entire])))]
    // segment begins outside and ends outside and does pass through along two edges
    #[test_case(Polygon([*I, *H, *M, *L, *G, *F, *P, *S]), (*I, *F) => Some((ne![ PolygonOp::Edge(0), PolygonOp::Edge(4) ], ne![ SegmentOp::Subsegment(Segment(*G, *F)), SegmentOp::Subsegment(Segment(*I, *H)) ])))]
    // segment begins outside and ends at point
    #[test_matrix([Polygon([*G, *Q, *S, *I])], [(*A, *G), (*B, *G), (*F, *G)] => Some((ne![PolygonOp::Point(0, *G)], ne![SegmentOp::Point(*G, One)])))]
    #[test_matrix([Polygon([*G, *Q, *S, *I])], [(*D, *I), (*E, *I), (*J, *I)] => Some((ne![PolygonOp::Point(3, *I)], ne![SegmentOp::Point(*I, One)])))]
    #[test_matrix([Polygon([*G, *Q, *S, *I])], [(*U, *Q), (*P, *Q), (*V, *Q)] => Some((ne![PolygonOp::Point(1, *Q)], ne![SegmentOp::Point(*Q, One)])))]
    // segment begins outside and ends on an edge
    #[test_matrix([Polygon([*I, *G, *Q, *S])], [(*C, *H), (*B, *H), (*D, *H)] => Some((ne![PolygonOp::EdgePoint(0, *H, Val(0.5))], ne![SegmentOp::Point(*H, One)])))]
    // segment begins outside and ends inside
    #[test_case(Polygon([*I, *G, *Q, *S]), (*C, *M) => Some((ne![PolygonOp::AreaSegment(Segment(*H, *M))], ne![SegmentOp::Subsegment(Segment(*H, *M))])))]
    #[test_case(Polygon([*I, *G, *Q, *S]), (*E, *M) => Some((ne![PolygonOp::AreaSegment(Segment(*I, *M))], ne![SegmentOp::Subsegment(Segment(*I, *M))])))]
    // segment begins at a point and ends outside
    #[test_case(Polygon([*I, *G, *Q, *S]), (*I, *J) => Some((ne![PolygonOp::Point(0, *I)], ne![SegmentOp::Point(*I, Zero)])))]
    // segment begins at a point and ends outside and passes totally through the polygon
    #[test_case(Polygon([*I, *G, *Q, *S]), (*I, *U) => Some((ne![PolygonOp::AreaSegment(Segment(*I, *Q))], ne![SegmentOp::Subsegment(Segment(*I, *Q))])))]
    // segment begins at a point and ends at a point
    #[test_case(Polygon([*I, *G, *Q, *S]), (*I, *G) => Some((ne![PolygonOp::Edge(0)], ne![SegmentOp::Entire])))]
    // segment begins at a point and ends on an edge
    #[test_case(Polygon([*I, *G, *Q, *S]), (*I, *H) => Some((ne![PolygonOp::EdgeSubsegment(0, Segment(*I, *H))], ne![SegmentOp::Entire])))]
    // segment begins at a point and ends inside
    #[test_case(Polygon([*I, *G, *Q, *S]), (*I, *M) => Some((ne![PolygonOp::AreaSegment(Segment(*I, *M))], ne![SegmentOp::Entire])))]
    // segment begins on an edge and ends outside
    #[test_case(Polygon([*I, *G, *Q, *S]), (*N, *O) => Some((ne![PolygonOp::EdgePoint(3, *N, Val(0.5))], ne![SegmentOp::Point(*N, Zero)])))]
    // segment begins on an edge and ends at a point
    #[test_case(Polygon([*I, *G, *Q, *S]), (*N, *I) => Some((ne![PolygonOp::EdgeSubsegment(3, Segment(*N, *I))], ne![SegmentOp::Entire])))]
    // segment begins on an edge and ends on an edge
    #[test_case(Polygon([*I, *G, *Q, *S]), (*N, *H) => Some((ne![PolygonOp::AreaSegment(Segment(*N, *H))], ne![SegmentOp::Entire])))]
    // segment begins on an edge and ends inside
    #[test_case(Polygon([*I, *G, *Q, *S]), (*N, *M) => Some((ne![PolygonOp::AreaSegment(Segment(*N, *M))], ne![SegmentOp::Entire])))]
    // segment begins inside and ends outside
    #[test_case(Polygon([*I, *G, *Q, *S]), (*M, *O) => Some((ne![PolygonOp::AreaSegment(Segment(*M, *N))], ne![SegmentOp::Subsegment(Segment(*M, *N))])))]
    // segment begins inside and ends at a point
    #[test_case(Polygon([*I, *G, *Q, *S]), (*M, *I) => Some((ne![PolygonOp::AreaSegment(Segment(*M, *I))], ne![SegmentOp::Entire])))]
    // segment begins inside and ends on an edge
    #[test_case(Polygon([*I, *G, *Q, *S]), (*M, *N) => Some((ne![PolygonOp::AreaSegment(Segment(*M, *N))], ne![SegmentOp::Entire])))]
    // segment begins inside and ends inside
    #[test_case(Polygon([*A, *U, *Y, *E]), (*G, *I) => Some((ne![PolygonOp::AreaSegment(Segment(*G, *I))], ne![SegmentOp::Entire])))]
    fn test_polygon_overlaps_segment(
        pg: Result<Polygon>,
        sg: impl Into<Segment>,
    ) -> Option<(NonEmpty<PolygonOp>, NonEmpty<SegmentOp>)> {
        polygon_overlaps_segment(&pg.unwrap(), &sg.into()).unwrap()
    }

    // yeah.... 4^3==64 test cases. that's life in the big city

    // multiline begins outside, pivots outside, and ends outside. no intersections
    #[test_case( Polygon([*I, *G, *Q, *S]), Multiline([*A, *C, *E]) => None)]
    // multiline begins outside, pivots outside, and ends outside. intersects along edge of polygon
    #[test_case( Polygon([*I, *G, *Q, *S]), Multiline([*A, *F, *J]) => Some(( ne![PolygonOp::Edge(0)], ne![MultilineOp::Subsegment(1, Segment(*I, *G))])))]
    // multiline begins outside, pivots outside, and ends outside. intersects through one point of polygon
    #[test_case( Polygon([*I, *G, *Q, *S]), Multiline([*A, *C, *K]) => Some(( ne![PolygonOp::Point(1, *G)], ne![MultilineOp::SegmentPoint(1, *G, Val(0.5))])))]
    // multiline begins outside, pivots outside, and ends outside. intersects through two points of polygon
    #[test_case( Polygon([*I, *G, *Q, *S]), Multiline([*E, *A, *Y]) => Some(( ne![PolygonOp::AreaSegment(Segment(*G, *S))], ne![MultilineOp::Subsegment(1, Segment(*G, *S))])))]
    // multiline begins outside, pivots outside, and ends on a point. no intersections
    #[test_case( Polygon([*I, *G, *Q, *S]), Multiline([*A, *C, *I]) => Some(( ne![PolygonOp::Point(0, *I)], ne![MultilineOp::Point(2, *I)])))]
    // multiline begins outside, pivots outside, and ends on a point. intersects through an edge
    #[test_case( Polygon([*I, *G, *Q, *S]), Multiline([*A, *F, *I]) => Some(( ne![PolygonOp::Edge(0)], ne![MultilineOp::Subsegment(1, Segment(*I, *G))])))]
    // multiline begins outside, pivots outside, and ends on an edge. no other intersections
    #[test_case( Polygon([*I, *G, *Q, *S]), Multiline([*A, *F, *H]) => Some(( ne![PolygonOp::EdgeSubsegment(0, Segment(*H, *G))], ne![MultilineOp::Subsegment(1, Segment(*G, *H))])))]
    // multiline begins outside, pivots outside, and ends on an edge. intersects along an edge
    #[test_case( Polygon([*I, *G, *Q, *S]), Multiline([*A, *C, *H]) => Some(( ne![PolygonOp::EdgePoint(0, *H, Val(0.5))], ne![MultilineOp::Point(2, *H)])))]
    // multiline begins outside, pivots outside, and ends inside.
    #[test_case( Polygon([*I, *G, *Q, *S]), Multiline([*A, *C, *M]) => Some(( ne![PolygonOp::AreaSegment(Segment(*H, *M))], ne![MultilineOp::Subsegment(1, Segment(*H, *M))])))]
    #[test_case( Polygon([*I, *G, *Q, *S]), Multiline([*C, *A, *M]) => Some(( ne![PolygonOp::AreaSegment(Segment(*G, *M))], ne![MultilineOp::Subsegment(1, Segment(*G, *M))])))]
    // multiline begins outside, pivots on a point, and ends outside. no other intersections
    #[test_case( Polygon([*I, *G, *Q, *S]), Multiline([*B, *G, *F]) => Some(( ne![PolygonOp::Point(1, *G)], ne![MultilineOp::Point(1, *G)],)))]
    // multiline begins outside, pivots on a point, and ends outside. intersects along an edge
    #[test_case( Polygon([*I, *G, *Q, *S]), Multiline([*D, *I, *F]) => Some(( ne![PolygonOp::Edge(0)], ne![MultilineOp::Subsegment(1, Segment(*I, *G))])))]
    #[test_case( Polygon([*I, *G, *Q, *S]), Multiline([*B, *G, *J]) => Some(( ne![PolygonOp::Edge(0)], ne![MultilineOp::Subsegment(1, Segment(*I, *G))])))]
    // multiline begins outside, pivots on a point, and ends on a point.
    #[test_case( Polygon([*I, *G, *Q, *S]), Multiline([*B, *G, *I]) => Some(( ne![PolygonOp::Edge(0)], ne![MultilineOp::Segment(1)],)))]
    #[test_case( Polygon([*I, *G, *Q, *S]), Multiline([*B, *G, *S]) => Some(( ne![PolygonOp::AreaSegment(Segment(*G, *S))], ne![MultilineOp::Segment(1)],)))]
    // multiline begins outside, pivots on a point, and ends on an edge. no other intersections
    #[test_case( Polygon([*I, *G, *Q, *S]), Multiline([*B, *G, *H]) => Some(( ne![PolygonOp::EdgeSubsegment(0, Segment(*G, *H))], ne![MultilineOp::Segment(1)],)))]
    // multiline begins outside, pivots on a point, and ends on an edge. different segments
    #[test_case( Polygon([*I, *G, *Q, *S]), Multiline([*B, *G, *N]) => Some(( ne![PolygonOp::AreaSegment(Segment(*G, *N))], ne![MultilineOp::Segment(1)],)))]
    // multiline begins outside, pivots on a point, and ends inside.
    #[test_case( Polygon([*I, *G, *Q, *S]), Multiline([*C, *I, *M]) => Some(( ne![PolygonOp::AreaSegment(Segment(*I, *M))], ne![MultilineOp::Segment(1)])))]
    // multiline begins outside, pivots on an edge, and ends outside. no other intersections
    #[test_case( Polygon([*I, *G, *Q, *S]), Multiline([*B, *H, *D]) => Some(( ne![PolygonOp::EdgePoint(0, *H, Val(0.5))], ne![MultilineOp::Point(1, *H)])))]
    // multiline begins outside, pivots on an edge, and ends outside. one segment intersection
    #[test_case( Polygon([*I, *G, *Q, *S]), Multiline([*B, *H, *P]) => Some(( ne![PolygonOp::AreaSegment(Segment(*H, *L))], ne![MultilineOp::Subsegment(1, Segment(*H, *L))])))]
    // multiline begins outside, pivots on an edge, and ends on a point. no overlaps
    #[test_case( Polygon([*I, *G, *Q, *S]), Multiline([*B, *H, *I]) => Some(( ne![PolygonOp::EdgeSubsegment(0, Segment(*H, *I))], ne![MultilineOp::Segment(1)])))]
    // multiline begins outside, pivots on an edge, and ends on a point. one segment overlap
    #[test_case( Polygon([*I, *G, *Q, *S]), Multiline([*B, *H, *S]) => Some(( ne![PolygonOp::AreaSegment(Segment(*H, *S))], ne![MultilineOp::Segment(1)])))]
    // TODO(ambuc): maybe fill in the rest of these test cases. so far we have built quite a lot of confidence though.
    // multiline begins outside, pivots on an edge, and ends on an edge.
    // multiline begins outside, pivots on an edge, and ends inside.
    // multiline begins outside, pivots inside, and ends outside.
    // multiline begins outside, pivots inside, and ends on a point.
    // multiline begins outside, pivots inside, and ends on an edge.
    // multiline begins outside, pivots inside, and ends inside.

    // multiline begins on a point, pivots outside, and ends outside.
    // multiline begins on a point, pivots outside, and ends on a point.
    // multiline begins on a point, pivots outside, and ends on an edge.
    // multiline begins on a point, pivots outside, and ends inside.
    // multiline begins on a point, pivots on a point, and ends outside.
    // multiline begins on a point, pivots on a point, and ends on a point.
    // multiline begins on a point, pivots on a point, and ends on an edge.
    // multiline begins on a point, pivots on a point, and ends inside.
    // multiline begins on a point, pivots on an edge, and ends outside.
    // multiline begins on a point, pivots on an edge, and ends on a point.
    // multiline begins on a point, pivots on an edge, and ends on an edge.
    // multiline begins on a point, pivots on an edge, and ends inside.
    // multiline begins on a point, pivots inside, and ends outside.
    // multiline begins on a point, pivots inside, and ends on a point.
    // multiline begins on a point, pivots inside, and ends on an edge.
    // multiline begins on a point, pivots inside, and ends inside.

    // multiline begins on an edge, pivots outside, and ends outside.
    // multiline begins on an edge, pivots outside, and ends on a point.
    // multiline begins on an edge, pivots outside, and ends on an edge.
    // multiline begins on an edge, pivots outside, and ends inside.
    // multiline begins on an edge, pivots on a point, and ends outside.
    // multiline begins on an edge, pivots on a point, and ends on a point.
    // multiline begins on an edge, pivots on a point, and ends on an edge.
    // multiline begins on an edge, pivots on a point, and ends inside.
    // multiline begins on an edge, pivots on an edge, and ends outside.
    // multiline begins on an edge, pivots on an edge, and ends on a point.
    // multiline begins on an edge, pivots on an edge, and ends on an edge.
    // multiline begins on an edge, pivots on an edge, and ends inside.
    // multiline begins on an edge, pivots inside, and ends outside.
    // multiline begins on an edge, pivots inside, and ends on a point.
    // multiline begins on an edge, pivots inside, and ends on an edge.
    // multiline begins on an edge, pivots inside, and ends inside.

    // multiline begins inside, pivots outside, and ends outside.
    // multiline begins inside, pivots outside, and ends on a point.
    // multiline begins inside, pivots outside, and ends on an edge.
    // multiline begins inside, pivots outside, and ends inside.
    // multiline begins inside, pivots on a point, and ends outside.
    // multiline begins inside, pivots on a point, and ends on a point.
    // multiline begins inside, pivots on a point, and ends on an edge.
    // multiline begins inside, pivots on a point, and ends inside.
    // multiline begins inside, pivots on an edge, and ends outside.
    // multiline begins inside, pivots on an edge, and ends on a point.
    // multiline begins inside, pivots on an edge, and ends on an edge.
    // multiline begins inside, pivots on an edge, and ends inside.
    // multiline begins inside, pivots inside, and ends outside.
    // multiline begins inside, pivots inside, and ends on a point.
    // multiline begins inside, pivots inside, and ends on an edge.
    // multiline begins inside, pivots inside, and ends inside.

    // plus more test cases around intersecting a point twice, or an edge twice,
    // or something like that.

    // plus more test cases around 'doubling back', if multilines can do that kind of thing.

    // plus a test case for MutlilineOp::Entire.

    fn test_polygon_overlaps_multiline(
        pg: Result<Polygon>,
        ml: Multiline,
    ) -> Option<(NonEmpty<PolygonOp>, NonEmpty<MultilineOp>)> {
        polygon_overlaps_multiline(&pg.unwrap(), &ml).unwrap()
    }

    //           ^ (y)
    //           |
    //   a . b . c . d . e
    //           |
    //   f . g . h . i . j
    //           |
    // <-k---l---m---n---o-> (x)
    //           |
    //   p . q . r . s . t
    //           |
    //   u . v . w . x . y
    //           |
    //           v

    #[test_case(Polygon([*B, *A, *F, *G]), Polygon([*D, *C, *H, *I]), None; "no overlap")]
    #[test_case(Polygon([*B, *A, *F, *G]), Polygon([*B, *A, *F, *G]), Some((ne![PolygonOp::Entire], ne![PolygonOp::Entire])); "entire overlap")]
    fn test_polygon_overlaps_polygon(
        pg1: Result<Polygon>,
        pg2: Result<Polygon>,
        expectation: Option<(NonEmpty<PolygonOp>, NonEmpty<PolygonOp>)>,
    ) -> Result<()> {
        pretty_assert_eq!(polygon_overlaps_polygon(&pg1?, &pg2?)?, expectation);
        Ok(())
    }
}
