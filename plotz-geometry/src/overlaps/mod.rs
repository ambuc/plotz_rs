#![allow(missing_docs)]

pub mod opinion;

use self::opinion::{
    rewrite_multiline_opinions, rewrite_segment_opinions, MultilineOpinion, PolygonOpinion,
    SegmentOpinion,
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
use nonempty::NonEmpty;

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

pub fn segment_overlaps_point(s: &Segment, p: &Point) -> Result<Option<(SegmentOpinion, Point)>> {
    if s.i == *p {
        Ok(Some((
            SegmentOpinion::AtPointAlongSegment {
                at_point: *p,
                percent_along: Percent::Zero,
            },
            *p,
        )))
    } else if s.f == *p {
        Ok(Some((
            SegmentOpinion::AtPointAlongSegment {
                at_point: *p,
                percent_along: Percent::One,
            },
            *p,
        )))
    } else if approx_eq!(
        f64,
        s.length(),
        Segment(s.i, *p).length() + Segment(*p, s.f).length()
    ) {
        Ok(Some((
            SegmentOpinion::AtPointAlongSegment {
                at_point: *p,
                percent_along: interpolate_2d_checked(s.i, s.f, *p)?,
            },
            *p,
        )))
    } else {
        Ok(None)
    }
}

pub fn segment_overlaps_segment(
    sa: &Segment,
    sb: &Segment,
) -> Result<Option<(SegmentOpinion, SegmentOpinion)>> {
    // NB: sa and sb are _not_ guaranteed to point the same way.

    if sa == sb || *sa == sb.flip() {
        return Ok(Some((
            SegmentOpinion::EntireSegment,
            SegmentOpinion::EntireSegment,
        )));
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
                        SegmentOpinion::AtPointAlongSegment {
                            at_point: pt,
                            percent_along: Percent::Zero,
                        },
                        SegmentOpinion::AtPointAlongSegment {
                            at_point: pt,
                            percent_along: Percent::One,
                        },
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
                        SegmentOpinion::AtPointAlongSegment {
                            at_point: pt,
                            percent_along: Percent::One,
                        },
                        SegmentOpinion::AtPointAlongSegment {
                            at_point: pt,
                            percent_along: Percent::Zero,
                        },
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
                        SegmentOpinion::AtPointAlongSegment {
                            at_point: pt,
                            percent_along: Percent::Zero,
                        },
                        SegmentOpinion::AtPointAlongSegment {
                            at_point: pt,
                            percent_along: Percent::Zero,
                        },
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
                        SegmentOpinion::AtPointAlongSegment {
                            at_point: pt,
                            percent_along: Percent::One,
                        },
                        SegmentOpinion::AtPointAlongSegment {
                            at_point: pt,
                            percent_along: Percent::One,
                        },
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
                    SegmentOpinion::EntireSegment,
                    SegmentOpinion::AlongSubsegment(isxn_segment),
                )));
            } else if isxn_segment == *sb {
                return Ok(Some((
                    SegmentOpinion::AlongSubsegment(isxn_segment),
                    SegmentOpinion::EntireSegment,
                )));
            } else {
                return Ok(Some((
                    SegmentOpinion::AlongSubsegment(
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
                    SegmentOpinion::AlongSubsegment(if isxn_segment.dot(sb) < 0.0 {
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
            SegmentOpinion::AtPointAlongSegment {
                at_point: pt,
                percent_along: interpolate_2d_checked(sa.i, sa.f, pt)?,
            },
            SegmentOpinion::AtPointAlongSegment {
                at_point: pt,
                percent_along: interpolate_2d_checked(sb.i, sb.f, pt)?,
            },
        )));
    }

    Ok(None)
}

pub fn multiline_overlaps_point(
    ml: &Multiline,
    p: &Point,
) -> Result<Option<(NonEmpty<MultilineOpinion>, Point)>> {
    let mut sg_ops: Vec<MultilineOpinion> = vec![];
    for (index, sg) in ml.to_segments().iter().enumerate() {
        if let Some((segment_opinion, _)) = segment_overlaps_point(sg, p)? {
            sg_ops.push(MultilineOpinion::from_segment_opinion(
                index,
                segment_opinion,
            ));
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
) -> Result<Option<(NonEmpty<MultilineOpinion>, NonEmpty<SegmentOpinion>)>> {
    let mut ml_opinions: Vec<MultilineOpinion> = vec![];
    let mut sg_opinions: Vec<SegmentOpinion> = vec![];

    for (ml_sg_idx, ml_sg) in ml.to_segments().iter().enumerate() {
        if let Some((ml_sg_op, sg_op)) = segment_overlaps_segment(ml_sg, sg)? {
            ml_opinions.push(MultilineOpinion::from_segment_opinion(ml_sg_idx, ml_sg_op));
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
) -> Result<Option<(NonEmpty<MultilineOpinion>, NonEmpty<MultilineOpinion>)>> {
    let mut ml1_opinions: Vec<MultilineOpinion> = vec![];
    let mut ml2_opinions: Vec<MultilineOpinion> = vec![];

    for (ml_sg1_idx, ml_sg1) in ml1.to_segments().iter().enumerate() {
        for (ml_sg2_idx, ml_sg2) in ml2.to_segments().iter().enumerate() {
            if let Some((ml_sg1_op, ml_sg2_op)) = segment_overlaps_segment(ml_sg1, ml_sg2)? {
                ml1_opinions.push(MultilineOpinion::from_segment_opinion(
                    ml_sg1_idx, ml_sg1_op,
                ));
                ml2_opinions.push(MultilineOpinion::from_segment_opinion(
                    ml_sg2_idx, ml_sg2_op,
                ));
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
) -> Result<Option<(PolygonOpinion, Point)>> {
    for (index, pg_pt) in polygon.pts.iter().enumerate() {
        if pg_pt == point {
            return Ok(Some((
                PolygonOpinion::AtPoint {
                    index,
                    at_point: *point,
                },
                *point,
            )));
        }
    }
    for (index, pg_sg) in polygon.to_segments().iter().enumerate() {
        if let Some((
            SegmentOpinion::AtPointAlongSegment {
                at_point,
                percent_along,
            },
            _,
        )) = segment_overlaps_point(pg_sg, point)?
        {
            return Ok(Some((
                PolygonOpinion::AlongEdge {
                    index,
                    at_point,
                    percent_along,
                },
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
        Ok(Some((PolygonOpinion::WithinArea, *point)))
    }
}

pub fn polygon_overlaps_segment(
    _polygon: &Polygon,
    _segment: &Segment,
) -> Result<Option<(NonEmpty<PolygonOpinion>, NonEmpty<SegmentOpinion>)>> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::lazy_static;
    use nonempty::nonempty;
    use test_case::test_case;

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

    #[test_case((*C, *D), *C, Some((SegmentOpinion::AtPointAlongSegment { at_point: *C, percent_along: Percent::Zero }, *C)); "at start 00")]
    #[test_case((*C, *D), *D, Some((SegmentOpinion::AtPointAlongSegment { at_point: *D, percent_along: Percent::One }, *D)); "at end 00")]
    #[test_case((*C, *I), *C, Some((SegmentOpinion::AtPointAlongSegment { at_point: *C, percent_along: Percent::Zero }, *C)); "at start 01")]
    #[test_case((*C, *I), *I, Some((SegmentOpinion::AtPointAlongSegment { at_point: *I, percent_along: Percent::One }, *I)); "at end 01")]
    #[test_case((*C, *E), *D, Some((SegmentOpinion::AtPointAlongSegment { at_point: *D, percent_along: Percent::Val(0.5) }, *D)); "halfway along 01")]
    #[test_case((*C, *O), *I, Some((SegmentOpinion::AtPointAlongSegment { at_point: *I, percent_along: Percent::Val(0.5) }, *I)); "halfway along 02")]
    #[test_case((*C, *W), *M, Some((SegmentOpinion::AtPointAlongSegment { at_point: *M, percent_along: Percent::Val(0.5) }, *M)); "halfway along 03")]
    fn test_segment_overlaps_point(
        segment: impl Into<Segment>,
        point: Point,
        expectation: Option<(SegmentOpinion, Point)>,
    ) -> Result<()> {
        assert_eq!(
            segment_overlaps_point(&segment.into(), &point)?,
            expectation
        );
        Ok(())
    }

    #[test_case((*C, *D), (*C, *D), Some((SegmentOpinion::EntireSegment, SegmentOpinion::EntireSegment)); "same 00")]
    #[test_case((*C, *M), (*C, *M), Some((SegmentOpinion::EntireSegment, SegmentOpinion::EntireSegment)); "same 01")]
    #[test_case((*C, *M), (*M, *C), Some((SegmentOpinion::EntireSegment, SegmentOpinion::EntireSegment)); "same reverse 00")]
    #[test_case((*Y, *M), (*M, *Y), Some((SegmentOpinion::EntireSegment, SegmentOpinion::EntireSegment)); "same reverse 01")]
    #[test_case((*B, *E), (*C, *D), Some((SegmentOpinion::AlongSubsegment(Segment(*C, *D)), SegmentOpinion::EntireSegment)); "total collision")]
    #[test_case((*B, *E), (*D, *C), Some((SegmentOpinion::AlongSubsegment(Segment(*D, *C)), SegmentOpinion::EntireSegment)); "total collision, flip")]
    #[test_case((*B, *D), (*C, *E), Some((SegmentOpinion::AlongSubsegment(Segment(*C, *D)), SegmentOpinion::AlongSubsegment(Segment(*C, *D)))); "partial collision")]
    #[test_case((*B, *D), (*E, *C), Some((SegmentOpinion::AlongSubsegment(Segment(*C, *D)), SegmentOpinion::AlongSubsegment(Segment(*D, *C)))); "partial collision, flip")]
    #[test_case((*A, *C), (*C, *E), Some((SegmentOpinion::AtPointAlongSegment { at_point: *C, percent_along: Percent::One }, SegmentOpinion::AtPointAlongSegment { at_point: *C, percent_along: Percent::Zero })); "at point end to start")]
    #[test_case((*A, *C), (*E, *C), Some((SegmentOpinion::AtPointAlongSegment { at_point: *C, percent_along: Percent::One }, SegmentOpinion::AtPointAlongSegment { at_point: *C, percent_along: Percent::One })); "at point end to end")]
    #[test_case((*C, *A), (*E, *C), Some((SegmentOpinion::AtPointAlongSegment { at_point: *C, percent_along: Percent::Zero }, SegmentOpinion::AtPointAlongSegment { at_point: *C, percent_along: Percent::One })); "at point head to end")]
    #[test_case((*C, *A), (*C, *E), Some((SegmentOpinion::AtPointAlongSegment { at_point: *C, percent_along: Percent::Zero }, SegmentOpinion::AtPointAlongSegment { at_point: *C, percent_along: Percent::Zero })); "at point head to head")]
    #[test_case((*A, *E), (*A, *C), Some((SegmentOpinion::AlongSubsegment(Segment(*A, *C)), SegmentOpinion::EntireSegment)); "subsegment 00")]
    #[test_case((*A, *C), (*A, *E), Some((SegmentOpinion::EntireSegment, SegmentOpinion::AlongSubsegment(Segment(*A, *C)))); "subsegment 00, flip")]
    #[test_case((*A, *E), (*B, *D), Some((SegmentOpinion::AlongSubsegment(Segment(*B, *D)), SegmentOpinion::EntireSegment)); "subsegment 01")]
    #[test_case((*A, *E), (*C, *E), Some((SegmentOpinion::AlongSubsegment(Segment(*C, *E)), SegmentOpinion::EntireSegment)); "subsegment 02")]
    #[test_case((*C, *O), (*E, *M), Some((SegmentOpinion::AtPointAlongSegment { at_point: *I, percent_along: Percent::Val(0.5) }, SegmentOpinion::AtPointAlongSegment { at_point: *I, percent_along: Percent::Val(0.5) })); "crosshairs 00")]
    #[test_case((*O, *C), (*M, *E), Some((SegmentOpinion::AtPointAlongSegment { at_point: *I, percent_along: Percent::Val(0.5) }, SegmentOpinion::AtPointAlongSegment { at_point: *I, percent_along: Percent::Val(0.5) })); "crosshairs 01")]
    #[test_case((*C, *O), (*M, *E), Some((SegmentOpinion::AtPointAlongSegment { at_point: *I, percent_along: Percent::Val(0.5) }, SegmentOpinion::AtPointAlongSegment { at_point: *I, percent_along: Percent::Val(0.5) })); "crosshairs 02")]
    #[test_case((*O, *C), (*E, *M), Some((SegmentOpinion::AtPointAlongSegment { at_point: *I, percent_along: Percent::Val(0.5) }, SegmentOpinion::AtPointAlongSegment { at_point: *I, percent_along: Percent::Val(0.5) })); "crosshairs 03")]
    fn test_segment_overlaps_segment(
        a: impl Into<Segment>,
        b: impl Into<Segment>,
        expectation: Option<(SegmentOpinion, SegmentOpinion)>,
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

    #[test_case(Multiline([*A, *C, *E, *O, *Y]), *A, Some((nonempty![MultilineOpinion::AtPoint { index: 0, at_point: *A }], *A)); "multiline point at index 0")]
    #[test_case(Multiline([*A, *C, *E, *O, *Y]), *E, Some((nonempty![MultilineOpinion::AtPoint { index: 2, at_point: *E }], *E)); "multiline point at index 2")]
    #[test_case(Multiline([*A, *C, *E, *O, *Y]), *Y, Some((nonempty![MultilineOpinion::AtPoint { index: 4, at_point: *Y }], *Y)); "multiline point at index 4")]
    #[test_case(Multiline([*A, *C, *E, *O, *Y]), *B, Some((nonempty![MultilineOpinion::AtPointAlongSharedSegment { index: 0, at_point: *B, percent_along: Percent::Val(0.5) }], *B)); "multiline point along segment 0")]
    #[test_case(Multiline([*A, *C, *E, *O, *Y]), *D, Some((nonempty![MultilineOpinion::AtPointAlongSharedSegment { index: 1, at_point: *D, percent_along: Percent::Val(0.5) }], *D)); "multiline point along segment 1")]
    #[test_case(Multiline([*A, *C, *E, *O, *Y]), *J, Some((nonempty![MultilineOpinion::AtPointAlongSharedSegment { index: 2, at_point: *J, percent_along: Percent::Val(0.5) }], *J)); "multiline point along segment 2")]
    #[test_case(Multiline([*A, *C, *E, *O, *Y]), *T, Some((nonempty![MultilineOpinion::AtPointAlongSharedSegment { index: 3, at_point: *T, percent_along: Percent::Val(0.5) }], *T)); "multiline point along segment 3")]
    #[test_case(Multiline([*A, *C, *E, *O, *Y]), *M, None; "unrelated")]
    fn test_multiline_overlaps_point(
        multiline: Multiline,
        point: Point,
        expectation: Option<(NonEmpty<MultilineOpinion>, Point)>,
    ) -> Result<()> {
        assert_eq!(multiline_overlaps_point(&multiline, &point)?, expectation);
        Ok(())
    }

    #[test_case(Multiline([*C, *E, *O]), Segment(*M, *N), None; "none 00")]
    #[test_case(Multiline([*C, *E, *J]), Segment(*M, *N), None; "none 01")]
    #[test_case(Multiline([*C, *E, *O]), Segment(*H, *N), None; "none 02")]
    #[test_case(Multiline([*C, *I, *O]), Segment(*D, *J), None; "none 03")]
    #[test_case(Multiline([*C, *E, *O]), Segment(*C, *M), Some((nonempty![MultilineOpinion::AtPoint { index: 0, at_point: *C}], nonempty![SegmentOpinion::AtPointAlongSegment{ at_point: *C, percent_along: Percent::Zero}])); "at point at point 00")]
    #[test_case(Multiline([*E, *O, *M]), Segment(*E, *C), Some((nonempty![MultilineOpinion::AtPoint { index: 0, at_point: *E}], nonempty![SegmentOpinion::AtPointAlongSegment{ at_point: *E, percent_along: Percent::Zero}])); "at point at point 01")]
    #[test_case(Multiline([*O, *M, *C]), Segment(*O, *E), Some((nonempty![MultilineOpinion::AtPoint { index: 0, at_point: *O}], nonempty![SegmentOpinion::AtPointAlongSegment{ at_point: *O, percent_along: Percent::Zero}])); "at point at point 02")]
    #[test_case(Multiline([*C, *I, *O]), Segment(*C, *M), Some((nonempty![MultilineOpinion::AtPoint { index: 0, at_point: *C}], nonempty![SegmentOpinion::AtPointAlongSegment{ at_point: *C, percent_along: Percent::Zero}])); "at point at point 03")]
    #[test_case(Multiline([*C, *E, *O]), Segment(*M, *C), Some((nonempty![MultilineOpinion::AtPoint { index: 0, at_point: *C}], nonempty![SegmentOpinion::AtPointAlongSegment{ at_point: *C, percent_along: Percent::One}])); "at point at point 04")]
    #[test_case(Multiline([*E, *O, *M]), Segment(*C, *E), Some((nonempty![MultilineOpinion::AtPoint { index: 0, at_point: *E}], nonempty![SegmentOpinion::AtPointAlongSegment{ at_point: *E, percent_along: Percent::One}])); "at point at point 05")]
    #[test_case(Multiline([*O, *M, *C]), Segment(*E, *O), Some((nonempty![MultilineOpinion::AtPoint { index: 0, at_point: *O}], nonempty![SegmentOpinion::AtPointAlongSegment{ at_point: *O, percent_along: Percent::One}])); "at point at point 06")]
    #[test_case(Multiline([*C, *I, *O]), Segment(*M, *C), Some((nonempty![MultilineOpinion::AtPoint { index: 0, at_point: *C}], nonempty![SegmentOpinion::AtPointAlongSegment{ at_point: *C, percent_along: Percent::One}])); "at point at point 07")]
    #[test_case(Multiline([*D, *I, *N]), Segment(*C, *E), Some((nonempty![MultilineOpinion::AtPoint { index: 0, at_point: *D}], nonempty![SegmentOpinion::AtPointAlongSegment{ at_point: *D, percent_along: Percent::Val(0.5)}])); "at point at point 08")]
    #[test_case(Multiline([*H, *I, *J]), Segment(*M, *C), Some((nonempty![MultilineOpinion::AtPoint { index: 0, at_point: *H}], nonempty![SegmentOpinion::AtPointAlongSegment{ at_point: *H, percent_along: Percent::Val(0.5)}])); "at point at point 09")]
    #[test_case(Multiline([*D, *I, *N]), Segment(*I, *J), Some((nonempty![MultilineOpinion::AtPoint { index: 1, at_point: *I}], nonempty![SegmentOpinion::AtPointAlongSegment{ at_point: *I, percent_along: Percent::Zero}])); "at point at point 10")]
    #[test_case(Multiline([*D, *I, *N]), Segment(*H, *I), Some((nonempty![MultilineOpinion::AtPoint { index: 1, at_point: *I}], nonempty![SegmentOpinion::AtPointAlongSegment{ at_point: *I, percent_along: Percent::One}])); "at point at point 11")]
    #[test_case(Multiline([*D, *I, *N]), Segment(*H, *J), Some((nonempty![MultilineOpinion::AtPoint { index: 1, at_point: *I}], nonempty![SegmentOpinion::AtPointAlongSegment{ at_point: *I, percent_along: Percent::Val(0.5)}])); "at point at point 12")]
    #[test_case(Multiline([*D, *I, *N]), Segment(*N, *O), Some((nonempty![MultilineOpinion::AtPoint { index: 2, at_point: *N}], nonempty![SegmentOpinion::AtPointAlongSegment{ at_point: *N, percent_along: Percent::Zero}])); "at point at point 13")]
    #[test_case(Multiline([*D, *I, *N]), Segment(*M, *N), Some((nonempty![MultilineOpinion::AtPoint { index: 2, at_point: *N}], nonempty![SegmentOpinion::AtPointAlongSegment{ at_point: *N, percent_along: Percent::One}])); "at point at point 14")]
    #[test_case(Multiline([*D, *I, *N]), Segment(*M, *O), Some((nonempty![MultilineOpinion::AtPoint { index: 2, at_point: *N}], nonempty![SegmentOpinion::AtPointAlongSegment{ at_point: *N, percent_along: Percent::Val(0.5)}])); "at point at point 15")]
    #[test_case( Multiline([*C, *E, *O]), Segment(*C, *O), Some(( nonempty![ MultilineOpinion::AtPoint { index: 0, at_point: *C, }, MultilineOpinion::AtPoint { index: 2, at_point: *O, } ], nonempty![ SegmentOpinion::AtPointAlongSegment { at_point: *C, percent_along: Percent::Zero, }, SegmentOpinion::AtPointAlongSegment { at_point: *O, percent_along: Percent::One, } ])); "segment bookends 1")]
    #[test_case( Multiline([*C, *E, *O]), Segment(*D, *J), Some(( nonempty![ MultilineOpinion::AtPointAlongSharedSegment { index: 0, at_point: *D, percent_along: Percent::Val(0.5), }, MultilineOpinion::AtPointAlongSharedSegment { index: 1, at_point: *J, percent_along: Percent::Val(0.5), } ], nonempty![ SegmentOpinion::AtPointAlongSegment { at_point: *D, percent_along: Percent::Zero, }, SegmentOpinion::AtPointAlongSegment { at_point: *J, percent_along: Percent::One, } ],)); "segment bookends 2")]
    #[test_case( Multiline([*C, *D, *E]), Segment(*C, *D), Some(( nonempty![ MultilineOpinion::EntireSubsegment { index: 0 } ], nonempty![ SegmentOpinion::EntireSegment ])); "partial collision")]
    #[test_case( Multiline([*C, *D, *E]), Segment(*D, *C), Some(( nonempty![ MultilineOpinion::EntireSubsegment { index: 0 } ], nonempty![ SegmentOpinion::EntireSegment ])); "partial collision 02")]
    #[test_case( Multiline([*C, *D, *E]), Segment(*D, *E), Some(( nonempty![ MultilineOpinion::EntireSubsegment { index: 1 } ], nonempty![ SegmentOpinion::EntireSegment ])); "partial collision 03")]
    #[test_case( Multiline([*C, *D, *E]), Segment(*E, *D), Some(( nonempty![ MultilineOpinion::EntireSubsegment { index: 1 } ], nonempty![ SegmentOpinion::EntireSegment ])); "partial collision 04")]
    #[test_case( Multiline([*C, *D, *E]), Segment(*C, *E), Some(( nonempty![ MultilineOpinion::EntireSubsegment { index: 0 }, MultilineOpinion::EntireSubsegment { index: 1 } ], nonempty![ SegmentOpinion::EntireSegment ])); "total collision 01")]
    #[test_case( Multiline([*C, *D, *E]), Segment(*E, *C), Some(( nonempty![ MultilineOpinion::EntireSubsegment { index: 0 }, MultilineOpinion::EntireSubsegment { index: 1 } ], nonempty![ SegmentOpinion::EntireSegment ])); "total collision 01 flip")]
    #[test_case( Multiline([*C, *D, *E]), Segment(Point(0.5,2), Point(1.5,2)), Some(( nonempty![ MultilineOpinion::AlongSubsegmentOf { index: 0, subsegment: Segment(Point(0.5,2), Point(1,2)) }, MultilineOpinion::AlongSubsegmentOf { index: 1, subsegment: Segment(Point(1,2), Point(1.5,2)) } ], nonempty![ SegmentOpinion::EntireSegment ])); "total collision half shift 01")]
    #[test_case(Multiline([*C, *D, *E]), Segment(Point(1.5,2), Point(0.5,2)), Some(( nonempty![ MultilineOpinion::AlongSubsegmentOf { index: 0, subsegment: Segment(Point(0.5,2), Point(1,2)) }, MultilineOpinion::AlongSubsegmentOf { index: 1, subsegment: Segment(Point(1,2), Point(1.5,2)) } ], nonempty![ SegmentOpinion::EntireSegment ])); "total collision half shift 01 flip")]
    #[test_case(Multiline([*H, *J, *O]), Segment(*D, *N), Some((nonempty![MultilineOpinion::AtPointAlongSharedSegment{ index: 0, at_point: *I, percent_along: Percent::Val(0.5)}], nonempty![SegmentOpinion::AtPointAlongSegment{ at_point: *I, percent_along: Percent::Val(0.5) }])); "at point on segment at point on segment 00")]
    #[test_case(Multiline([*H, *J, *O]), Segment(*I, *N), Some((nonempty![MultilineOpinion::AtPointAlongSharedSegment{ index: 0, at_point: *I, percent_along: Percent::Val(0.5)}], nonempty![SegmentOpinion::AtPointAlongSegment{ at_point: *I, percent_along: Percent::Zero }])); "at point on segment at point on segment 01")]
    #[test_case(Multiline([*H, *J, *O]), Segment(*D, *I), Some((nonempty![MultilineOpinion::AtPointAlongSharedSegment{ index: 0, at_point: *I, percent_along: Percent::Val(0.5)}], nonempty![SegmentOpinion::AtPointAlongSegment{ at_point: *I, percent_along: Percent::One }])); "at point on segment at point on segment 02")]
    #[test_case(Multiline([*M, *H, *J]), Segment(*D, *N), Some((nonempty![MultilineOpinion::AtPointAlongSharedSegment{ index: 1, at_point: *I, percent_along: Percent::Val(0.5)}], nonempty![SegmentOpinion::AtPointAlongSegment{ at_point: *I, percent_along: Percent::Val(0.5) }])); "at point on segment at point on segment 03")]
    #[test_case(Multiline([*M, *H, *J]), Segment(*I, *N), Some((nonempty![MultilineOpinion::AtPointAlongSharedSegment{ index: 1, at_point: *I, percent_along: Percent::Val(0.5)}], nonempty![SegmentOpinion::AtPointAlongSegment{ at_point: *I, percent_along: Percent::Zero }])); "at point on segment at point on segment 04")]
    #[test_case(Multiline([*M, *H, *J]), Segment(*D, *I), Some((nonempty![MultilineOpinion::AtPointAlongSharedSegment{ index: 1, at_point: *I, percent_along: Percent::Val(0.5)}], nonempty![SegmentOpinion::AtPointAlongSegment{ at_point: *I, percent_along: Percent::One }])); "at point on segment at point on segment 05")]
    fn test_multiline_overlaps_segment(
        ml: Multiline,
        sg: Segment,
        expectation: Option<(NonEmpty<MultilineOpinion>, NonEmpty<SegmentOpinion>)>,
    ) -> Result<()> {
        assert_eq!(multiline_overlaps_segment(&ml, &sg)?, expectation);
        Ok(())
    }

    #[test_case(Multiline([*C, *D, *E]), Multiline([*H, *I, *J]), None; "none 01")]
    #[test_case(Multiline([*C, *D, *E]), Multiline([*M, *N, *O]), None; "none 02")]
    #[test_case(Multiline([*C, *I, *O]), Multiline([*D, *J]), None; "none diagonal")]
    #[test_case(Multiline([*C, *D, *E]), Multiline([*C, *H, *M]), Some( ( nonempty![ MultilineOpinion::AtPoint { index: 0, at_point: *C }, ], nonempty![ MultilineOpinion::AtPoint { index: 0, at_point: *C } ])); "AtPoint 0, AtPoint 0")]
    #[test_case( Multiline([*C, *D, *E]), Multiline([*M, *H, *C]), Some( ( nonempty![ MultilineOpinion::AtPoint { index: 0, at_point: *C }, ], nonempty![ MultilineOpinion::AtPoint { index: 2, at_point: *C } ])); "AtPoint 0, AtPoint 2")]
    #[test_case( Multiline([*E, *D, *C]), Multiline([*M, *H, *C]), Some( ( nonempty![ MultilineOpinion::AtPoint { index: 2, at_point: *C }, ], nonempty![ MultilineOpinion::AtPoint { index: 2, at_point: *C } ])); "AtPoint 2, AtPoint 2")]
    #[test_case( Multiline([*C, *I, *O]), Multiline([*M, *I, *E]), Some( ( nonempty![ MultilineOpinion::AtPoint { index: 1, at_point: *I }, ], nonempty![ MultilineOpinion::AtPoint { index: 1, at_point: *I } ])); "AtPoint 1, AtPoint 1")]
    #[test_case( Multiline([*C, *O]), Multiline([*E, *M]), Some( ( nonempty![ MultilineOpinion::AtPointAlongSharedSegment { index: 0, at_point: *I, percent_along: Percent::Val(0.5) } ], nonempty![ MultilineOpinion::AtPointAlongSharedSegment { index: 0, at_point: *I, percent_along: Percent::Val(0.5) } ])); "crosshairs")]
    #[test_case( Multiline([*C, *D, *E]), Multiline([*C, *D, *I]), Some( ( nonempty![ MultilineOpinion::EntireSubsegment { index: 0 }, ], nonempty![ MultilineOpinion::EntireSubsegment { index: 0 }, ])); "partial collision, entire subsegment 0 0")]
    #[test_case( Multiline([*E, *D, *C]), Multiline([*I, *D, *C]), Some( ( nonempty![ MultilineOpinion::EntireSubsegment { index: 1 }, ], nonempty![ MultilineOpinion::EntireSubsegment { index: 1 }, ])); "partial collision, entire subsegment 1 1")]
    #[test_case( Multiline([*C, *D, *E]), Multiline([*D, *E, *J]), Some( ( nonempty![ MultilineOpinion::EntireSubsegment { index: 1 }, ], nonempty![ MultilineOpinion::EntireSubsegment { index: 0 }, ])); "partial collision, entire subsegment 1 0")]
    #[test_case( Multiline([*C, *D, *E]), Multiline([*E, *D, *C]), Some( ( nonempty![ MultilineOpinion::EntireSubsegment { index: 0 }, MultilineOpinion::EntireSubsegment { index: 1 } ], nonempty![ MultilineOpinion::EntireSubsegment { index: 1 }, MultilineOpinion::EntireSubsegment { index: 0 } ])); "partial collision, entire subsegment 01 01 flipped")]
    #[test_case( Multiline([*C, *D, *E, *J, *O]), Multiline([*C, *D, *I, *J, *O]), Some( ( nonempty![ MultilineOpinion::EntireSubsegment { index: 0 }, MultilineOpinion::EntireSubsegment { index: 3 } ], nonempty![ MultilineOpinion::EntireSubsegment { index: 0 }, MultilineOpinion::EntireSubsegment { index: 3 } ])); "shared segment, then diversion, then another shared segment")]
    #[test_case( Multiline([*C, *D, *E, *J, *O]), Multiline([*C, *D, *I, *J]), Some( ( nonempty![ MultilineOpinion::EntireSubsegment { index: 0 }, MultilineOpinion::AtPoint { index: 3, at_point: *J } ], nonempty![ MultilineOpinion::EntireSubsegment { index: 0 }, MultilineOpinion::AtPoint { index: 3, at_point: *J } ])); "shared segment, then diversion, then atpoint")]
    fn test_multiline_overlaps_multiline(
        ml1: Multiline,
        ml2: Multiline,
        expectation: Option<(NonEmpty<MultilineOpinion>, NonEmpty<MultilineOpinion>)>,
    ) -> Result<()> {
        assert_eq!(multiline_overlaps_multiline(&ml1, &ml2)?, expectation);
        Ok(())
    }

    #[test_case(Polygon([*D, *H, *N, *J]), &C, None; "point not in polygon 00")]
    #[test_case(Polygon([*D, *H, *N, *J]), &E, None; "point not in polygon 01")]
    #[test_case(Polygon([*D, *H, *N, *J]), &I, Some((PolygonOpinion::WithinArea, *I)); "point in polygon")]
    #[test_case(Polygon([*D, *H, *N, *J]), &D, Some((PolygonOpinion::AtPoint{index:0, at_point: *D}, *D)); "point at point of polygon 00")]
    #[test_case(Polygon([*D, *H, *N, *J]), &H, Some((PolygonOpinion::AtPoint{index:1, at_point: *H}, *H)); "point at point of polygon 01")]
    #[test_case(Polygon([*D, *H, *N, *J]), &N, Some((PolygonOpinion::AtPoint{index:2, at_point: *N}, *N)); "point at point of polygon 02")]
    #[test_case(Polygon([*D, *H, *N, *J]), &J, Some((PolygonOpinion::AtPoint{index:3, at_point: *J}, *J)); "point at point of polygon 03")]
    #[test_case(Polygon([*C, *M, *O, *E]), &H, Some((PolygonOpinion::AlongEdge{ index: 0, at_point: *H, percent_along: Percent::Val(0.5) }, *H)); "point at edge of polygon 00")]
    #[test_case(Polygon([*C, *M, *O, *E]), &N, Some((PolygonOpinion::AlongEdge{ index: 1, at_point: *N, percent_along: Percent::Val(0.5) }, *N)); "point at edge of polygon 01")]
    #[test_case(Polygon([*C, *M, *O, *E]), &J, Some((PolygonOpinion::AlongEdge{ index: 2, at_point: *J, percent_along: Percent::Val(0.5) }, *J)); "point at edge of polygon 02")]
    #[test_case(Polygon([*C, *M, *O, *E]), &D, Some((PolygonOpinion::AlongEdge{ index: 3, at_point: *D, percent_along: Percent::Val(0.5) }, *D)); "point at edge of polygon 03")]
    fn test_polygon_overlaps_point(
        pg: Result<Polygon>,
        pt: &Point,
        expectation: Option<(PolygonOpinion, Point)>,
    ) -> Result<()> {
        assert_eq!(polygon_overlaps_point(&pg?, pt)?, expectation);
        Ok(())
    }

    mod pg_sg {
        use super::*;
        use test_case::test_case;

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

        fn overlaps(
            pg: Result<Polygon>,
            sg: Segment,
            expectation: Option<(NonEmpty<PolygonOpinion>, NonEmpty<SegmentOpinion>)>,
        ) -> Result<()> {
            Ok(())
        }
    }
}
