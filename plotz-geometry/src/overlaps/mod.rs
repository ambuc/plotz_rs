#![allow(missing_docs)]

pub mod opinion;
use self::opinion::*;
use crate::{
    interpolate::interpolate_2d_checked,
    obj2::Obj2,
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
use nonempty::NonEmpty;

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
            Ok(matches!(segment_overlaps_segment(s1, s2)?, Some((_, SegmentOp::EntireSegment))))
        },

        (Obj2::Multiline(ml), Obj2::Point(p)) => {
            Ok(multiline_overlaps_point(ml, p)?.is_some())
        },
        (Obj2::Multiline(ml), Obj2::Segment(sg)) => {
            if let Some((_, sg_ops)) = multiline_overlaps_segment(ml, sg)? {
                Ok(sg_ops.head == SegmentOp::EntireSegment && sg_ops.tail.is_empty())
            } else {
                Ok(false)
            }
        }
        (Obj2::Multiline(ml1), Obj2::Multiline(ml2)) => {
            if let Some((_, ml2_ops)) = multiline_overlaps_multiline(ml1, ml2)? {
                Ok(ml2_ops.head == MultilineOp::EntireMultiline && ml2_ops.tail.is_empty())
            } else {
                Ok(false)
            }
        }
        (Obj2::Polygon(pg), Obj2::Point(p)) => {
            Ok(polygon_overlaps_point(pg, p)?.is_some())
        }
        (Obj2::Polygon(pg), Obj2::Segment(sg)) => {
            if let Some((_, sg_ops)) = polygon_overlaps_segment(pg, sg)? {
                Ok(sg_ops.head == SegmentOp::EntireSegment && sg_ops.tail.is_empty())
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
        Ok(Some((SegmentOp::PointAlongSegment(*p, Zero), *p)))
    } else if s.f == *p {
        Ok(Some((SegmentOp::PointAlongSegment(*p, One), *p)))
    } else if approx_eq!(
        f64,
        s.length(),
        Segment(s.i, *p).length() + Segment(*p, s.f).length()
    ) {
        Ok(Some((
            SegmentOp::PointAlongSegment(*p, interpolate_2d_checked(s.i, s.f, *p)?),
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
                return Ok(Some((SegmentOp::EntireSegment, SegmentOp::EntireSegment)));
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
                Some((z1 @ SegmentOp::PointAlongSegment(_, One), _)),
                None,
                None,
                Some((z2 @ SegmentOp::PointAlongSegment(_, Zero), _)),
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
                Some((z1 @ SegmentOp::PointAlongSegment(_, Zero), _)),
                Some((z2 @ SegmentOp::PointAlongSegment(_, One), _)),
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
                Some((z1 @ SegmentOp::PointAlongSegment(_, Zero), _)),
                None,
                Some((z2 @ SegmentOp::PointAlongSegment(_, Zero), _)),
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
                Some((z1 @ SegmentOp::PointAlongSegment(_, One), _)),
                None,
                Some((z2 @ SegmentOp::PointAlongSegment(_, One), _)),
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
                    SegmentOp::EntireSegment,
                    SegmentOp::Subsegment(isxn_segment),
                )));
            } else if isxn_segment == *sb {
                return Ok(Some((
                    SegmentOp::Subsegment(isxn_segment),
                    SegmentOp::EntireSegment,
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
            SegmentOp::PointAlongSegment(pt, interpolate_2d_checked(sa.i, sa.f, pt)?),
            SegmentOp::PointAlongSegment(pt, interpolate_2d_checked(sb.i, sb.f, pt)?),
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
        return Ok(Some((PolygonOp::OnPoint(idx, *point), *point)));
    }

    // PolygonOp::PointAlongEdge special case.
    for (index, pg_sg) in polygon.to_segments().iter().enumerate() {
        if let Some((SegmentOp::PointAlongSegment(at_point, percent_along), _)) =
            segment_overlaps_point(pg_sg, point)?
        {
            return Ok(Some((
                PolygonOp::PointAlongEdge(index, at_point, percent_along),
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
        Ok(Some((PolygonOp::PointWithinArea(*point), *point)))
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
                    Some((PolygonOp::PointAlongEdge(idx, _, _), _)),
                    Some((PolygonOp::PointAlongEdge(jdx, _, _), _)),
                ) if idx == jdx => {
                    // then we need to add the type PolygonOp::SubsegmentOfEdge instead.
                    pg_op_set.add(PolygonOp::SubsegmentOfEdge(idx, s))?;
                }
                _ => {
                    pg_op_set.add(PolygonOp::SegmentWithinArea(s))?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::Percent::Val;
    use lazy_static::lazy_static;
    use nonempty::nonempty as ne;
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

    #[test_case(*C, *C => Some(*C))]
    #[test_case(*D, *D => Some(*D))]
    #[test_case(*D, *H => None)]
    #[test_case(*A, *B => None)]
    fn test_point_overlaps_point(a: Point, b: Point) -> Option<Point> {
        point_overlaps_point(&a, &b).unwrap()
    }

    #[test_case((*C, *D), *C => Some((SegmentOp::PointAlongSegment(*C, Zero), *C)); "at start 00")]
    #[test_case((*C, *D), *D => Some((SegmentOp::PointAlongSegment(*D, One), *D)); "at end 00")]
    #[test_case((*C, *I), *C => Some((SegmentOp::PointAlongSegment(*C, Zero), *C)); "at start 01")]
    #[test_case((*C, *I), *I => Some((SegmentOp::PointAlongSegment(*I, One), *I)); "at end 01")]
    #[test_case((*C, *E), *D => Some((SegmentOp::PointAlongSegment(*D, Val(0.5)), *D)); "halfway along 01")]
    #[test_case((*C, *O), *I => Some((SegmentOp::PointAlongSegment(*I, Val(0.5)), *I)); "halfway along 02")]
    #[test_case((*C, *W), *M => Some((SegmentOp::PointAlongSegment(*M, Val(0.5)), *M)); "halfway along 03")]
    fn test_segment_overlaps_point(
        segment: impl Into<Segment>,
        point: Point,
    ) -> Option<(SegmentOp, Point)> {
        segment_overlaps_point(&segment.into(), &point).unwrap()
    }

    #[test_case((*C, *D), (*C, *D), Some((SegmentOp::EntireSegment, SegmentOp::EntireSegment)); "same 00")]
    #[test_case((*C, *M), (*C, *M), Some((SegmentOp::EntireSegment, SegmentOp::EntireSegment)); "same 01")]
    #[test_case((*C, *M), (*M, *C), Some((SegmentOp::EntireSegment, SegmentOp::EntireSegment)); "same reverse 00")]
    #[test_case((*Y, *M), (*M, *Y), Some((SegmentOp::EntireSegment, SegmentOp::EntireSegment)); "same reverse 01")]
    #[test_case((*B, *E), (*C, *D), Some((SegmentOp::Subsegment(Segment(*C, *D)), SegmentOp::EntireSegment)); "total collision")]
    #[test_case((*B, *E), (*D, *C), Some((SegmentOp::Subsegment(Segment(*D, *C)), SegmentOp::EntireSegment)); "total collision, flip")]
    #[test_case((*B, *D), (*C, *E), Some((SegmentOp::Subsegment(Segment(*C, *D)), SegmentOp::Subsegment(Segment(*C, *D)))); "partial collision")]
    #[test_case((*B, *D), (*E, *C), Some((SegmentOp::Subsegment(Segment(*C, *D)), SegmentOp::Subsegment(Segment(*D, *C)))); "partial collision, flip")]
    #[test_case((*A, *C), (*C, *E), Some((SegmentOp::PointAlongSegment(*C, One), SegmentOp::PointAlongSegment(*C, Zero))); "at point end to start")]
    #[test_case((*A, *C), (*E, *C), Some((SegmentOp::PointAlongSegment(*C, One), SegmentOp::PointAlongSegment(*C, One))); "at point end to end")]
    #[test_case((*C, *A), (*E, *C), Some((SegmentOp::PointAlongSegment(*C, Zero), SegmentOp::PointAlongSegment(*C, One))); "at point head to end")]
    #[test_case((*C, *A), (*C, *E), Some((SegmentOp::PointAlongSegment(*C, Zero), SegmentOp::PointAlongSegment(*C, Zero))); "at point head to head")]
    #[test_case((*A, *E), (*A, *C), Some((SegmentOp::Subsegment(Segment(*A, *C)), SegmentOp::EntireSegment)); "subsegment 00")]
    #[test_case((*A, *C), (*A, *E), Some((SegmentOp::EntireSegment, SegmentOp::Subsegment(Segment(*A, *C)))); "subsegment 00, flip")]
    #[test_case((*A, *E), (*B, *D), Some((SegmentOp::Subsegment(Segment(*B, *D)), SegmentOp::EntireSegment)); "subsegment 01")]
    #[test_case((*A, *E), (*C, *E), Some((SegmentOp::Subsegment(Segment(*C, *E)), SegmentOp::EntireSegment)); "subsegment 02")]
    #[test_case((*C, *O), (*E, *M), Some((SegmentOp::PointAlongSegment(*I, Val(0.5)), SegmentOp::PointAlongSegment(*I, Val(0.5)))); "crosshairs 00")]
    #[test_case((*O, *C), (*M, *E), Some((SegmentOp::PointAlongSegment(*I, Val(0.5)), SegmentOp::PointAlongSegment(*I, Val(0.5)))); "crosshairs 01")]
    #[test_case((*C, *O), (*M, *E), Some((SegmentOp::PointAlongSegment(*I, Val(0.5)), SegmentOp::PointAlongSegment(*I, Val(0.5)))); "crosshairs 02")]
    #[test_case((*O, *C), (*E, *M), Some((SegmentOp::PointAlongSegment(*I, Val(0.5)), SegmentOp::PointAlongSegment(*I, Val(0.5)))); "crosshairs 03")]
    fn test_segment_overlaps_segment(
        a: impl Into<Segment>,
        b: impl Into<Segment>,
        expectation: Option<(SegmentOp, SegmentOp)>,
    ) -> Result<()> {
        let a = a.into();
        let b = b.into();
        if let Some((o1, o2)) = expectation {
            assert_eq!(segment_overlaps_segment(&a, &b)?, Some((o1, o2)));
            assert_eq!(segment_overlaps_segment(&b, &a)?, Some((o2, o1)));
        } else {
            assert_eq!(segment_overlaps_segment(&a, &b)?, expectation);
        }
        Ok(())
    }

    #[test_case(Multiline([*A, *C, *E, *O, *Y]), *A => Some((ne![MultilineOp::Point(0, *A)], *A)); "multiline point at index 0")]
    #[test_case(Multiline([*A, *C, *E, *O, *Y]), *E => Some((ne![MultilineOp::Point(2, *E)], *E)); "multiline point at index 2")]
    #[test_case(Multiline([*A, *C, *E, *O, *Y]), *Y => Some((ne![MultilineOp::Point(4, *Y)], *Y)); "multiline point at index 4")]
    #[test_case(Multiline([*A, *C, *E, *O, *Y]), *B => Some((ne![MultilineOp::PointAlongSegmentOf(0, *B, Val(0.5))], *B)); "multiline point along segment 0")]
    #[test_case(Multiline([*A, *C, *E, *O, *Y]), *D => Some((ne![MultilineOp::PointAlongSegmentOf(1, *D, Val(0.5))], *D)); "multiline point along segment 1")]
    #[test_case(Multiline([*A, *C, *E, *O, *Y]), *J => Some((ne![MultilineOp::PointAlongSegmentOf(2, *J, Val(0.5))], *J)); "multiline point along segment 2")]
    #[test_case(Multiline([*A, *C, *E, *O, *Y]), *T => Some((ne![MultilineOp::PointAlongSegmentOf(3, *T, Val(0.5))], *T)); "multiline point along segment 3")]
    #[test_case(Multiline([*A, *C, *E, *O, *Y]), *M => None; "unrelated")]
    fn test_multiline_overlaps_point(
        multiline: Multiline,
        point: Point,
    ) -> Option<(NonEmpty<MultilineOp>, Point)> {
        multiline_overlaps_point(&multiline, &point).unwrap()
    }

    #[test_case(Multiline([*C, *E, *O]), (*M, *N) => None; "none 00")]
    #[test_case(Multiline([*C, *E, *J]), (*M, *N) => None; "none 01")]
    #[test_case(Multiline([*C, *E, *O]), (*H, *N) => None; "none 02")]
    #[test_case(Multiline([*C, *I, *O]), (*D, *J) => None; "none 03")]
    #[test_case(Multiline([*C, *E, *O]), (*C, *M) => Some((ne![MultilineOp::Point(0, *C)], ne![SegmentOp::PointAlongSegment(*C, Zero)])); "at point at point 00")]
    #[test_case(Multiline([*E, *O, *M]), (*E, *C) => Some((ne![MultilineOp::Point(0, *E)], ne![SegmentOp::PointAlongSegment(*E, Zero)])); "at point at point 01")]
    #[test_case(Multiline([*O, *M, *C]), (*O, *E) => Some((ne![MultilineOp::Point(0, *O)], ne![SegmentOp::PointAlongSegment(*O, Zero)])); "at point at point 02")]
    #[test_case(Multiline([*C, *I, *O]), (*C, *M) => Some((ne![MultilineOp::Point(0, *C)], ne![SegmentOp::PointAlongSegment(*C, Zero)])); "at point at point 03")]
    #[test_case(Multiline([*C, *E, *O]), (*M, *C) => Some((ne![MultilineOp::Point(0, *C)], ne![SegmentOp::PointAlongSegment(*C, One)])); "at point at point 04")]
    #[test_case(Multiline([*E, *O, *M]), (*C, *E) => Some((ne![MultilineOp::Point(0, *E)], ne![SegmentOp::PointAlongSegment(*E, One)])); "at point at point 05")]
    #[test_case(Multiline([*O, *M, *C]), (*E, *O) => Some((ne![MultilineOp::Point(0, *O)], ne![SegmentOp::PointAlongSegment(*O, One)])); "at point at point 06")]
    #[test_case(Multiline([*C, *I, *O]), (*M, *C) => Some((ne![MultilineOp::Point(0, *C)], ne![SegmentOp::PointAlongSegment(*C, One)])); "at point at point 07")]
    #[test_case(Multiline([*D, *I, *N]), (*C, *E) => Some((ne![MultilineOp::Point(0, *D)], ne![SegmentOp::PointAlongSegment(*D, Val(0.5))])); "at point at point 08")]
    #[test_case(Multiline([*H, *I, *J]), (*M, *C) => Some((ne![MultilineOp::Point(0, *H)], ne![SegmentOp::PointAlongSegment(*H, Val(0.5))])); "at point at point 09")]
    #[test_case(Multiline([*D, *I, *N]), (*I, *J) => Some((ne![MultilineOp::Point(1, *I)], ne![SegmentOp::PointAlongSegment(*I, Zero)])); "at point at point 10")]
    #[test_case(Multiline([*D, *I, *N]), (*H, *I) => Some((ne![MultilineOp::Point(1, *I)], ne![SegmentOp::PointAlongSegment(*I, One)])); "at point at point 11")]
    #[test_case(Multiline([*D, *I, *N]), (*H, *J) => Some((ne![MultilineOp::Point(1, *I)], ne![SegmentOp::PointAlongSegment(*I, Val(0.5))])); "at point at point 12")]
    #[test_case(Multiline([*D, *I, *N]), (*N, *O) => Some((ne![MultilineOp::Point(2, *N)], ne![SegmentOp::PointAlongSegment(*N, Zero)])); "at point at point 13")]
    #[test_case(Multiline([*D, *I, *N]), (*M, *N) => Some((ne![MultilineOp::Point(2, *N)], ne![SegmentOp::PointAlongSegment(*N, One)])); "at point at point 14")]
    #[test_case(Multiline([*D, *I, *N]), (*M, *O) => Some((ne![MultilineOp::Point(2, *N)], ne![SegmentOp::PointAlongSegment(*N, Val(0.5))])); "at point at point 15")]
    #[test_case(Multiline([*C, *E, *O]), (*C, *O) => Some((ne![MultilineOp::Point(0, *C), MultilineOp::Point(2, *O) ], ne![ SegmentOp::PointAlongSegment(*C, Zero), SegmentOp::PointAlongSegment(*O, One)])); "segment bookends 1")]
    #[test_case(Multiline([*C, *E, *O]), (*D, *J) => Some((ne![MultilineOp::PointAlongSegmentOf(0, *D, Val(0.5)), MultilineOp::PointAlongSegmentOf(1, *J, Val(0.5)) ], ne![ SegmentOp::PointAlongSegment(*D, Zero), SegmentOp::PointAlongSegment(*J, One)])); "segment bookends 2")]
    #[test_case(Multiline([*C, *D, *E]), (*C, *D) => Some((ne![MultilineOp::EntireSubsegment(0)], ne![SegmentOp::EntireSegment])); "partial collision")]
    #[test_case(Multiline([*C, *D, *E]), (*D, *C) => Some((ne![MultilineOp::EntireSubsegment(0)], ne![SegmentOp::EntireSegment])); "partial collision 02")]
    #[test_case(Multiline([*C, *D, *E]), (*D, *E) => Some((ne![MultilineOp::EntireSubsegment(1)], ne![SegmentOp::EntireSegment])); "partial collision 03")]
    #[test_case(Multiline([*C, *D, *E]), (*E, *D) => Some((ne![MultilineOp::EntireSubsegment(1)], ne![SegmentOp::EntireSegment])); "partial collision 04")]
    #[test_case(Multiline([*C, *D, *E]), (*C, *E) => Some((ne![MultilineOp::EntireMultiline], ne![ SegmentOp::EntireSegment ])); "total collision 01")]
    #[test_case(Multiline([*C, *D, *E]), (*E, *C) => Some((ne![MultilineOp::EntireMultiline], ne![ SegmentOp::EntireSegment ])); "total collision 01 flip")]
    #[test_case(Multiline([*C, *D, *E]), (Point(0.5,2), Point(1.5,2)) => Some(( ne![ MultilineOp::SubsegmentOf(0, Segment(Point(0.5,2),Point(1,2))), MultilineOp::SubsegmentOf(1, Segment(Point(1,2), Point(1.5,2))) ], ne![SegmentOp::EntireSegment])); "total collision half shift 01")]
    #[test_case(Multiline([*C, *D, *E]), (Point(1.5,2), Point(0.5,2)) => Some(( ne![ MultilineOp::SubsegmentOf(0, Segment(Point(0.5,2),Point(1,2))), MultilineOp::SubsegmentOf(1, Segment(Point(1,2), Point(1.5,2))) ], ne![SegmentOp::EntireSegment])); "total collision half shift 01 flip")]
    #[test_case(Multiline([*H, *J, *O]), (*D, *N) => Some((ne![MultilineOp::PointAlongSegmentOf(0, *I, Val(0.5))], ne![SegmentOp::PointAlongSegment(*I, Val(0.5))])); "at point on segment at point on segment 00")]
    #[test_case(Multiline([*H, *J, *O]), (*I, *N) => Some((ne![MultilineOp::PointAlongSegmentOf(0, *I, Val(0.5))], ne![SegmentOp::PointAlongSegment(*I, Zero)])); "at point on segment at point on segment 01")]
    #[test_case(Multiline([*H, *J, *O]), (*D, *I) => Some((ne![MultilineOp::PointAlongSegmentOf(0, *I, Val(0.5))], ne![SegmentOp::PointAlongSegment(*I, One)])); "at point on segment at point on segment 02")]
    #[test_case(Multiline([*M, *H, *J]), (*D, *N) => Some((ne![MultilineOp::PointAlongSegmentOf(1, *I, Val(0.5))], ne![SegmentOp::PointAlongSegment(*I, Val(0.5))])); "at point on segment at point on segment 03")]
    #[test_case(Multiline([*M, *H, *J]), (*I, *N) => Some((ne![MultilineOp::PointAlongSegmentOf(1, *I, Val(0.5))], ne![SegmentOp::PointAlongSegment(*I, Zero)])); "at point on segment at point on segment 04")]
    #[test_case(Multiline([*M, *H, *J]), (*D, *I) => Some((ne![MultilineOp::PointAlongSegmentOf(1, *I, Val(0.5))], ne![SegmentOp::PointAlongSegment(*I, One)])); "at point on segment at point on segment 05")]
    fn test_multiline_overlaps_segment(
        ml: Multiline,
        sg: impl Into<Segment>,
    ) -> Option<(NonEmpty<MultilineOp>, NonEmpty<SegmentOp>)> {
        let sg = sg.into();
        multiline_overlaps_segment(&ml, &sg).unwrap()
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
    #[test_case(Multiline([*C, *D, *E]), Multiline([*H, *I, *J]) => None; "none 01")]
    #[test_case(Multiline([*C, *D, *E]), Multiline([*M, *N, *O]) => None; "none 02")]
    #[test_case(Multiline([*C, *I, *O]), Multiline([*D, *J]) => None; "none diagonal")]
    #[test_case(Multiline([*C, *D, *E]), Multiline([*C, *H, *M]) => Some((ne![MultilineOp::Point(0, *C)], ne![MultilineOp::Point(0, *C)])); "AtPoint 0, AtPoint 0")]
    #[test_case(Multiline([*C, *D, *E]), Multiline([*M, *H, *C]) => Some((ne![MultilineOp::Point(0, *C)], ne![MultilineOp::Point(2, *C)])); "AtPoint 0, AtPoint 2")]
    #[test_case(Multiline([*E, *D, *C]), Multiline([*M, *H, *C]) => Some((ne![MultilineOp::Point(2, *C)], ne![MultilineOp::Point(2, *C)])); "AtPoint 2, AtPoint 2")]
    #[test_case(Multiline([*C, *I, *O]), Multiline([*M, *I, *E]) => Some((ne![MultilineOp::Point(1, *I)], ne![MultilineOp::Point(1, *I)])); "AtPoint 1, AtPoint 1")]
    #[test_case(Multiline([*C, *O]), Multiline([*E, *M]) => Some((ne![MultilineOp::PointAlongSegmentOf(0, *I, Val(0.5))], ne![MultilineOp::PointAlongSegmentOf(0, *I, Val(0.5))])); "crosshairs")]
    #[test_case(Multiline([*C, *D, *E]), Multiline([*C, *D, *I]) => Some((ne![MultilineOp::EntireSubsegment(0)], ne![MultilineOp::EntireSubsegment(0)])); "partial collision, entire subsegment 0 0")]
    #[test_case(Multiline([*E, *D, *C]), Multiline([*I, *D, *C]) => Some((ne![MultilineOp::EntireSubsegment(1)], ne![MultilineOp::EntireSubsegment(1)])); "partial collision, entire subsegment 1 1")]
    #[test_case(Multiline([*C, *D, *E]), Multiline([*D, *E, *J]) => Some((ne![MultilineOp::EntireSubsegment(1)], ne![MultilineOp::EntireSubsegment(0)])); "partial collision, entire subsegment 1 0")]
    #[test_case(Multiline([*C, *D, *E]), Multiline([*E, *D, *C]) => Some((ne![MultilineOp::EntireMultiline], ne![MultilineOp::EntireMultiline])); "partial collision, entire subsegment 01 01 flipped")]
    #[test_case(Multiline([*C, *D, *E, *J, *O]), Multiline([*C, *D, *I, *J, *O]) => Some((ne![MultilineOp::EntireSubsegment(0), MultilineOp::EntireSubsegment(3)], ne![MultilineOp::EntireSubsegment(0), MultilineOp::EntireSubsegment(3)])); "shared segment, then diversion, then another shared segment")]
    #[test_case(Multiline([*C, *D, *E, *J, *O]), Multiline([*C, *D, *I, *J]) => Some((ne![MultilineOp::Point(3, *J), MultilineOp::EntireSubsegment(0)], ne![MultilineOp::Point(3, *J), MultilineOp::EntireSubsegment(0)])); "shared segment, then diversion, then atpoint")]
    #[test_case( Multiline([*A, *B, *C]), Multiline([*A, *C, *E]) => Some(( ne![ MultilineOp::EntireMultiline, ], ne![ MultilineOp::EntireSubsegment(0), ])))]
    fn test_multiline_overlaps_multiline(
        ml1: Multiline,
        ml2: Multiline,
    ) -> Option<(NonEmpty<MultilineOp>, NonEmpty<MultilineOp>)> {
        multiline_overlaps_multiline(&ml1, &ml2).unwrap()
    }

    #[test_case(Polygon([*D, *H, *N, *J]), &C => None; "point not in polygon 00")]
    #[test_case(Polygon([*D, *H, *N, *J]), &E => None; "point not in polygon 01")]
    #[test_case(Polygon([*D, *H, *N, *J]), &I => Some((PolygonOp::PointWithinArea(*I), *I)); "point in polygon")]
    #[test_case(Polygon([*D, *H, *N, *J]), &D => Some((PolygonOp::OnPoint(0, *D), *D)); "point at point of polygon 00")]
    #[test_case(Polygon([*D, *H, *N, *J]), &H => Some((PolygonOp::OnPoint(1, *H), *H)); "point at point of polygon 01")]
    #[test_case(Polygon([*D, *H, *N, *J]), &N => Some((PolygonOp::OnPoint(2, *N), *N)); "point at point of polygon 02")]
    #[test_case(Polygon([*D, *H, *N, *J]), &J => Some((PolygonOp::OnPoint(3, *J), *J)); "point at point of polygon 03")]
    #[test_case(Polygon([*C, *M, *O, *E]), &H => Some((PolygonOp::PointAlongEdge(0, *H, Val(0.5)), *H)); "point at edge of polygon 00")]
    #[test_case(Polygon([*C, *M, *O, *E]), &N => Some((PolygonOp::PointAlongEdge(1, *N, Val(0.5)), *N)); "point at edge of polygon 01")]
    #[test_case(Polygon([*C, *M, *O, *E]), &J => Some((PolygonOp::PointAlongEdge(2, *J, Val(0.5)), *J)); "point at edge of polygon 02")]
    #[test_case(Polygon([*C, *M, *O, *E]), &D => Some((PolygonOp::PointAlongEdge(3, *D, Val(0.5)), *D)); "point at edge of polygon 03")]
    fn test_polygon_overlaps_point(pg: Result<Polygon>, pt: &Point) -> Option<(PolygonOp, Point)> {
        polygon_overlaps_point(&pg.unwrap(), pt).unwrap()
    }

    // segment begins outside and ends outside and does not pass through
    #[test_matrix([Polygon([*G, *Q, *S, *I])], [(*A, *E), (*E, *A), (*B, *F), (*T, *X), (*O, *J)] => None)]
    // segment begins outside and ends outside and does pass through at a point
    #[test_matrix([Polygon([*G, *Q, *S, *I])], [(*C, *K), (*K, *C)] => Some((ne![PolygonOp::OnPoint(0, *G)], ne![SegmentOp::PointAlongSegment(*G, Val(0.5))])))]
    // segment begins outside and ends outside and does pass through at two points
    #[test_case(Polygon([*I, *M, *G, *K, *O]), (*J, *F) => Some((ne![ PolygonOp::OnPoint(0, *I), PolygonOp::OnPoint(2, *G)], ne![SegmentOp::PointAlongSegment(*G, Val(0.75)), SegmentOp::PointAlongSegment(*I, Val(0.25))])))]
    // segment begins outside and ends outside and does pass through along two edges
    #[test_case(Polygon([*T, *N, *M, *H, *B, *F, *X]), (*T, *B) => Some((ne![PolygonOp::EntireEdge(0), PolygonOp::EntireEdge(3)], ne![SegmentOp::Subsegment(Segment(*H, *B)), SegmentOp::Subsegment(Segment(*T, *N))])))]
    // segment begins outside and ends outside and does pass through along an edge
    #[test_case(Polygon([*I, *G, *K, *O]), (*F, *J) => Some((ne![PolygonOp::EntireEdge(0)], ne![SegmentOp::Subsegment(Segment(*I, *G))])))]
    #[test_case(Polygon([*I, *G, *K, *O]), (*I, *G) => Some((ne![PolygonOp::EntireEdge(0)], ne![SegmentOp::EntireSegment])))]
    // segment begins outside and ends outside and does pass through along two edges
    #[test_case(Polygon([*I, *H, *M, *L, *G, *F, *P, *S]), (*I, *F) => Some((ne![ PolygonOp::EntireEdge(0), PolygonOp::EntireEdge(4) ], ne![ SegmentOp::Subsegment(Segment(*G, *F)), SegmentOp::Subsegment(Segment(*I, *H)) ])))]
    // segment begins outside and ends at point
    #[test_matrix([Polygon([*G, *Q, *S, *I])], [(*A, *G), (*B, *G), (*F, *G)] => Some((ne![PolygonOp::OnPoint(0, *G)], ne![SegmentOp::PointAlongSegment(*G, One)])))]
    #[test_matrix([Polygon([*G, *Q, *S, *I])], [(*D, *I), (*E, *I), (*J, *I)] => Some((ne![PolygonOp::OnPoint(3, *I)], ne![SegmentOp::PointAlongSegment(*I, One)])))]
    #[test_matrix([Polygon([*G, *Q, *S, *I])], [(*U, *Q), (*P, *Q), (*V, *Q)] => Some((ne![PolygonOp::OnPoint(1, *Q)], ne![SegmentOp::PointAlongSegment(*Q, One)])))]
    // segment begins outside and ends on an edge
    #[test_matrix([Polygon([*I, *G, *Q, *S])], [(*C, *H), (*B, *H), (*D, *H)] => Some((ne![PolygonOp::PointAlongEdge(0, *H, Val(0.5))], ne![SegmentOp::PointAlongSegment(*H, One)])))]
    // segment begins outside and ends inside
    #[test_case(Polygon([*I, *G, *Q, *S]), (*C, *M) => Some((ne![PolygonOp::SegmentWithinArea(Segment(*H, *M))], ne![SegmentOp::Subsegment(Segment(*H, *M))])))]
    #[test_case(Polygon([*I, *G, *Q, *S]), (*E, *M) => Some((ne![PolygonOp::SegmentWithinArea(Segment(*I, *M))], ne![SegmentOp::Subsegment(Segment(*I, *M))])))]
    // segment begins at a point and ends outside
    #[test_case(Polygon([*I, *G, *Q, *S]), (*I, *J) => Some((ne![PolygonOp::OnPoint(0, *I)], ne![SegmentOp::PointAlongSegment(*I, Zero)])))]
    // segment begins at a point and ends outside and passes totally through the polygon
    #[test_case(Polygon([*I, *G, *Q, *S]), (*I, *U) => Some((ne![PolygonOp::SegmentWithinArea(Segment(*I, *Q))], ne![SegmentOp::Subsegment(Segment(*I, *Q))])))]
    // segment begins at a point and ends at a point
    #[test_case(Polygon([*I, *G, *Q, *S]), (*I, *G) => Some((ne![PolygonOp::EntireEdge(0)], ne![SegmentOp::EntireSegment])))]
    // segment begins at a point and ends on an edge
    #[test_case(Polygon([*I, *G, *Q, *S]), (*I, *H) => Some((ne![PolygonOp::SubsegmentOfEdge(0, Segment(*I, *H))], ne![SegmentOp::EntireSegment])))]
    // segment begins at a point and ends inside
    #[test_case(Polygon([*I, *G, *Q, *S]), (*I, *M) => Some((ne![PolygonOp::SegmentWithinArea(Segment(*I, *M))], ne![SegmentOp::EntireSegment])))]
    // segment begins on an edge and ends outside
    #[test_case(Polygon([*I, *G, *Q, *S]), (*N, *O) => Some((ne![PolygonOp::PointAlongEdge(3, *N, Val(0.5))], ne![SegmentOp::PointAlongSegment(*N, Zero)])))]
    // segment begins on an edge and ends at a point
    #[test_case(Polygon([*I, *G, *Q, *S]), (*N, *I) => Some((ne![PolygonOp::SubsegmentOfEdge(3, Segment(*N, *I))], ne![SegmentOp::EntireSegment])))]
    // segment begins on an edge and ends on an edge
    #[test_case(Polygon([*I, *G, *Q, *S]), (*N, *H) => Some((ne![PolygonOp::SegmentWithinArea(Segment(*N, *H))], ne![SegmentOp::EntireSegment])))]
    // segment begins on an edge and ends inside
    #[test_case(Polygon([*I, *G, *Q, *S]), (*N, *M) => Some((ne![PolygonOp::SegmentWithinArea(Segment(*N, *M))], ne![SegmentOp::EntireSegment])))]
    // segment begins inside and ends outside
    #[test_case(Polygon([*I, *G, *Q, *S]), (*M, *O) => Some((ne![PolygonOp::SegmentWithinArea(Segment(*M, *N))], ne![SegmentOp::Subsegment(Segment(*M, *N))])))]
    // segment begins inside and ends at a point
    #[test_case(Polygon([*I, *G, *Q, *S]), (*M, *I) => Some((ne![PolygonOp::SegmentWithinArea(Segment(*M, *I))], ne![SegmentOp::EntireSegment])))]
    // segment begins inside and ends on an edge
    #[test_case(Polygon([*I, *G, *Q, *S]), (*M, *N) => Some((ne![PolygonOp::SegmentWithinArea(Segment(*M, *N))], ne![SegmentOp::EntireSegment])))]
    // segment begins inside and ends inside
    #[test_case(Polygon([*A, *U, *Y, *E]), (*G, *I) => Some((ne![PolygonOp::SegmentWithinArea(Segment(*G, *I))], ne![SegmentOp::EntireSegment])))]
    fn test_polygon_overlaps_segment(
        pg: Result<Polygon>,
        sg: impl Into<Segment>,
    ) -> Option<(NonEmpty<PolygonOp>, NonEmpty<SegmentOp>)> {
        polygon_overlaps_segment(&pg.unwrap(), &sg.into()).unwrap()
    }

    // yeah.... 4^3==64 test cases. that's life in the big city

    // multiline begins outside, pivots outside, and ends outside. no intersections
    #[test_case(
        Polygon([*I, *G, *Q, *S]), Multiline([*A, *C, *E])
        => None
    )]
    // multiline begins outside, pivots outside, and ends outside. intersects along edge of polygon
    #[test_case(
        Polygon([*I, *G, *Q, *S]), Multiline([*A, *F, *J])
        => Some((
            ne![PolygonOp::EntireEdge(0)],
            ne![MultilineOp::SubsegmentOf(1, Segment(*I, *G))]
        ))
    )]
    // multiline begins outside, pivots outside, and ends outside. intersects through one point of polygon
    #[test_case(
        Polygon([*I, *G, *Q, *S]), Multiline([*A, *C, *K])
        => Some((
            ne![PolygonOp::OnPoint(1, *G)],
            ne![MultilineOp::PointAlongSegmentOf(1, *G, Val(0.5))]
        ))
    )]
    // multiline begins outside, pivots outside, and ends outside. intersects through two points of polygon
    #[test_case(
        Polygon([*I, *G, *Q, *S]), Multiline([*E, *A, *Y])
        => Some((
            ne![PolygonOp::SegmentWithinArea(Segment(*G, *S))],
            ne![MultilineOp::SubsegmentOf(1, Segment(*G, *S))]
        ))
    )]
    // multiline begins outside, pivots outside, and ends on a point. no intersections
    #[test_case(
        Polygon([*I, *G, *Q, *S]), Multiline([*A, *C, *I])
        => Some((
            ne![PolygonOp::OnPoint(0, *I)],
            ne![MultilineOp::Point(2, *I)]
        ))
    )]
    // multiline begins outside, pivots outside, and ends on a point. intersects through an edge
    #[test_case(
        Polygon([*I, *G, *Q, *S]), Multiline([*A, *F, *I])
        => Some((
            ne![PolygonOp::EntireEdge(0)],
            ne![MultilineOp::SubsegmentOf(1, Segment(*I, *G))]
        ))
    )]
    // multiline begins outside, pivots outside, and ends on an edge. no other intersections
    #[test_case(
        Polygon([*I, *G, *Q, *S]), Multiline([*A, *F, *H])
        => Some((
            ne![PolygonOp::SubsegmentOfEdge(0, Segment(*H, *G))],
            ne![MultilineOp::SubsegmentOf(1, Segment(*G, *H))]
        ))
    )]
    // multiline begins outside, pivots outside, and ends on an edge. intersects along an edge
    #[test_case(
        Polygon([*I, *G, *Q, *S]), Multiline([*A, *C, *H])
        => Some((
            ne![PolygonOp::PointAlongEdge(0, *H, Val(0.5))],
            ne![MultilineOp::Point(2, *H)]
        ))
    )]
    // multiline begins outside, pivots outside, and ends inside.
    #[test_case(
        Polygon([*I, *G, *Q, *S]), Multiline([*A, *C, *M])
        => Some((
            ne![PolygonOp::SegmentWithinArea(Segment(*H, *M))],
            ne![MultilineOp::SubsegmentOf(1, Segment(*H, *M))]
        ))
    )]
    #[test_case(
        Polygon([*I, *G, *Q, *S]), Multiline([*C, *A, *M])
        => Some((
            ne![PolygonOp::SegmentWithinArea(Segment(*G, *M))],
            ne![MultilineOp::SubsegmentOf(1, Segment(*G, *M))]
        ))
    )]
    // multiline begins outside, pivots on a point, and ends outside. no other intersections
    #[test_case(
        Polygon([*I, *G, *Q, *S]), Multiline([*B, *G, *F])
        => Some((
            ne![PolygonOp::OnPoint(1, *G)],
            ne![MultilineOp::Point(1, *G)],
        ))
    )]
    // multiline begins outside, pivots on a point, and ends outside. intersects along an edge
    #[test_case(
        Polygon([*I, *G, *Q, *S]), Multiline([*D, *I, *F])
        => Some((
            ne![PolygonOp::EntireEdge(0)],
            ne![MultilineOp::SubsegmentOf(1, Segment(*I, *G))]
        ))
    )]
    #[test_case(
        Polygon([*I, *G, *Q, *S]), Multiline([*B, *G, *J])
        => Some((
            ne![PolygonOp::EntireEdge(0)],
            ne![MultilineOp::SubsegmentOf(1, Segment(*I, *G))]
        ))
    )]
    // multiline begins outside, pivots on a point, and ends on a point.
    #[test_case(
        Polygon([*I, *G, *Q, *S]), Multiline([*B, *G, *I])
        => Some((
            ne![PolygonOp::EntireEdge(0)],
            ne![MultilineOp::EntireSubsegment(1)],
        ))
    )]
    #[test_case(
        Polygon([*I, *G, *Q, *S]), Multiline([*B, *G, *S])
        => Some((
            ne![PolygonOp::SegmentWithinArea(Segment(*G, *S))],
            ne![MultilineOp::EntireSubsegment(1)],
        ))
    )]
    // multiline begins outside, pivots on a point, and ends on an edge.
    #[test_case(
        Polygon([*I, *G, *Q, *S]), Multiline([*B, *G, *H])
        => Some((
            ne![PolygonOp::SubsegmentOfEdge(0, Segment(*G, *H))],
            ne![MultilineOp::EntireSubsegment(1)],
        ))
    )]
    #[test_case(
        Polygon([*I, *G, *Q, *S]), Multiline([*B, *G, *N])
        => Some((
            ne![PolygonOp::SegmentWithinArea(Segment(*G, *N))],
            ne![MultilineOp::EntireSubsegment(1)],
        ))
    )]
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
    // multiline begins outside, pivots on a point, and ends inside.
    // multiline begins outside, pivots on an edge, and ends outside.
    // multiline begins outside, pivots on an edge, and ends on a point.
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

    fn test_polygon_overlaps_multiline(
        pg: Result<Polygon>,
        ml: Multiline,
    ) -> Option<(NonEmpty<PolygonOp>, NonEmpty<MultilineOp>)> {
        polygon_overlaps_multiline(&pg.unwrap(), &ml).unwrap()
    }
}
