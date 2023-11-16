#![allow(missing_docs)]

pub mod opinion;

use self::opinion::{
    rewrite_multiline_opinions, rewrite_segment_opinions, MultilineOpinion, Opinion, SegmentOpinion,
};
use crate::{
    interpolate::interpolate_2d_checked,
    shapes::{multiline::Multiline, point::Point, segment::Segment},
    utils::Percent,
};
use anyhow::{anyhow, Result};
use float_cmp::approx_eq;
use nonempty::{nonempty, NonEmpty};

#[derive(PartialEq, Clone, Debug)]
pub enum Overlap {
    // respects order of overlaps() argument.
    Some(Opinion, Opinion),
    None,
}

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

pub fn segment_overlaps_segment(sa: &Segment, sb: &Segment) -> Result<Overlap> {
    // NB: sa and sb are _not_ guaranteed to point the same way.

    if sa == sb || *sa == sb.flip() {
        return Ok(Overlap::Some(
            Opinion::Segment(nonempty![SegmentOpinion::EntireSegment]),
            Opinion::Segment(nonempty![SegmentOpinion::EntireSegment]),
        ));
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
                    return Ok(Overlap::Some(
                        Opinion::Segment(nonempty![SegmentOpinion::AtPointAlongSegment {
                            at_point: pt,
                            percent_along: Percent::Zero
                        }]),
                        Opinion::Segment(nonempty![SegmentOpinion::AtPointAlongSegment {
                            at_point: pt,
                            percent_along: Percent::One
                        }]),
                    ));
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
                    return Ok(Overlap::Some(
                        Opinion::Segment(nonempty![SegmentOpinion::AtPointAlongSegment {
                            at_point: pt,
                            percent_along: Percent::One
                        }]),
                        Opinion::Segment(nonempty![SegmentOpinion::AtPointAlongSegment {
                            at_point: pt,
                            percent_along: Percent::Zero
                        }]),
                    ));
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
                    return Ok(Overlap::Some(
                        Opinion::Segment(nonempty![SegmentOpinion::AtPointAlongSegment {
                            at_point: pt,
                            percent_along: Percent::Zero
                        }]),
                        Opinion::Segment(nonempty![SegmentOpinion::AtPointAlongSegment {
                            at_point: pt,
                            percent_along: Percent::Zero
                        }]),
                    ));
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
                    return Ok(Overlap::Some(
                        Opinion::Segment(nonempty![SegmentOpinion::AtPointAlongSegment {
                            at_point: pt,
                            percent_along: Percent::One
                        }]),
                        Opinion::Segment(nonempty![SegmentOpinion::AtPointAlongSegment {
                            at_point: pt,
                            percent_along: Percent::One
                        }]),
                    ));
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
                return Ok(Overlap::Some(
                    Opinion::Segment(nonempty![SegmentOpinion::EntireSegment,]),
                    Opinion::Segment(nonempty![SegmentOpinion::AlongSubsegment(isxn_segment),]),
                ));
            } else if isxn_segment == *sb {
                return Ok(Overlap::Some(
                    Opinion::Segment(nonempty![SegmentOpinion::AlongSubsegment(isxn_segment),]),
                    Opinion::Segment(nonempty![SegmentOpinion::EntireSegment,]),
                ));
            } else {
                return Ok(Overlap::Some(
                    Opinion::Segment(nonempty![SegmentOpinion::AlongSubsegment(
                        // why dot/flip here? if the resultant segment doesn't
                        // run 'along' the input |sa|, |sb|, we have to flip it
                        // so that its subsegment is correctly oriented with
                        // respect to the input subsegment.
                        if isxn_segment.dot(sa) < 0.0 {
                            isxn_segment.flip()
                        } else {
                            isxn_segment
                        }
                    ),]),
                    Opinion::Segment(nonempty![SegmentOpinion::AlongSubsegment(
                        if isxn_segment.dot(sb) < 0.0 {
                            isxn_segment.flip()
                        } else {
                            isxn_segment
                        }
                    ),]),
                ));
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
        return Ok(Overlap::Some(
            Opinion::Segment(nonempty![SegmentOpinion::AtPointAlongSegment {
                at_point: pt,
                percent_along: interpolate_2d_checked(sa.i, sa.f, pt)?,
            }]),
            Opinion::Segment(nonempty![SegmentOpinion::AtPointAlongSegment {
                at_point: pt,
                percent_along: interpolate_2d_checked(sb.i, sb.f, pt)?,
            }]),
        ));
    }

    Ok(Overlap::None)
}

pub fn multiline_overlaps_point(ml: &Multiline, p: &Point) -> Result<Overlap> {
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
        None => Ok(Overlap::None),
        Some(u) => Ok(Overlap::Some(Opinion::Multiline(u), Opinion::Point)),
    }
}

pub fn multiline_overlaps_segment(ml: &Multiline, sg: &Segment) -> Result<Overlap> {
    let mut ml_opinions: Vec<MultilineOpinion> = vec![];
    let mut sg_opinions: Vec<SegmentOpinion> = vec![];

    for (ml_sg_idx, ml_sg) in ml.to_segments().iter().enumerate() {
        match segment_overlaps_segment(ml_sg, sg)? {
            Overlap::Some(Opinion::Segment(ml_sg_ops), Opinion::Segment(sg_ops)) => {
                assert_eq!(ml_sg_ops.len(), 1);
                assert_eq!(sg_ops.len(), 1);

                ml_opinions.push(MultilineOpinion::from_segment_opinion(
                    ml_sg_idx,
                    ml_sg_ops.head,
                ));
                sg_opinions.push(sg_ops.head);
            }
            Overlap::Some(_, _) => {
                return Err(anyhow!("segment_overlaps_segment should not have returned anything besides Opinion::Segment(_)."));
            }

            // finally Isxn::None.
            Overlap::None => {
                // nothing to do.
            }
        }
    }

    rewrite_multiline_opinions(&mut ml_opinions)?;
    rewrite_segment_opinions(&mut sg_opinions, sg)?;

    match (
        NonEmpty::from_vec(ml_opinions),
        NonEmpty::from_vec(sg_opinions),
    ) {
        (Some(total_ml_ops), Some(total_sg_ops)) => Ok(Overlap::Some(
            Opinion::Multiline(total_ml_ops),
            Opinion::Segment(total_sg_ops),
        )),
        (Some(_), None) | (None, Some(_)) => Err(anyhow!(
            "unexpected case - how can one object see collisions but the other doesn't?"
        )),
        (None, None) => Ok(Overlap::None),
    }
}

pub fn multiline_overlaps_multiline(ml1: &Multiline, ml2: &Multiline) -> Result<Overlap> {
    let mut ml1_opinions: Vec<MultilineOpinion> = vec![];
    let mut ml2_opinions: Vec<MultilineOpinion> = vec![];

    for (ml_sg1_idx, ml_sg1) in ml1.to_segments().iter().enumerate() {
        for (ml_sg2_idx, ml_sg2) in ml2.to_segments().iter().enumerate() {
            match segment_overlaps_segment(ml_sg1, ml_sg2)? {
                Overlap::Some(Opinion::Segment(ml_sg1_ops), Opinion::Segment(ml_sg2_ops)) => {
                    for ml_sg1_op in ml_sg1_ops.into_iter() {
                        ml1_opinions.push(MultilineOpinion::from_segment_opinion(ml_sg1_idx, ml_sg1_op));
                    }
                    for ml_sg2_op in ml_sg2_ops.into_iter() {
                        ml2_opinions.push(MultilineOpinion::from_segment_opinion(ml_sg2_idx, ml_sg2_op));
                    }
                }
                Overlap::Some(_, _) => panic!("segment_overlaps_segment should not have returned anything besides Opinion::Segment(_)."),
                Overlap::None => {
                    // nothing to do
                }
            }
        }
    }

    rewrite_multiline_opinions(&mut ml1_opinions)?;
    rewrite_multiline_opinions(&mut ml2_opinions)?;

    match (
        NonEmpty::from_vec(ml1_opinions),
        NonEmpty::from_vec(ml2_opinions),
    ) {
        (Some(total_ml1_ops), Some(total_ml2_ops)) => Ok(Overlap::Some(
            Opinion::Multiline(total_ml1_ops),
            Opinion::Multiline(total_ml2_ops),
        )),
        (Some(_), None) | (None, Some(_)) => Err(anyhow!(
            "unexpected case - how can one object see collisions but the other doesn't?"
        )),
        (None, None) => Ok(Overlap::None),
    }
    //
}

#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::lazy_static;
    use nonempty::nonempty;

    //   ^
    //   |
    //   A  B  C
    //   |
    //   D  E  F
    //   |
    // --G--H--I->
    //   |
    lazy_static! {
        static ref A: Point = Point(0, 2);
        static ref B: Point = Point(1, 2);
        static ref C: Point = Point(2, 2);
        static ref D: Point = Point(0, 1);
        static ref E: Point = Point(1, 1);
        static ref F: Point = Point(2, 1);
        static ref G: Point = Point(0, 0);
        static ref H: Point = Point(1, 0);
        static ref I: Point = Point(2, 0);
    }

    mod pt_pt {
        use super::*;

        #[test]
        fn the_same() -> Result<()> {
            for i in &[*A, *B, *C] {
                assert_eq!(point_overlaps_point(i, i)?, Some(*i));
            }
            Ok(())
        }

        #[test]
        fn not_the_same() -> Result<()> {
            for i in &[*A, *B, *C] {
                assert_eq!(point_overlaps_point(i, &D)?, None);
            }
            Ok(())
        }
    }

    mod sg_pt {
        use super::*;

        #[test]
        fn at_start_or_end() -> Result<()> {
            for (i, f) in &[(*A, *B), (*A, *E), (*A, *G)] {
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
            for (i, m, f) in &[(*A, *B, *C), (*A, *E, *I), (*A, *D, *G)] {
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
            for i in &[*A, *B, *C] {
                for j in &[*D, *E, *F] {
                    assert_eq!(
                        segment_overlaps_segment(&Segment(*i, *j), &Segment(*i, *j))?,
                        Overlap::Some(
                            Opinion::Segment(nonempty![SegmentOpinion::EntireSegment]),
                            Opinion::Segment(nonempty![SegmentOpinion::EntireSegment]),
                        )
                    );
                }
            }
            Ok(())
        }

        #[test]
        fn the_same_but_reversed() -> Result<()> {
            for i in &[*A, *B, *C] {
                for j in &[*D, *E, *F] {
                    assert_eq!(
                        segment_overlaps_segment(&Segment(*i, *j), &Segment(*j, *i))?,
                        Overlap::Some(
                            Opinion::Segment(nonempty![SegmentOpinion::EntireSegment]),
                            Opinion::Segment(nonempty![SegmentOpinion::EntireSegment]),
                        )
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
                    Overlap::Some(
                        Opinion::Segment(nonempty![SegmentOpinion::AtPointAlongSegment {
                            at_point,
                            percent_along: a_pct,
                        }]),
                        Opinion::Segment(nonempty![SegmentOpinion::AtPointAlongSegment {
                            at_point,
                            percent_along: b_pct,
                        }])
                    )
                );

                Ok(())
            }

            #[test_case(Segment(*Q, *T), Segment(*Q, *E), Segment(*Q, *E))]
            #[test_case(Segment(*Q, *T), Segment(*W, *R), Segment(*W, *R))]
            #[test_case(Segment(*Q, *T), Segment(*E, *T), Segment(*E, *T))]
            fn atsubsegment(sga: Segment, sgb: Segment, subsegment: Segment) -> Result<()> {
                assert_eq!(
                    segment_overlaps_segment(&sga, &sgb)?,
                    Overlap::Some(
                        Opinion::Segment(nonempty![SegmentOpinion::AlongSubsegment(subsegment)]),
                        Opinion::Segment(nonempty![SegmentOpinion::EntireSegment])
                    )
                );
                assert_eq!(
                    segment_overlaps_segment(&sgb, &sga)?,
                    Overlap::Some(
                        Opinion::Segment(nonempty![SegmentOpinion::EntireSegment]),
                        Opinion::Segment(nonempty![SegmentOpinion::AlongSubsegment(subsegment)])
                    )
                );

                Ok(())
            }
        }

        #[test]
        fn partway() -> Result<()> {
            {
                // given two non-colinear segments,
                let (p0, p1) = (*A, *B);
                for p2 in &[*D, *E, *F, *G, *H, *I] {
                    assert_eq!(
                        segment_overlaps_segment(&Segment(p0, p1), &Segment(p1, *p2))?,
                        Overlap::Some(
                            Opinion::Segment(nonempty![SegmentOpinion::AtPointAlongSegment {
                                at_point: p1,
                                percent_along: Percent::One
                            }]),
                            Opinion::Segment(nonempty![SegmentOpinion::AtPointAlongSegment {
                                at_point: p1,
                                percent_along: Percent::Zero
                            }]),
                        )
                    );
                }
            }

            {
                // midpoints
                let sa = Segment(*A, *I);
                let sb = Segment(*C, *G);
                for (sa, sb) in &[
                    (sa, sb),
                    (sa, sb.flip()),
                    (sa.flip(), sb),
                    (sa.flip(), sb.flip()),
                ] {
                    assert_eq!(
                        segment_overlaps_segment(sa, sb)?,
                        Overlap::Some(
                            Opinion::Segment(nonempty![SegmentOpinion::AtPointAlongSegment {
                                at_point: *E,
                                percent_along: Percent::Val(0.5)
                            }]),
                            Opinion::Segment(nonempty![SegmentOpinion::AtPointAlongSegment {
                                at_point: *E,
                                percent_along: Percent::Val(0.5)
                            }]),
                        )
                    );
                }
            }

            Ok(())
        }

        #[test_case(
            Segment(*A, *C),
            Segment(Point(0.5, 2), Point(1.5,2)),
            Overlap::Some(
                Opinion::Segment(nonempty![
                    SegmentOpinion::AlongSubsegment(Segment(Point(0.5,2), Point(1.5, 2)))
                ]),
                Opinion::Segment(nonempty![
                    SegmentOpinion::EntireSegment
                ]),
            );
            "partial collision"
        )]
        #[test_case(
            Segment(*A, *C),
            Segment(Point(1.5, 2), Point(0.5,2)),
            Overlap::Some(
                Opinion::Segment(nonempty![
                    SegmentOpinion::AlongSubsegment(Segment(Point(1.5,2), Point(0.5, 2)))
                ]),
                Opinion::Segment(nonempty![
                    SegmentOpinion::EntireSegment
                ]),
            );
            "partial collision, flip"
        )]
        #[test_case(
            Segment(Point(0,2), Point(1,2)),
            Segment(Point(1.5,2), Point(0.5,2)),
            Overlap::Some(
                Opinion::Segment(nonempty![
                    SegmentOpinion::AlongSubsegment(Segment(Point(0.5,2), Point(1,2))),
                ]),
                Opinion::Segment(nonempty![
                    SegmentOpinion::AlongSubsegment(Segment(Point(1,2), Point(0.5,2))),
                ])
            );
            "partial collision, backwards"
        )]
        fn isxn(a: Segment, b: Segment, expectation: Overlap) -> Result<()> {
            assert_eq!(segment_overlaps_segment(&a, &b)?, expectation);
            Ok(())
        }
    }

    mod ml_pt {
        use super::*;

        //   ^
        //   |
        //   A  B  C
        //   |
        //   D  E  F
        //   |
        // --G--H--I->
        //   |
        #[test]
        fn test_along_two_segment_multiline() -> Result<()> {
            for ((start, midpoint1, pivot, midpoint2, end), unrelated) in &[
                ((*G, *H, *I, *F, *C), *A),
                ((*G, *D, *A, *B, *C), *I),
                ((*C, *B, *A, *D, *G), *I),
                ((*G, *E, *C, *B, *A), *D),
            ] {
                let ml = Multiline([*start, *pivot, *end]);

                // check points

                for (pt, idx) in [(start, 0), (pivot, 1), (end, 2)] {
                    assert_eq!(
                        multiline_overlaps_point(&ml, &pt)?,
                        Overlap::Some(
                            Opinion::Multiline(nonempty![MultilineOpinion::AtPoint {
                                index: idx,
                                at_point: *pt
                            }]),
                            Opinion::Point
                        )
                    );
                }

                // check segments

                for (pt, idx) in [(midpoint1, 0), (midpoint2, 1)] {
                    assert_eq!(
                        multiline_overlaps_point(&ml, &pt)?,
                        Overlap::Some(
                            Opinion::Multiline(nonempty![
                                MultilineOpinion::AtPointAlongSharedSegment {
                                    index: idx,
                                    at_point: *pt,
                                    percent_along: Percent::Val(0.5)
                                }
                            ]),
                            Opinion::Point
                        ),
                    );
                }

                assert_eq!(multiline_overlaps_point(&ml, unrelated)?, Overlap::None);
            }

            Ok(())
        }
    }

    mod ml_sg {
        use super::*;
        use test_case::test_case;

        //   ^
        //   |
        //   A  B  C
        //   |
        //   D  E  F
        //   |
        // --G--H--I->
        //   |

        #[test_case(Multiline([*A, *C, *I]), Segment(*G, *H))]
        #[test_case(Multiline([*A, *C, *F]), Segment(*G, *H))]
        #[test_case(Multiline([*A, *C, *I]), Segment(*D, *H))]
        #[test_case(Multiline([*A, *E, *I]), Segment(*B, *F))]
        fn none(ml: Multiline, sg: Segment) -> Result<()> {
            assert_eq!(multiline_overlaps_segment(&ml, &sg)?, Overlap::None);
            Ok(())
        }

        #[test_case(Multiline([*A, *C, *I]), Segment(*A, *G), 0, *A, Percent::Zero)]
        #[test_case(Multiline([*C, *I, *G]), Segment(*C, *A), 0, *C, Percent::Zero)]
        #[test_case(Multiline([*I, *G, *A]), Segment(*I, *C), 0, *I, Percent::Zero)]
        #[test_case(Multiline([*A, *E, *I]), Segment(*A, *G), 0, *A, Percent::Zero)]
        #[test_case(Multiline([*A, *C, *I]), Segment(*G, *A), 0, *A, Percent::One)]
        #[test_case(Multiline([*C, *I, *G]), Segment(*A, *C), 0, *C, Percent::One)]
        #[test_case(Multiline([*I, *G, *A]), Segment(*C, *I), 0, *I, Percent::One)]
        #[test_case(Multiline([*A, *E, *I]), Segment(*G, *A), 0, *A, Percent::One)]
        #[test_case(Multiline([*B, *E, *H]), Segment(*A, *C), 0, *B, Percent::Val(0.5))]
        #[test_case(Multiline([*D, *E, *F]), Segment(*G, *A), 0, *D, Percent::Val(0.5))]
        #[test_case(Multiline([*B, *E, *H]), Segment(*E, *F), 1, *E, Percent::Zero)]
        #[test_case(Multiline([*B, *E, *H]), Segment(*D, *E), 1, *E, Percent::One)]
        #[test_case(Multiline([*B, *E, *H]), Segment(*D, *F), 1, *E, Percent::Val(0.5))]
        #[test_case(Multiline([*B, *E, *H]), Segment(*H, *I), 2, *H, Percent::Zero)]
        #[test_case(Multiline([*B, *E, *H]), Segment(*G, *H), 2, *H, Percent::One)]
        #[test_case(Multiline([*B, *E, *H]), Segment(*G, *I), 2, *H, Percent::Val(0.5))]
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
                Overlap::Some(
                    Opinion::Multiline(nonempty![MultilineOpinion::AtPoint { index, at_point }]),
                    Opinion::Segment(nonempty![SegmentOpinion::AtPointAlongSegment {
                        at_point,
                        percent_along,
                    }])
                )
            );
            Ok(())
        }

        #[test_case(Multiline([*D, *F, *I]), Segment(*B, *H), 0, *E, Percent::Val(0.5), Percent::Val(0.5))]
        #[test_case(Multiline([*D, *F, *I]), Segment(*E, *H), 0, *E, Percent::Val(0.5), Percent::Zero)]
        #[test_case(Multiline([*D, *F, *I]), Segment(*B, *E), 0, *E, Percent::Val(0.5), Percent::One)]
        #[test_case(Multiline([*G, *D, *F]), Segment(*B, *H), 1, *E, Percent::Val(0.5), Percent::Val(0.5))]
        #[test_case(Multiline([*G, *D, *F]), Segment(*E, *H), 1, *E, Percent::Val(0.5), Percent::Zero)]
        #[test_case(Multiline([*G, *D, *F]), Segment(*B, *E), 1, *E, Percent::Val(0.5), Percent::One)]
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
                Overlap::Some(
                    Opinion::Multiline(nonempty![MultilineOpinion::AtPointAlongSharedSegment {
                        index,
                        at_point,
                        percent_along: ml_pct_along,
                    }]),
                    Opinion::Segment(nonempty![SegmentOpinion::AtPointAlongSegment {
                        at_point,
                        percent_along: sg_pct_along,
                    }])
                )
            );
            Ok(())
        }

        #[test_case(
            Multiline([*A, *C, *I]),
            Segment(*A, *I),
            Overlap::Some(
                Opinion::Multiline(nonempty![
                    MultilineOpinion::AtPoint {
                        index: 0,
                        at_point: *A,
                    },
                    MultilineOpinion::AtPoint {
                        index: 2,
                        at_point: *I,
                    }
                ]),
                Opinion::Segment(nonempty![
                    SegmentOpinion::AtPointAlongSegment {
                        at_point: *A,
                        percent_along: Percent::Zero,
                    },
                    SegmentOpinion::AtPointAlongSegment {
                        at_point: *I,
                        percent_along: Percent::One,
                    }
                ])
            );
            "segment bookends 1"
        )]
        #[test_case(
            Multiline([*A, *C, *I]),
            Segment(*B, *F),
            Overlap::Some(
                Opinion::Multiline(nonempty![
                    MultilineOpinion::AtPointAlongSharedSegment {
                        index: 0,
                        at_point: *B,
                        percent_along: Percent::Val(0.5),
                    },
                    MultilineOpinion::AtPointAlongSharedSegment {
                        index: 1,
                        at_point: *F,
                        percent_along: Percent::Val(0.5),
                    }
                ]),
                Opinion::Segment(nonempty![
                    SegmentOpinion::AtPointAlongSegment {
                        at_point: *B,
                        percent_along: Percent::Zero,
                    },
                    SegmentOpinion::AtPointAlongSegment {
                        at_point: *F,
                        percent_along: Percent::One,
                    }
                ]),
            );
            "segment bookends 2"
        )]
        #[test_case(
            Multiline([*A, *B, *C]),
            Segment(*A, *B),
            Overlap::Some(
                Opinion::Multiline(nonempty![
                    MultilineOpinion::EntireSubsegment { index: 0 }
                ]),
                Opinion::Segment(nonempty![
                    SegmentOpinion::EntireSegment
                ])
            );
            "partial collision"
        )]
        #[test_case(
            Multiline([*A, *B, *C]),
            Segment(*B, *A),
            Overlap::Some(
                Opinion::Multiline(nonempty![
                    MultilineOpinion::EntireSubsegment { index: 0 }
                ]),
                Opinion::Segment(nonempty![
                    SegmentOpinion::EntireSegment
                ])
            );
            "partial collision 02"
        )]
        #[test_case(
            Multiline([*A, *B, *C]),
            Segment(*B, *C),
            Overlap::Some(
                Opinion::Multiline(nonempty![
                    MultilineOpinion::EntireSubsegment { index: 1 }
                ]),
                Opinion::Segment(nonempty![
                    SegmentOpinion::EntireSegment
                ])
            );
            "partial collision 03"
        )]
        #[test_case(
            Multiline([*A, *B, *C]),
            Segment(*C, *B),
            Overlap::Some(
                Opinion::Multiline(nonempty![
                    MultilineOpinion::EntireSubsegment { index: 1 }
                ]),
                Opinion::Segment(nonempty![
                    SegmentOpinion::EntireSegment
                ])
            );
            "partial collision 04"
        )]
        #[test_case(
            Multiline([*A, *B, *C]),
            Segment(*A, *C),
            Overlap::Some(
                Opinion::Multiline(nonempty![
                    MultilineOpinion::EntireSubsegment { index: 0 },
                    MultilineOpinion::EntireSubsegment { index: 1 }
                ]),
                Opinion::Segment(nonempty![
                    SegmentOpinion::EntireSegment
                ])
            );
            "total collision 01"
        )]
        #[test_case(
            Multiline([*A, *B, *C]),
            Segment(*C, *A),
            Overlap::Some(
                Opinion::Multiline(nonempty![
                    MultilineOpinion::EntireSubsegment { index: 0 },
                    MultilineOpinion::EntireSubsegment { index: 1 }
                ]),
                Opinion::Segment(nonempty![
                    SegmentOpinion::EntireSegment
                ])
            );
            "total collision 01 flip"
        )]
        #[test_case(
            Multiline([*A, *B, *C]),
            Segment(Point(0.5,2), Point(1.5,2)),
            Overlap::Some(
                Opinion::Multiline(nonempty![
                    MultilineOpinion::AlongSubsegmentOf {
                        index: 0,
                        subsegment: Segment(Point(0.5,2), Point(1,2))
                    },
                    MultilineOpinion::AlongSubsegmentOf {
                        index: 1,
                        subsegment: Segment(Point(1,2), Point(1.5,2))
                    }
                ]),
                Opinion::Segment(nonempty![
                    SegmentOpinion::EntireSegment
                ])
            );
            "total collision half shift 01"
        )]
        #[test_case(
            Multiline([*A, *B, *C]),
            Segment(Point(1.5,2), Point(0.5,2)),
            Overlap::Some(
                Opinion::Multiline(nonempty![
                    MultilineOpinion::AlongSubsegmentOf {
                        index: 0,
                        subsegment: Segment(Point(0.5,2), Point(1,2))
                    },
                    MultilineOpinion::AlongSubsegmentOf {
                        index: 1,
                        subsegment: Segment(Point(1,2), Point(1.5,2))
                    }
                ]),
                Opinion::Segment(nonempty![
                    SegmentOpinion::EntireSegment
                ])
            );
            "total collision half shift 01 flip"
        )]
        fn isxn(ml: Multiline, sg: Segment, expectation: Overlap) -> Result<()> {
            assert_eq!(multiline_overlaps_segment(&ml, &sg)?, expectation);
            Ok(())
        }
    }

    mod ml_ml {
        use super::*;
        use test_case::test_case;

        //   ^
        //   |
        //   A  B  C
        //   |
        //   D  E  F
        //   |
        // --G--H--I->
        //   |

        #[test_case(Multiline([*A, *B, *C]), Multiline([*D, *E, *F]), Overlap::None; "none 01")]
        #[test_case(Multiline([*A, *B, *C]), Multiline([*G, *H, *I]), Overlap::None; "none 02")]
        #[test_case(Multiline([*A, *E, *I]), Multiline([*B, *F]), Overlap::None; "none diagonal")]
        #[test_case(
            Multiline([*A, *B, *C]),
            Multiline([*A, *D, *G]),
            Overlap::Some(
                Opinion::Multiline(nonempty![
                    MultilineOpinion::AtPoint { index: 0, at_point: *A },
                ]),
                Opinion::Multiline(nonempty![
                    MultilineOpinion::AtPoint { index: 0, at_point: *A }
                ])
            );
            "AtPoint 0, AtPoint 0"
        )]
        #[test_case(
            Multiline([*A, *B, *C]),
            Multiline([*G, *D, *A]),
            Overlap::Some(
                Opinion::Multiline(nonempty![
                    MultilineOpinion::AtPoint { index: 0, at_point: *A },
                ]),
                Opinion::Multiline(nonempty![
                    MultilineOpinion::AtPoint { index: 2, at_point: *A }
                ])
            );
            "AtPoint 0, AtPoint 2"
        )]
        #[test_case(
            Multiline([*C, *B, *A]),
            Multiline([*G, *D, *A]),
            Overlap::Some(
                Opinion::Multiline(nonempty![
                    MultilineOpinion::AtPoint { index: 2, at_point: *A },
                ]),
                Opinion::Multiline(nonempty![
                    MultilineOpinion::AtPoint { index: 2, at_point: *A }
                ])
            );
            "AtPoint 2, AtPoint 2"
        )]
        #[test_case(
            Multiline([*A, *E, *I]),
            Multiline([*G, *E, *C]),
            Overlap::Some(
                Opinion::Multiline(nonempty![
                    MultilineOpinion::AtPoint { index: 1, at_point: *E },
                ]),
                Opinion::Multiline(nonempty![
                    MultilineOpinion::AtPoint { index: 1, at_point: *E }
                ])
            );
            "AtPoint 1, AtPoint 1"
        )]
        #[test_case(
            Multiline([*A, *I]),
            Multiline([*C, *G]),
            Overlap::Some(
                Opinion::Multiline(nonempty![
                    MultilineOpinion::AtPointAlongSharedSegment { index: 0, at_point: *E, percent_along: Percent::Val(0.5) }
                ]),
                Opinion::Multiline(nonempty![
                    MultilineOpinion::AtPointAlongSharedSegment { index: 0, at_point: *E, percent_along: Percent::Val(0.5) }
                ])
            );
            "crosshairs"
        )]
        #[test_case(
            Multiline([*A, *B, *C]),
            Multiline([*A, *B, *E]),
            Overlap::Some(
                Opinion::Multiline(nonempty![
                    MultilineOpinion::EntireSubsegment { index: 0 },
                ]),
                Opinion::Multiline(nonempty![
                    MultilineOpinion::EntireSubsegment { index: 0 },
                ])
            );
            "partial collision, entire subsegment 0 0"
        )]
        #[test_case(
            Multiline([*C, *B, *A]),
            Multiline([*E, *B, *A]),
            Overlap::Some(
                Opinion::Multiline(nonempty![
                    MultilineOpinion::EntireSubsegment { index: 1 },
                ]),
                Opinion::Multiline(nonempty![
                    MultilineOpinion::EntireSubsegment { index: 1 },
                ])
            );
            "partial collision, entire subsegment 1 1"
        )]
        #[test_case(
            Multiline([*A, *B, *C]),
            Multiline([*B, *C, *F]),
            Overlap::Some(
                Opinion::Multiline(nonempty![
                    MultilineOpinion::EntireSubsegment { index: 1 },
                ]),
                Opinion::Multiline(nonempty![
                    MultilineOpinion::EntireSubsegment { index: 0 },
                ])
            );
            "partial collision, entire subsegment 1 0"
        )]
        #[test_case(
            Multiline([*A, *B, *C]),
            Multiline([*C, *B, *A]),
            Overlap::Some(
                Opinion::Multiline(nonempty![
                    MultilineOpinion::EntireSubsegment { index: 0 },
                    MultilineOpinion::EntireSubsegment { index: 1 }
                ]),
                Opinion::Multiline(nonempty![
                    MultilineOpinion::EntireSubsegment { index: 1 },
                    MultilineOpinion::EntireSubsegment { index: 0 }
                ])
            );
            "partial collision, entire subsegment 01 01 flipped"
        )]
        #[test_case(
            Multiline([*A, *B, *C, *F, *I]),
            Multiline([*A, *B, *E, *F, *I]),
            Overlap::Some(
                Opinion::Multiline(nonempty![
                    MultilineOpinion::EntireSubsegment { index: 0 },
                    MultilineOpinion::EntireSubsegment { index: 3 }
                ]),
                Opinion::Multiline(nonempty![
                    MultilineOpinion::EntireSubsegment { index: 0 },
                    MultilineOpinion::EntireSubsegment { index: 3 }
                ])
            );
            "shared segment, then diversion, then another shared segment"
        )]
        #[test_case(
            Multiline([*A, *B, *C, *F, *I]),
            Multiline([*A, *B, *E, *F]),
            Overlap::Some(
                Opinion::Multiline(nonempty![
                    MultilineOpinion::EntireSubsegment { index: 0 },
                    MultilineOpinion::AtPoint { index: 3, at_point: *F }
                ]),
                Opinion::Multiline(nonempty![
                    MultilineOpinion::EntireSubsegment { index: 0 },
                    MultilineOpinion::AtPoint { index: 3, at_point: *F }
                ])
            );
            "shared segment, then diversion, then atpoint"
        )]
        fn isxn(ml1: Multiline, ml2: Multiline, expectation: Overlap) -> Result<()> {
            assert_eq!(multiline_overlaps_multiline(&ml1, &ml2)?, expectation);
            Ok(())
        }
    }
}
