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

    mod pt_pt {
        use super::*;
        use test_case::test_case;

        #[test_case(*C, *C, Some(*C))]
        #[test_case(*D, *D, Some(*D))]
        #[test_case(*D, *H, None)]
        #[test_case(*A, *B, None)]
        fn overlaps(a: Point, b: Point, expectation: Option<Point>) -> Result<()> {
            assert_eq!(point_overlaps_point(&a, &b)?, expectation);
            Ok(())
        }
    }

    mod sg_pt {
        use super::*;

        #[test]
        fn at_start_or_end() -> Result<()> {
            for (i, f) in &[(*C, *D), (*C, *I), (*C, *M)] {
                assert_eq!(
                    segment_overlaps_point(&Segment(*i, *f), i)?,
                    Some((
                        SegmentOpinion::AtPointAlongSegment {
                            at_point: *i,
                            percent_along: Percent::Zero
                        },
                        *i
                    ))
                );
                assert_eq!(
                    segment_overlaps_point(&Segment(*i, *f), f)?,
                    Some((
                        SegmentOpinion::AtPointAlongSegment {
                            at_point: *f,
                            percent_along: Percent::One
                        },
                        *f
                    ))
                );
            }
            Ok(())
        }

        #[test]
        fn halfway_along() -> Result<()> {
            for (i, m, f) in &[(*C, *D, *E), (*C, *I, *O), (*C, *H, *M)] {
                assert_eq!(
                    segment_overlaps_point(&Segment(*i, *f), m)?,
                    Some((
                        SegmentOpinion::AtPointAlongSegment {
                            at_point: *m,
                            percent_along: Percent::Val(0.5)
                        },
                        *m
                    ))
                );
            }
            Ok(())
        }
    }

    mod sg_sg {
        use super::*;
        use test_case::test_case;

        #[test]
        fn the_same() -> Result<()> {
            for i in &[*C, *D, *E] {
                for j in &[*H, *I, *J] {
                    assert_eq!(
                        segment_overlaps_segment(&Segment(*i, *j), &Segment(*i, *j))?,
                        Some((SegmentOpinion::EntireSegment, SegmentOpinion::EntireSegment,))
                    );
                }
            }
            Ok(())
        }

        #[test]
        fn the_same_but_reversed() -> Result<()> {
            for i in &[*C, *D, *E] {
                for j in &[*H, *I, *J] {
                    assert_eq!(
                        segment_overlaps_segment(&Segment(*i, *j), &Segment(*j, *i))?,
                        Some((SegmentOpinion::EntireSegment, SegmentOpinion::EntireSegment))
                    );
                }
            }
            Ok(())
        }

        mod colinear {
            use super::*;
            use test_case::test_case;

            //   ^
            //   |
            // --Q--W--E--R--T-->
            //   |
            lazy_static! {
                static ref Q: Point = Point(0, 0);
                static ref W: Point = Point(1, 0);
                static ref E: Point = Point(2, 0);
                static ref R: Point = Point(3, 0);
                static ref T: Point = Point(4, 0);
            }

            #[test_case(Segment(*Q, *E),Percent::One, Segment(*E, *T), Percent::Zero, *E)]
            #[test_case(Segment(*E, *Q),Percent::Zero, Segment(*E, *T), Percent::Zero, *E)]
            #[test_case(Segment(*E, *Q),Percent::Zero, Segment(*T, *E), Percent::One, *E)]
            #[test_case(Segment(*Q, *E),Percent::One, Segment(*T, *E), Percent::One, *E)]

            fn atpoint(
                sga: Segment,
                a_pct: Percent,
                sgb: Segment,
                b_pct: Percent,
                at_point: Point,
            ) -> Result<()> {
                assert_eq!(
                    segment_overlaps_segment(&sga, &sgb)?,
                    Some((
                        SegmentOpinion::AtPointAlongSegment {
                            at_point,
                            percent_along: a_pct,
                        },
                        SegmentOpinion::AtPointAlongSegment {
                            at_point,
                            percent_along: b_pct,
                        },
                    ))
                );

                Ok(())
            }

            #[test_case(Segment(*Q, *T), Segment(*Q, *E), Segment(*Q, *E))]
            #[test_case(Segment(*Q, *T), Segment(*W, *R), Segment(*W, *R))]
            #[test_case(Segment(*Q, *T), Segment(*E, *T), Segment(*E, *T))]
            fn atsubsegment(sga: Segment, sgb: Segment, subsegment: Segment) -> Result<()> {
                assert_eq!(
                    segment_overlaps_segment(&sga, &sgb)?,
                    Some((
                        SegmentOpinion::AlongSubsegment(subsegment),
                        SegmentOpinion::EntireSegment,
                    ))
                );
                assert_eq!(
                    segment_overlaps_segment(&sgb, &sga)?,
                    Some((
                        SegmentOpinion::EntireSegment,
                        SegmentOpinion::AlongSubsegment(subsegment),
                    ))
                );

                Ok(())
            }
        }

        #[test]
        fn partway() -> Result<()> {
            {
                // given two non-colinear segments,
                let (p0, p1) = (*C, *D);
                for p2 in &[*H, *I, *J, *M, *N, *O] {
                    assert_eq!(
                        segment_overlaps_segment(&Segment(p0, p1), &Segment(p1, *p2))?,
                        Some((
                            SegmentOpinion::AtPointAlongSegment {
                                at_point: p1,
                                percent_along: Percent::One
                            },
                            SegmentOpinion::AtPointAlongSegment {
                                at_point: p1,
                                percent_along: Percent::Zero
                            },
                        ))
                    );
                }
            }

            {
                // midpoints
                let sa = Segment(*C, *O);
                let sb = Segment(*E, *M);
                for (sa, sb) in &[
                    (sa, sb),
                    (sa, sb.flip()),
                    (sa.flip(), sb),
                    (sa.flip(), sb.flip()),
                ] {
                    assert_eq!(
                        segment_overlaps_segment(sa, sb)?,
                        Some((
                            SegmentOpinion::AtPointAlongSegment {
                                at_point: *I,
                                percent_along: Percent::Val(0.5)
                            },
                            SegmentOpinion::AtPointAlongSegment {
                                at_point: *I,
                                percent_along: Percent::Val(0.5)
                            }
                        ))
                    );
                }
            }

            Ok(())
        }

        #[test_case(
            Segment(*C, *E),
            Segment(Point(0.5, 2), Point(1.5,2)),
            Some((
                    SegmentOpinion::AlongSubsegment(Segment(Point(0.5,2), Point(1.5, 2))),
                    SegmentOpinion::EntireSegment
            ));
            "partial collision"
        )]
        #[test_case(
            Segment(*C, *E),
            Segment(Point(1.5, 2), Point(0.5,2)),
            Some((
                    SegmentOpinion::AlongSubsegment(Segment(Point(1.5,2), Point(0.5, 2))),
                    SegmentOpinion::EntireSegment
            ));
            "partial collision, flip"
        )]
        #[test_case(
            Segment(Point(0,2), Point(1,2)),
            Segment(Point(1.5,2), Point(0.5,2)),
            Some((
                    SegmentOpinion::AlongSubsegment(Segment(Point(0.5,2), Point(1,2))),
                    SegmentOpinion::AlongSubsegment(Segment(Point(1,2), Point(0.5,2))),
            ));
            "partial collision, backwards"
        )]
        fn isxn(
            a: Segment,
            b: Segment,
            expectation: Option<(SegmentOpinion, SegmentOpinion)>,
        ) -> Result<()> {
            assert_eq!(segment_overlaps_segment(&a, &b)?, expectation);
            Ok(())
        }
    }

    mod ml_pt {
        use super::*;

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
        #[test]
        fn test_along_two_segment_multiline() -> Result<()> {
            for ((start, midpoint1, pivot, midpoint2, end), unrelated) in &[
                ((*M, *N, *O, *J, *E), *C),
                ((*M, *H, *C, *D, *E), *O),
                ((*E, *D, *C, *H, *M), *O),
                ((*M, *I, *E, *D, *C), *H),
            ] {
                let ml = Multiline([*start, *pivot, *end]);

                // check points

                for (pt, idx) in [(start, 0), (pivot, 1), (end, 2)] {
                    assert_eq!(
                        multiline_overlaps_point(&ml, &pt)?,
                        Some((
                            nonempty![MultilineOpinion::AtPoint {
                                index: idx,
                                at_point: *pt
                            }],
                            *pt,
                        ))
                    );
                }

                // check segments

                for (pt, idx) in [(midpoint1, 0), (midpoint2, 1)] {
                    assert_eq!(
                        multiline_overlaps_point(&ml, &pt)?,
                        Some((
                            nonempty![MultilineOpinion::AtPointAlongSharedSegment {
                                index: idx,
                                at_point: *pt,
                                percent_along: Percent::Val(0.5)
                            }],
                            *pt,
                        )),
                    );
                }

                assert_eq!(multiline_overlaps_point(&ml, unrelated)?, None);
            }

            Ok(())
        }
    }

    mod ml_sg {
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

        #[test_case(Multiline([*C, *E, *O]), Segment(*M, *N))]
        #[test_case(Multiline([*C, *E, *J]), Segment(*M, *N))]
        #[test_case(Multiline([*C, *E, *O]), Segment(*H, *N))]
        #[test_case(Multiline([*C, *I, *O]), Segment(*D, *J))]
        fn none(ml: Multiline, sg: Segment) -> Result<()> {
            assert_eq!(multiline_overlaps_segment(&ml, &sg)?, None);
            Ok(())
        }

        #[test_case(Multiline([*C, *E, *O]), Segment(*C, *M), 0, *C, Percent::Zero)]
        #[test_case(Multiline([*E, *O, *M]), Segment(*E, *C), 0, *E, Percent::Zero)]
        #[test_case(Multiline([*O, *M, *C]), Segment(*O, *E), 0, *O, Percent::Zero)]
        #[test_case(Multiline([*C, *I, *O]), Segment(*C, *M), 0, *C, Percent::Zero)]
        #[test_case(Multiline([*C, *E, *O]), Segment(*M, *C), 0, *C, Percent::One)]
        #[test_case(Multiline([*E, *O, *M]), Segment(*C, *E), 0, *E, Percent::One)]
        #[test_case(Multiline([*O, *M, *C]), Segment(*E, *O), 0, *O, Percent::One)]
        #[test_case(Multiline([*C, *I, *O]), Segment(*M, *C), 0, *C, Percent::One)]
        #[test_case(Multiline([*D, *I, *N]), Segment(*C, *E), 0, *D, Percent::Val(0.5))]
        #[test_case(Multiline([*H, *I, *J]), Segment(*M, *C), 0, *H, Percent::Val(0.5))]
        #[test_case(Multiline([*D, *I, *N]), Segment(*I, *J), 1, *I, Percent::Zero)]
        #[test_case(Multiline([*D, *I, *N]), Segment(*H, *I), 1, *I, Percent::One)]
        #[test_case(Multiline([*D, *I, *N]), Segment(*H, *J), 1, *I, Percent::Val(0.5))]
        #[test_case(Multiline([*D, *I, *N]), Segment(*N, *O), 2, *N, Percent::Zero)]
        #[test_case(Multiline([*D, *I, *N]), Segment(*M, *N), 2, *N, Percent::One)]
        #[test_case(Multiline([*D, *I, *N]), Segment(*M, *O), 2, *N, Percent::Val(0.5))]
        // At segment midpoint
        fn one_overlap_ml_atpoint_sg(
            ml: Multiline,
            sg: Segment,
            index: usize,
            at_point: Point,
            percent_along: Percent,
        ) -> Result<()> {
            assert_eq!(
                multiline_overlaps_segment(&ml, &sg)?,
                Some((
                    nonempty![MultilineOpinion::AtPoint { index, at_point }],
                    nonempty![SegmentOpinion::AtPointAlongSegment {
                        at_point,
                        percent_along,
                    }]
                ))
            );
            Ok(())
        }

        #[test_case(Multiline([*H, *J, *O]), Segment(*D, *N), 0, *I, Percent::Val(0.5), Percent::Val(0.5))]
        #[test_case(Multiline([*H, *J, *O]), Segment(*I, *N), 0, *I, Percent::Val(0.5), Percent::Zero)]
        #[test_case(Multiline([*H, *J, *O]), Segment(*D, *I), 0, *I, Percent::Val(0.5), Percent::One)]
        #[test_case(Multiline([*M, *H, *J]), Segment(*D, *N), 1, *I, Percent::Val(0.5), Percent::Val(0.5))]
        #[test_case(Multiline([*M, *H, *J]), Segment(*I, *N), 1, *I, Percent::Val(0.5), Percent::Zero)]
        #[test_case(Multiline([*M, *H, *J]), Segment(*D, *I), 1, *I, Percent::Val(0.5), Percent::One)]
        fn one_overlap_ml_alongsharedsegment_sg(
            ml: Multiline,
            sg: Segment,
            index: usize,
            at_point: Point,
            ml_pct_along: Percent,
            sg_pct_along: Percent,
        ) -> Result<()> {
            assert_eq!(
                multiline_overlaps_segment(&ml, &sg)?,
                Some((
                    nonempty![MultilineOpinion::AtPointAlongSharedSegment {
                        index,
                        at_point,
                        percent_along: ml_pct_along,
                    }],
                    nonempty![SegmentOpinion::AtPointAlongSegment {
                        at_point,
                        percent_along: sg_pct_along,
                    }]
                ))
            );
            Ok(())
        }

        #[test_case(
            Multiline([*C, *E, *O]),
            Segment(*C, *O),
            Some((
                nonempty![
                    MultilineOpinion::AtPoint {
                        index: 0,
                        at_point: *C,
                    },
                    MultilineOpinion::AtPoint {
                        index: 2,
                        at_point: *O,
                    }
                ],
                nonempty![
                    SegmentOpinion::AtPointAlongSegment {
                        at_point: *C,
                        percent_along: Percent::Zero,
                    },
                    SegmentOpinion::AtPointAlongSegment {
                        at_point: *O,
                        percent_along: Percent::One,
                    }
                ]
            ));
            "segment bookends 1"
        )]
        #[test_case(
            Multiline([*C, *E, *O]),
            Segment(*D, *J),
            Some((
                nonempty![
                    MultilineOpinion::AtPointAlongSharedSegment {
                        index: 0,
                        at_point: *D,
                        percent_along: Percent::Val(0.5),
                    },
                    MultilineOpinion::AtPointAlongSharedSegment {
                        index: 1,
                        at_point: *J,
                        percent_along: Percent::Val(0.5),
                    }
                ],
                nonempty![
                    SegmentOpinion::AtPointAlongSegment {
                        at_point: *D,
                        percent_along: Percent::Zero,
                    },
                    SegmentOpinion::AtPointAlongSegment {
                        at_point: *J,
                        percent_along: Percent::One,
                    }
                ],
            ));
            "segment bookends 2"
        )]
        #[test_case(
            Multiline([*C, *D, *E]),
            Segment(*C, *D),
            Some((
                nonempty![ MultilineOpinion::EntireSubsegment { index: 0 } ],
                nonempty![ SegmentOpinion::EntireSegment ]
            ));
            "partial collision"
        )]
        #[test_case(
            Multiline([*C, *D, *E]),
            Segment(*D, *C),
            Some((
                nonempty![ MultilineOpinion::EntireSubsegment { index: 0 } ],
                nonempty![ SegmentOpinion::EntireSegment ]
            ));
            "partial collision 02"
        )]
        #[test_case(
            Multiline([*C, *D, *E]),
            Segment(*D, *E),
            Some((
                nonempty![ MultilineOpinion::EntireSubsegment { index: 1 } ],
                nonempty![ SegmentOpinion::EntireSegment ]
            ));
            "partial collision 03"
        )]
        #[test_case(
            Multiline([*C, *D, *E]),
            Segment(*E, *D),
            Some((
                nonempty![ MultilineOpinion::EntireSubsegment { index: 1 } ],
                nonempty![ SegmentOpinion::EntireSegment ]
            ));
            "partial collision 04"
        )]
        #[test_case(
            Multiline([*C, *D, *E]),
            Segment(*C, *E),
            Some((
                nonempty![
                    MultilineOpinion::EntireSubsegment { index: 0 },
                    MultilineOpinion::EntireSubsegment { index: 1 }
                ],
                nonempty![ SegmentOpinion::EntireSegment ]
            ));
            "total collision 01"
        )]
        #[test_case(
            Multiline([*C, *D, *E]),
            Segment(*E, *C),
            Some((
                nonempty![
                    MultilineOpinion::EntireSubsegment { index: 0 },
                    MultilineOpinion::EntireSubsegment { index: 1 }
                ],
                nonempty![ SegmentOpinion::EntireSegment ]
            ));
            "total collision 01 flip"
        )]
        #[test_case(
            Multiline([*C, *D, *E]),
            Segment(Point(0.5,2), Point(1.5,2)),
            Some((
                nonempty![
                    MultilineOpinion::AlongSubsegmentOf {
                        index: 0,
                        subsegment: Segment(Point(0.5,2), Point(1,2))
                    },
                    MultilineOpinion::AlongSubsegmentOf {
                        index: 1,
                        subsegment: Segment(Point(1,2), Point(1.5,2))
                    }
                ],
                nonempty![ SegmentOpinion::EntireSegment ]
            ));
            "total collision half shift 01"
        )]
        #[test_case(
            Multiline([*C, *D, *E]),
            Segment(Point(1.5,2), Point(0.5,2)),
            Some((
                nonempty![
                    MultilineOpinion::AlongSubsegmentOf {
                        index: 0,
                        subsegment: Segment(Point(0.5,2), Point(1,2))
                    },
                    MultilineOpinion::AlongSubsegmentOf {
                        index: 1,
                        subsegment: Segment(Point(1,2), Point(1.5,2))
                    }
                ],
                nonempty![ SegmentOpinion::EntireSegment ]
            ));
            "total collision half shift 01 flip"
        )]
        fn isxn(
            ml: Multiline,
            sg: Segment,
            expectation: Option<(NonEmpty<MultilineOpinion>, NonEmpty<SegmentOpinion>)>,
        ) -> Result<()> {
            assert_eq!(multiline_overlaps_segment(&ml, &sg)?, expectation);
            Ok(())
        }
    }

    mod ml_ml {
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

        #[test_case(Multiline([*C, *D, *E]), Multiline([*H, *I, *J]), None; "none 01")]
        #[test_case(Multiline([*C, *D, *E]), Multiline([*M, *N, *O]), None; "none 02")]
        #[test_case(Multiline([*C, *I, *O]), Multiline([*D, *J]), None; "none diagonal")]
        #[test_case(
            Multiline([*C, *D, *E]),
            Multiline([*C, *H, *M]),
            Some( (
                nonempty![ MultilineOpinion::AtPoint { index: 0, at_point: *C }, ],
                nonempty![ MultilineOpinion::AtPoint { index: 0, at_point: *C } ]));
            "AtPoint 0, AtPoint 0"
        )]
        #[test_case(
            Multiline([*C, *D, *E]),
            Multiline([*M, *H, *C]),
            Some( (
                nonempty![ MultilineOpinion::AtPoint { index: 0, at_point: *C }, ],
                nonempty![ MultilineOpinion::AtPoint { index: 2, at_point: *C } ]));
            "AtPoint 0, AtPoint 2"
        )]
        #[test_case(
            Multiline([*E, *D, *C]),
            Multiline([*M, *H, *C]),
            Some( (
                nonempty![ MultilineOpinion::AtPoint { index: 2, at_point: *C }, ],
                nonempty![ MultilineOpinion::AtPoint { index: 2, at_point: *C } ]));
            "AtPoint 2, AtPoint 2"
        )]
        #[test_case(
            Multiline([*C, *I, *O]),
            Multiline([*M, *I, *E]),
            Some( (
                nonempty![ MultilineOpinion::AtPoint { index: 1, at_point: *I }, ],
                nonempty![ MultilineOpinion::AtPoint { index: 1, at_point: *I } ]));
            "AtPoint 1, AtPoint 1"
        )]
        #[test_case(
            Multiline([*C, *O]),
            Multiline([*E, *M]),
            Some( (
                nonempty![ MultilineOpinion::AtPointAlongSharedSegment { index: 0, at_point: *I, percent_along: Percent::Val(0.5) } ],
                nonempty![ MultilineOpinion::AtPointAlongSharedSegment { index: 0, at_point: *I, percent_along: Percent::Val(0.5) } ]));
            "crosshairs"
        )]
        #[test_case(
            Multiline([*C, *D, *E]),
            Multiline([*C, *D, *I]),
            Some( (
                nonempty![ MultilineOpinion::EntireSubsegment { index: 0 }, ],
                nonempty![ MultilineOpinion::EntireSubsegment { index: 0 }, ]));
            "partial collision, entire subsegment 0 0"
        )]
        #[test_case(
            Multiline([*E, *D, *C]),
            Multiline([*I, *D, *C]),
            Some( (
                nonempty![ MultilineOpinion::EntireSubsegment { index: 1 }, ],
                nonempty![ MultilineOpinion::EntireSubsegment { index: 1 }, ]));
            "partial collision, entire subsegment 1 1"
        )]
        #[test_case(
            Multiline([*C, *D, *E]),
            Multiline([*D, *E, *J]),
            Some( (
                nonempty![ MultilineOpinion::EntireSubsegment { index: 1 }, ],
                nonempty![ MultilineOpinion::EntireSubsegment { index: 0 }, ]));
            "partial collision, entire subsegment 1 0"
        )]
        #[test_case(
            Multiline([*C, *D, *E]),
            Multiline([*E, *D, *C]),
            Some( (
                nonempty![ MultilineOpinion::EntireSubsegment { index: 0 }, MultilineOpinion::EntireSubsegment { index: 1 } ],
                nonempty![ MultilineOpinion::EntireSubsegment { index: 1 }, MultilineOpinion::EntireSubsegment { index: 0 } ]));
            "partial collision, entire subsegment 01 01 flipped"
        )]
        #[test_case(
            Multiline([*C, *D, *E, *J, *O]),
            Multiline([*C, *D, *I, *J, *O]),
            Some( (
                nonempty![ MultilineOpinion::EntireSubsegment { index: 0 }, MultilineOpinion::EntireSubsegment { index: 3 } ],
                nonempty![ MultilineOpinion::EntireSubsegment { index: 0 }, MultilineOpinion::EntireSubsegment { index: 3 } ]));
            "shared segment, then diversion, then another shared segment"
        )]
        #[test_case(
            Multiline([*C, *D, *E, *J, *O]),
            Multiline([*C, *D, *I, *J]),
            Some( (
                nonempty![ MultilineOpinion::EntireSubsegment { index: 0 }, MultilineOpinion::AtPoint { index: 3, at_point: *J } ],
                nonempty![ MultilineOpinion::EntireSubsegment { index: 0 }, MultilineOpinion::AtPoint { index: 3, at_point: *J } ]));
            "shared segment, then diversion, then atpoint"
        )]
        fn isxn(
            ml1: Multiline,
            ml2: Multiline,
            expectation: Option<(NonEmpty<MultilineOpinion>, NonEmpty<MultilineOpinion>)>,
        ) -> Result<()> {
            assert_eq!(multiline_overlaps_multiline(&ml1, &ml2)?, expectation);
            Ok(())
        }
    }

    mod pg_pt {
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
        #[test_case(Polygon([*D, *H, *N, *J]), &C, None; "point not in polygon 00")]
        #[test_case(Polygon([*D, *H, *N, *J]), &E, None; "point not in polygon 01")]
        #[test_case(
            Polygon([*D, *H, *N, *J]),
            &I,
            Some((PolygonOpinion::WithinArea, *I));
            "point in polygon"
        )]
        #[test_case(
            Polygon([*D, *H, *N, *J]),
            &D,
            Some((PolygonOpinion::AtPoint{index:0, at_point: *D}, *D));
            "point at point of polygon 00"
        )]
        #[test_case(
            Polygon([*D, *H, *N, *J]),
            &H,
            Some((PolygonOpinion::AtPoint{index:1, at_point: *H}, *H));
            "point at point of polygon 01"
        )]
        #[test_case(
            Polygon([*D, *H, *N, *J]),
            &N,
            Some((PolygonOpinion::AtPoint{index:2, at_point: *N}, *N));
            "point at point of polygon 02"
        )]
        #[test_case(
            Polygon([*D, *H, *N, *J]),
            &J,
            Some((PolygonOpinion::AtPoint{index:3, at_point: *J}, *J));
            "point at point of polygon 03"
        )]
        #[test_case(
            Polygon([*C, *M, *O, *E]),
            &H,
            Some((PolygonOpinion::AlongEdge{
                index: 0,
                at_point: *H,
                percent_along: Percent::Val(0.5)
            }, *H));
            "point at edge of polygon 00"
        )]
        #[test_case(
            Polygon([*C, *M, *O, *E]),
            &N,
            Some((PolygonOpinion::AlongEdge{
                index: 1,
                at_point: *N,
                percent_along: Percent::Val(0.5)
            }, *N));
            "point at edge of polygon 01"
        )]
        #[test_case(
            Polygon([*C, *M, *O, *E]),
            &J,
            Some((PolygonOpinion::AlongEdge{
                index: 2,
                at_point: *J,
                percent_along: Percent::Val(0.5)
            }, *J));
            "point at edge of polygon 02"
        )]
        #[test_case(
            Polygon([*C, *M, *O, *E]),
            &D,
            Some((PolygonOpinion::AlongEdge{
                index: 3,
                at_point: *D,
                percent_along: Percent::Val(0.5)
            }, *D));
            "point at edge of polygon 03"
        )]
        fn overlaps(
            pg: Result<Polygon>,
            pt: &Point,
            expectation: Option<(PolygonOpinion, Point)>,
        ) -> Result<()> {
            assert_eq!(polygon_overlaps_point(&pg?, pt)?, expectation);
            Ok(())
        }
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
