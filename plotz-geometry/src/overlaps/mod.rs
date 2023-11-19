#![allow(missing_docs)]

pub mod opinion;

use self::opinion::{
    rewrite_multiline_opinions, rewrite_segment_opinions, MultilineOp, PolygonOp, SegmentOp,
};
use crate::{
    interpolate::interpolate_2d_checked,
    shapes::{
        multiline::Multiline,
        point::Point,
        polygon::{abp, Polygon},
        segment::Segment,
    },
    utils::Percent,
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

pub fn point_overlaps_point(a: &Point, b: &Point) -> Result<Option<Point>> {
    if a == b {
        Ok(Some(*a))
    } else {
        Ok(None)
    }
}

pub fn segment_overlaps_point(s: &Segment, p: &Point) -> Result<Option<(SegmentOp, Point)>> {
    if s.i == *p {
        Ok(Some((SegmentOp::PointAlongSegment(*p, Percent::Zero), *p)))
    } else if s.f == *p {
        Ok(Some((SegmentOp::PointAlongSegment(*p, Percent::One), *p)))
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

    if sa == sb || *sa == sb.flip() {
        return Ok(Some((SegmentOp::EntireSegment, SegmentOp::EntireSegment)));
    }

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
            //
            // |-->|
            // |-->|
            (Some(_), Some(_), Some(_), Some(_)) => {
                return Err(anyhow!(
                    "these are the same line; sa==sb should have triggered."
                ));
            }

            // |-->|
            // |--->|
            // or
            //  |-->|
            // |--->|
            // or
            //  |-->|
            // |---->|
            (Some(_), Some(_), _, _) => Some(*sa),

            // |---->|
            // |-->|
            // or
            // |---->|
            //  |-->|
            // |--->|
            //  |-->|
            (_, _, Some(_), Some(_)) => Some(*sb),

            (Some(_), None, None, Some(_)) => {
                if sa.i == sb.f {
                    //     |-->|
                    // |-->|
                    let pt = sa.i;
                    return Ok(Some((
                        SegmentOp::PointAlongSegment(pt, Percent::Zero),
                        SegmentOp::PointAlongSegment(pt, Percent::One),
                    )));
                }
                //    |--->|
                // |--->|
                Some(Segment(sa.i, sb.f))
            }

            (None, Some(_), Some(_), None) => {
                if sa.f == sb.i {
                    // |-->|
                    //     |-->|
                    let pt = sa.f;
                    return Ok(Some((
                        SegmentOp::PointAlongSegment(pt, Percent::One),
                        SegmentOp::PointAlongSegment(pt, Percent::Zero),
                    )));
                }
                // |--->|
                //    |--->|
                Some(Segment(sb.i, sa.f))
            }

            // Head-to-head collision.
            (Some(_), None, Some(_), None) => {
                if sa.i == sb.i {
                    // |<--|
                    //     |-->|
                    let pt = sa.i;
                    return Ok(Some((
                        SegmentOp::PointAlongSegment(pt, Percent::Zero),
                        SegmentOp::PointAlongSegment(pt, Percent::Zero),
                    )));
                }
                // |<---|
                //    |--->|
                Some(Segment(sa.i, sb.i))
            }

            // Tail-to-tail collision.
            (None, Some(_), None, Some(_)) => {
                if sa.f == sb.f {
                    //     |<--|
                    // |-->|
                    let pt = sa.f;
                    return Ok(Some((
                        SegmentOp::PointAlongSegment(pt, Percent::One),
                        SegmentOp::PointAlongSegment(pt, Percent::One),
                    )));
                }
                //   |<--|
                // |-->|
                Some(Segment(sa.f, sb.f))
            }

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
    let mut sg_ops: Vec<MultilineOp> = vec![];
    for (index, sg) in ml.to_segments().iter().enumerate() {
        if let Some((segment_opinion, _)) = segment_overlaps_point(sg, p)? {
            sg_ops.push(MultilineOp::from_segment_opinion(index, segment_opinion));
        }
    }
    sg_ops.dedup();
    match NonEmpty::from_vec(sg_ops) {
        None => Ok(None),
        Some(u) => Ok(Some((u, *p))),
    }
}

pub fn multiline_overlaps_segment(
    ml: &Multiline,
    sg: &Segment,
) -> Result<Option<(NonEmpty<MultilineOp>, NonEmpty<SegmentOp>)>> {
    let mut ml_opinions: Vec<MultilineOp> = vec![];
    let mut sg_opinions: Vec<SegmentOp> = vec![];

    for (ml_sg_idx, ml_sg) in ml.to_segments().iter().enumerate() {
        if let Some((ml_sg_op, sg_op)) = segment_overlaps_segment(ml_sg, sg)? {
            ml_opinions.push(MultilineOp::from_segment_opinion(ml_sg_idx, ml_sg_op));
            sg_opinions.push(sg_op);
        }
    }

    rewrite_multiline_opinions(&mut ml_opinions)?;
    rewrite_segment_opinions(&mut sg_opinions, sg)?;

    match (
        NonEmpty::from_vec(ml_opinions),
        NonEmpty::from_vec(sg_opinions),
    ) {
        (Some(total_ml_ops), Some(total_sg_ops)) => Ok(Some((total_ml_ops, total_sg_ops))),
        (Some(_), None) | (None, Some(_)) => Err(anyhow!(
            "unexpected case - how can one object see collisions but the other doesn't?"
        )),
        (None, None) => Ok(None),
    }
}

pub fn multiline_overlaps_multiline(
    ml1: &Multiline,
    ml2: &Multiline,
) -> Result<Option<(NonEmpty<MultilineOp>, NonEmpty<MultilineOp>)>> {
    let mut ml1_opinions: Vec<MultilineOp> = vec![];
    let mut ml2_opinions: Vec<MultilineOp> = vec![];

    for (ml_sg1_idx, ml_sg1) in ml1.to_segments().iter().enumerate() {
        for (ml_sg2_idx, ml_sg2) in ml2.to_segments().iter().enumerate() {
            if let Some((ml_sg1_op, ml_sg2_op)) = segment_overlaps_segment(ml_sg1, ml_sg2)? {
                ml1_opinions.push(MultilineOp::from_segment_opinion(ml_sg1_idx, ml_sg1_op));
                ml2_opinions.push(MultilineOp::from_segment_opinion(ml_sg2_idx, ml_sg2_op));
            }
        }
    }

    rewrite_multiline_opinions(&mut ml1_opinions)?;
    rewrite_multiline_opinions(&mut ml2_opinions)?;

    match (
        NonEmpty::from_vec(ml1_opinions),
        NonEmpty::from_vec(ml2_opinions),
    ) {
        (Some(total_ml1_ops), Some(total_ml2_ops)) => Ok(Some((total_ml1_ops, total_ml2_ops))),
        (Some(_), None) | (None, Some(_)) => Err(anyhow!(
            "unexpected case - how can one object see collisions but the other doesn't?"
        )),
        (None, None) => Ok(None),
    }
    //
}

pub fn polygon_overlaps_point(
    polygon: &Polygon,
    point: &Point,
) -> Result<Option<(PolygonOp, Point)>> {
    for (index, pg_pt) in polygon.pts.iter().enumerate() {
        if pg_pt == point {
            return Ok(Some((PolygonOp::Point(index, *point), *point)));
        }
    }
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

    let theta: f64 = (polygon.pts.iter())
        .zip(polygon.pts.iter().cycle().skip(1))
        .map(|(i, j)| abp(point, i, j))
        .sum();

    if approx_eq!(f64, theta, 0_f64, epsilon = 0.00001) {
        Ok(None)
    } else {
        Ok(Some((PolygonOp::WithinArea, *point)))
    }
}

pub fn polygon_overlaps_segment(
    polygon: &Polygon,
    segment: &Segment,
) -> Result<Option<(NonEmpty<PolygonOp>, NonEmpty<SegmentOp>)>> {
    let mut _pg_ops: Vec<PolygonOp> = vec![];
    let mut sg_ops: Vec<SegmentOp> = vec![];
    for (_pg_sg_idx, pg_sg) in polygon.to_segments().iter().enumerate() {
        if let Some((_pg_sg_op, _sg_op)) = segment_overlaps_segment(pg_sg, segment)? {
            // match pg_sg_op {
            // SegmentOp::PointAlongSegment(_, _) => todo!(),
            // SegmentOp::Subsegment(_) => todo!(),
            // SegmentOp::EntireSegment => todo!(),
            // }
            //
            // match sg_op {
            // SegmentOp::PointAlongSegment(_, _) => todo!(),
            // SegmentOp::Subsegment(_) => todo!(),
            // SegmentOp::EntireSegment => todo!(),
            // }
        }
    }

    // rewrite_polygon_opinions(&mut pg_ops, polygon)?;
    rewrite_segment_opinions(&mut sg_ops, segment)?;

    match (NonEmpty::from_vec(_pg_ops), NonEmpty::from_vec(sg_ops)) {
        (Some(total_pg_ops), Some(total_sg_ops)) => Ok(Some((total_pg_ops, total_sg_ops))),
        (None, None) => {
            // check the unusual case of no intersections, but the segment is totally contained within the polygon.
            match (
                polygon_overlaps_point(polygon, &segment.i)?,
                polygon_overlaps_point(polygon, &segment.f)?,
            ) {
                (Some(_), Some(_)) => Ok(Some((nonempty![PolygonOp::WithinArea], nonempty![SegmentOp::EntireSegment]))),
                (None, None) => Ok(None),
                _ => Err(anyhow!("unexpected case - how can one end of the segment be within the polygon, the other without, but still we didn't see any collisions above?")),
            }
        }
        _ => Err(anyhow!(
            "unexpected case - how can one object see collisions but the other doesn't?"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::lazy_static;
    use nonempty::nonempty as ne;
    use test_case::{test_case, test_matrix};
    use Percent::{One, Val, Zero};

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
        assert_eq!(point_overlaps_point(&a, &b)?, expectation);
        Ok(())
    }

    #[test_case((*C, *D), *C, Some((SegmentOp::PointAlongSegment(*C, Zero), *C)); "at start 00")]
    #[test_case((*C, *D), *D, Some((SegmentOp::PointAlongSegment(*D, One), *D)); "at end 00")]
    #[test_case((*C, *I), *C, Some((SegmentOp::PointAlongSegment(*C, Zero), *C)); "at start 01")]
    #[test_case((*C, *I), *I, Some((SegmentOp::PointAlongSegment(*I, One), *I)); "at end 01")]
    #[test_case((*C, *E), *D, Some((SegmentOp::PointAlongSegment(*D, Val(0.5)), *D)); "halfway along 01")]
    #[test_case((*C, *O), *I, Some((SegmentOp::PointAlongSegment(*I, Val(0.5)), *I)); "halfway along 02")]
    #[test_case((*C, *W), *M, Some((SegmentOp::PointAlongSegment(*M, Val(0.5)), *M)); "halfway along 03")]
    fn test_segment_overlaps_point(
        segment: impl Into<Segment>,
        point: Point,
        expectation: Option<(SegmentOp, Point)>,
    ) -> Result<()> {
        assert_eq!(
            segment_overlaps_point(&segment.into(), &point)?,
            expectation
        );
        Ok(())
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

    #[test_case(Multiline([*A, *C, *E, *O, *Y]), *A, Some((ne![MultilineOp::Point(0, *A)], *A)); "multiline point at index 0")]
    #[test_case(Multiline([*A, *C, *E, *O, *Y]), *E, Some((ne![MultilineOp::Point(2, *E)], *E)); "multiline point at index 2")]
    #[test_case(Multiline([*A, *C, *E, *O, *Y]), *Y, Some((ne![MultilineOp::Point(4, *Y)], *Y)); "multiline point at index 4")]
    #[test_case(Multiline([*A, *C, *E, *O, *Y]), *B, Some((ne![MultilineOp::PointAlongSegmentOf(0, *B, Val(0.5))], *B)); "multiline point along segment 0")]
    #[test_case(Multiline([*A, *C, *E, *O, *Y]), *D, Some((ne![MultilineOp::PointAlongSegmentOf(1, *D, Val(0.5))], *D)); "multiline point along segment 1")]
    #[test_case(Multiline([*A, *C, *E, *O, *Y]), *J, Some((ne![MultilineOp::PointAlongSegmentOf(2, *J, Val(0.5))], *J)); "multiline point along segment 2")]
    #[test_case(Multiline([*A, *C, *E, *O, *Y]), *T, Some((ne![MultilineOp::PointAlongSegmentOf(3, *T, Val(0.5))], *T)); "multiline point along segment 3")]
    #[test_case(Multiline([*A, *C, *E, *O, *Y]), *M, None; "unrelated")]
    fn test_multiline_overlaps_point(
        multiline: Multiline,
        point: Point,
        expectation: Option<(NonEmpty<MultilineOp>, Point)>,
    ) -> Result<()> {
        assert_eq!(multiline_overlaps_point(&multiline, &point)?, expectation);
        Ok(())
    }

    #[test_case(Multiline([*C, *E, *O]), (*M, *N), None; "none 00")]
    #[test_case(Multiline([*C, *E, *J]), (*M, *N), None; "none 01")]
    #[test_case(Multiline([*C, *E, *O]), (*H, *N), None; "none 02")]
    #[test_case(Multiline([*C, *I, *O]), (*D, *J), None; "none 03")]
    #[test_case(Multiline([*C, *E, *O]), (*C, *M), Some((ne![MultilineOp::Point(0, *C)], ne![SegmentOp::PointAlongSegment(*C, Zero)])); "at point at point 00")]
    #[test_case(Multiline([*E, *O, *M]), (*E, *C), Some((ne![MultilineOp::Point(0, *E)], ne![SegmentOp::PointAlongSegment(*E, Zero)])); "at point at point 01")]
    #[test_case(Multiline([*O, *M, *C]), (*O, *E), Some((ne![MultilineOp::Point(0, *O)], ne![SegmentOp::PointAlongSegment(*O, Zero)])); "at point at point 02")]
    #[test_case(Multiline([*C, *I, *O]), (*C, *M), Some((ne![MultilineOp::Point(0, *C)], ne![SegmentOp::PointAlongSegment(*C, Zero)])); "at point at point 03")]
    #[test_case(Multiline([*C, *E, *O]), (*M, *C), Some((ne![MultilineOp::Point(0, *C)], ne![SegmentOp::PointAlongSegment(*C, One)])); "at point at point 04")]
    #[test_case(Multiline([*E, *O, *M]), (*C, *E), Some((ne![MultilineOp::Point(0, *E)], ne![SegmentOp::PointAlongSegment(*E, One)])); "at point at point 05")]
    #[test_case(Multiline([*O, *M, *C]), (*E, *O), Some((ne![MultilineOp::Point(0, *O)], ne![SegmentOp::PointAlongSegment(*O, One)])); "at point at point 06")]
    #[test_case(Multiline([*C, *I, *O]), (*M, *C), Some((ne![MultilineOp::Point(0, *C)], ne![SegmentOp::PointAlongSegment(*C, One)])); "at point at point 07")]
    #[test_case(Multiline([*D, *I, *N]), (*C, *E), Some((ne![MultilineOp::Point(0, *D)], ne![SegmentOp::PointAlongSegment(*D, Val(0.5))])); "at point at point 08")]
    #[test_case(Multiline([*H, *I, *J]), (*M, *C), Some((ne![MultilineOp::Point(0, *H)], ne![SegmentOp::PointAlongSegment(*H, Val(0.5))])); "at point at point 09")]
    #[test_case(Multiline([*D, *I, *N]), (*I, *J), Some((ne![MultilineOp::Point(1, *I)], ne![SegmentOp::PointAlongSegment(*I, Zero)])); "at point at point 10")]
    #[test_case(Multiline([*D, *I, *N]), (*H, *I), Some((ne![MultilineOp::Point(1, *I)], ne![SegmentOp::PointAlongSegment(*I, One)])); "at point at point 11")]
    #[test_case(Multiline([*D, *I, *N]), (*H, *J), Some((ne![MultilineOp::Point(1, *I)], ne![SegmentOp::PointAlongSegment(*I, Val(0.5))])); "at point at point 12")]
    #[test_case(Multiline([*D, *I, *N]), (*N, *O), Some((ne![MultilineOp::Point(2, *N)], ne![SegmentOp::PointAlongSegment(*N, Zero)])); "at point at point 13")]
    #[test_case(Multiline([*D, *I, *N]), (*M, *N), Some((ne![MultilineOp::Point(2, *N)], ne![SegmentOp::PointAlongSegment(*N, One)])); "at point at point 14")]
    #[test_case(Multiline([*D, *I, *N]), (*M, *O), Some((ne![MultilineOp::Point(2, *N)], ne![SegmentOp::PointAlongSegment(*N, Val(0.5))])); "at point at point 15")]
    #[test_case( Multiline([*C, *E, *O]), (*C, *O), Some((ne![MultilineOp::Point(0, *C), MultilineOp::Point(2, *O) ], ne![ SegmentOp::PointAlongSegment(*C, Zero), SegmentOp::PointAlongSegment(*O, One)])); "segment bookends 1")]
    #[test_case( Multiline([*C, *E, *O]), (*D, *J), Some((ne![MultilineOp::PointAlongSegmentOf(0, *D, Val(0.5)), MultilineOp::PointAlongSegmentOf(1, *J, Val(0.5)) ], ne![ SegmentOp::PointAlongSegment(*D, Zero), SegmentOp::PointAlongSegment(*J, One)])); "segment bookends 2")]
    #[test_case( Multiline([*C, *D, *E]), (*C, *D), Some((ne![MultilineOp::EntireSubsegment(0)], ne![SegmentOp::EntireSegment])); "partial collision")]
    #[test_case( Multiline([*C, *D, *E]), (*D, *C), Some((ne![MultilineOp::EntireSubsegment(0)], ne![SegmentOp::EntireSegment])); "partial collision 02")]
    #[test_case( Multiline([*C, *D, *E]), (*D, *E), Some((ne![MultilineOp::EntireSubsegment(1)], ne![SegmentOp::EntireSegment])); "partial collision 03")]
    #[test_case( Multiline([*C, *D, *E]), (*E, *D), Some((ne![MultilineOp::EntireSubsegment(1)], ne![SegmentOp::EntireSegment])); "partial collision 04")]
    #[test_case( Multiline([*C, *D, *E]), (*C, *E), Some((ne![MultilineOp::EntireSubsegment(0), MultilineOp::EntireSubsegment(1) ], ne![ SegmentOp::EntireSegment ])); "total collision 01")]
    #[test_case( Multiline([*C, *D, *E]), (*E, *C), Some((ne![MultilineOp::EntireSubsegment(0), MultilineOp::EntireSubsegment(1) ], ne![ SegmentOp::EntireSegment ])); "total collision 01 flip")]
    #[test_case( Multiline([*C, *D, *E]), (Point(0.5,2), Point(1.5,2)), Some(( ne![ MultilineOp::SubsegmentOf(0, Segment(Point(0.5,2),Point(1,2))), MultilineOp::SubsegmentOf(1, Segment(Point(1,2), Point(1.5,2))) ], ne![SegmentOp::EntireSegment])); "total collision half shift 01")]
    #[test_case(Multiline([*C, *D, *E]), (Point(1.5,2), Point(0.5,2)), Some(( ne![ MultilineOp::SubsegmentOf(0, Segment(Point(0.5,2),Point(1,2))), MultilineOp::SubsegmentOf(1, Segment(Point(1,2), Point(1.5,2))) ], ne![SegmentOp::EntireSegment])); "total collision half shift 01 flip")]
    #[test_case(Multiline([*H, *J, *O]), (*D, *N), Some((ne![MultilineOp::PointAlongSegmentOf(0, *I, Val(0.5))], ne![SegmentOp::PointAlongSegment(*I, Val(0.5))])); "at point on segment at point on segment 00")]
    #[test_case(Multiline([*H, *J, *O]), (*I, *N), Some((ne![MultilineOp::PointAlongSegmentOf(0, *I, Val(0.5))], ne![SegmentOp::PointAlongSegment(*I, Zero)])); "at point on segment at point on segment 01")]
    #[test_case(Multiline([*H, *J, *O]), (*D, *I), Some((ne![MultilineOp::PointAlongSegmentOf(0, *I, Val(0.5))], ne![SegmentOp::PointAlongSegment(*I, One)])); "at point on segment at point on segment 02")]
    #[test_case(Multiline([*M, *H, *J]), (*D, *N), Some((ne![MultilineOp::PointAlongSegmentOf(1, *I, Val(0.5))], ne![SegmentOp::PointAlongSegment(*I, Val(0.5))])); "at point on segment at point on segment 03")]
    #[test_case(Multiline([*M, *H, *J]), (*I, *N), Some((ne![MultilineOp::PointAlongSegmentOf(1, *I, Val(0.5))], ne![SegmentOp::PointAlongSegment(*I, Zero)])); "at point on segment at point on segment 04")]
    #[test_case(Multiline([*M, *H, *J]), (*D, *I), Some((ne![MultilineOp::PointAlongSegmentOf(1, *I, Val(0.5))], ne![SegmentOp::PointAlongSegment(*I, One)])); "at point on segment at point on segment 05")]
    fn test_multiline_overlaps_segment(
        ml: Multiline,
        sg: impl Into<Segment>,
        expectation: Option<(NonEmpty<MultilineOp>, NonEmpty<SegmentOp>)>,
    ) -> Result<()> {
        let sg = sg.into();
        assert_eq!(multiline_overlaps_segment(&ml, &sg)?, expectation);
        Ok(())
    }

    #[test_case(Multiline([*C, *D, *E]), Multiline([*H, *I, *J]), None; "none 01")]
    #[test_case(Multiline([*C, *D, *E]), Multiline([*M, *N, *O]), None; "none 02")]
    #[test_case(Multiline([*C, *I, *O]), Multiline([*D, *J]), None; "none diagonal")]
    #[test_case(Multiline([*C, *D, *E]), Multiline([*C, *H, *M]), Some((ne![MultilineOp::Point(0, *C)], ne![MultilineOp::Point(0, *C)])); "AtPoint 0, AtPoint 0")]
    #[test_case(Multiline([*C, *D, *E]), Multiline([*M, *H, *C]), Some((ne![MultilineOp::Point(0, *C)], ne![MultilineOp::Point(2, *C)])); "AtPoint 0, AtPoint 2")]
    #[test_case(Multiline([*E, *D, *C]), Multiline([*M, *H, *C]), Some((ne![MultilineOp::Point(2, *C)], ne![MultilineOp::Point(2, *C)])); "AtPoint 2, AtPoint 2")]
    #[test_case(Multiline([*C, *I, *O]), Multiline([*M, *I, *E]), Some((ne![MultilineOp::Point(1, *I)], ne![MultilineOp::Point(1, *I)])); "AtPoint 1, AtPoint 1")]
    #[test_case(Multiline([*C, *O]), Multiline([*E, *M]), Some((ne![MultilineOp::PointAlongSegmentOf(0, *I, Val(0.5))], ne![MultilineOp::PointAlongSegmentOf(0, *I, Val(0.5))])); "crosshairs")]
    #[test_case(Multiline([*C, *D, *E]), Multiline([*C, *D, *I]), Some((ne![MultilineOp::EntireSubsegment(0)], ne![MultilineOp::EntireSubsegment(0)])); "partial collision, entire subsegment 0 0")]
    #[test_case(Multiline([*E, *D, *C]), Multiline([*I, *D, *C]), Some((ne![MultilineOp::EntireSubsegment(1)], ne![MultilineOp::EntireSubsegment(1)])); "partial collision, entire subsegment 1 1")]
    #[test_case(Multiline([*C, *D, *E]), Multiline([*D, *E, *J]), Some((ne![MultilineOp::EntireSubsegment(1)], ne![MultilineOp::EntireSubsegment(0)])); "partial collision, entire subsegment 1 0")]
    #[test_case(Multiline([*C, *D, *E]), Multiline([*E, *D, *C]), Some((ne![MultilineOp::EntireSubsegment(0), MultilineOp::EntireSubsegment(1)], ne![MultilineOp::EntireSubsegment(1), MultilineOp::EntireSubsegment(0)])); "partial collision, entire subsegment 01 01 flipped")]
    #[test_case(Multiline([*C, *D, *E, *J, *O]), Multiline([*C, *D, *I, *J, *O]), Some((ne![MultilineOp::EntireSubsegment(0), MultilineOp::EntireSubsegment(3)], ne![MultilineOp::EntireSubsegment(0), MultilineOp::EntireSubsegment(3)])); "shared segment, then diversion, then another shared segment")]
    #[test_case(Multiline([*C, *D, *E, *J, *O]), Multiline([*C, *D, *I, *J]), Some((ne![MultilineOp::EntireSubsegment(0), MultilineOp::Point(3, *J)], ne![MultilineOp::EntireSubsegment(0), MultilineOp::Point(3, *J)])); "shared segment, then diversion, then atpoint")]
    fn test_multiline_overlaps_multiline(
        ml1: Multiline,
        ml2: Multiline,
        expectation: Option<(NonEmpty<MultilineOp>, NonEmpty<MultilineOp>)>,
    ) -> Result<()> {
        assert_eq!(multiline_overlaps_multiline(&ml1, &ml2)?, expectation);
        Ok(())
    }

    #[test_case(Polygon([*D, *H, *N, *J]), &C, None; "point not in polygon 00")]
    #[test_case(Polygon([*D, *H, *N, *J]), &E, None; "point not in polygon 01")]
    #[test_case(Polygon([*D, *H, *N, *J]), &I, Some((PolygonOp::WithinArea, *I)); "point in polygon")]
    #[test_case(Polygon([*D, *H, *N, *J]), &D, Some((PolygonOp::Point(0, *D), *D)); "point at point of polygon 00")]
    #[test_case(Polygon([*D, *H, *N, *J]), &H, Some((PolygonOp::Point(1, *H), *H)); "point at point of polygon 01")]
    #[test_case(Polygon([*D, *H, *N, *J]), &N, Some((PolygonOp::Point(2, *N), *N)); "point at point of polygon 02")]
    #[test_case(Polygon([*D, *H, *N, *J]), &J, Some((PolygonOp::Point(3, *J), *J)); "point at point of polygon 03")]
    #[test_case(Polygon([*C, *M, *O, *E]), &H, Some((PolygonOp::PointAlongEdge(0, *H, Val(0.5)), *H)); "point at edge of polygon 00")]
    #[test_case(Polygon([*C, *M, *O, *E]), &N, Some((PolygonOp::PointAlongEdge(1, *N, Val(0.5)), *N)); "point at edge of polygon 01")]
    #[test_case(Polygon([*C, *M, *O, *E]), &J, Some((PolygonOp::PointAlongEdge(2, *J, Val(0.5)), *J)); "point at edge of polygon 02")]
    #[test_case(Polygon([*C, *M, *O, *E]), &D, Some((PolygonOp::PointAlongEdge(3, *D, Val(0.5)), *D)); "point at edge of polygon 03")]
    fn test_polygon_overlaps_point(
        pg: Result<Polygon>,
        pt: &Point,
        expectation: Option<(PolygonOp, Point)>,
    ) -> Result<()> {
        assert_eq!(polygon_overlaps_point(&pg?, pt)?, expectation);
        Ok(())
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

    // No overlap, each segment is wholly outside of the polygon.
    #[test_matrix([
        Polygon([*G, *Q, *S, *I])],
        [ (*A, *E), (*E, *A), (*B, *F), (*T, *X), (*O, *J), ],
        None
    )]
    // Each segment is wholly within the polygon - no edge or point overlaps.
    #[test_matrix([
        Polygon([*A, *U, *Y, *E])],
        [ (*G, *I), (*Q, *R), (*I, *M), (*S, *G), ],
        Some((nonempty![PolygonOp::WithinArea], nonempty![SegmentOp::EntireSegment]))
    )]
    fn test_polygon_overlaps_segment(
        pg: Result<Polygon>,
        sg: impl Into<Segment>,
        expectation: Option<(NonEmpty<PolygonOp>, NonEmpty<SegmentOp>)>,
    ) -> Result<()> {
        let sg = sg.into();
        assert_eq!(polygon_overlaps_segment(&pg?, &sg)?, expectation);
        Ok(())
    }
}
