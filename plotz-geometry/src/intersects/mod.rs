#![allow(missing_docs)]

pub mod opinion;

use self::opinion::{MultilineOpinion, Opinion, SegmentOpinion};
use crate::{
    interpolate::interpolate_2d_checked,
    obj2::Obj2,
    shapes::{multiline::Multiline, point::Point, segment::Segment},
    utils::Percent,
};
use anyhow::{anyhow, Result};
use float_cmp::approx_eq;
use nonempty::{nonempty, NonEmpty};

pub enum PolygonIntersectionResult {
    AtPoint(
        // The index at which it occurred.
        usize,
    ),
    AlongSegment(
        // The index at which it occurred.
        usize,
        // The percentage of the way along this segment at which it occurred.
        Percent,
    ),
}

#[derive(PartialEq, Clone, Debug)]
pub enum Isxn {
    // SpecialCase(General),
    // respects order of intersects() argument.
    Some(Opinion, Opinion),
    None,
}

impl Isxn {
    fn flip(self) -> Isxn {
        match self {
            Isxn::Some(a, b) => Isxn::Some(b, a),
            _ => self,
        }
    }
}

pub fn obj_intersects_obj(a: &Obj2, b: &Obj2) -> Result<Isxn> {
    //
    //           || pt | sg | ml |
    // ==========++====+====+====+==
    //     point || ✔️  | \  | \  |
    //   segment || ✔️  | ✔️  | \  |
    // multiline || ✔️  | ✔️  | ~  |
    // ==========++====+====+====+==
    //
    match a {
        Obj2::Point(p) => match b {
            Obj2::Point(p2) => point_intersects_point(p, p2),
            _ => obj_intersects_obj(b, a).map(Isxn::flip),
        },
        Obj2::Segment(sg) => match b {
            Obj2::Point(pt) => segment_intersects_point(sg, pt),
            Obj2::Segment(sg2) => segment_intersects_segment(sg, sg2),
            _ => obj_intersects_obj(b, a).map(Isxn::flip),
        },
        Obj2::Multiline(ml) => match b {
            Obj2::Point(pt) => multiline_intersects_point(ml, pt),
            Obj2::Segment(sg) => multiline_intersects_segment(ml, sg),
            Obj2::Multiline(ml2) => multiline_intersects_multiline(ml, ml2),
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}

pub fn point_intersects_point(a: &Point, b: &Point) -> Result<Isxn> {
    if a == b {
        Ok(Isxn::Some(Opinion::Point, Opinion::Point))
    } else {
        Ok(Isxn::None)
    }
}

pub fn segment_intersects_point(s: &Segment, p: &Point) -> Result<Isxn> {
    if s.i == *p {
        Ok(Isxn::Some(
            Opinion::Segment(nonempty![SegmentOpinion::AtPointAlongSegment {
                at_point: *p,
                percent_along: Percent::Zero,
            }]),
            Opinion::Point,
        ))
    } else if s.f == *p {
        Ok(Isxn::Some(
            Opinion::Segment(nonempty![SegmentOpinion::AtPointAlongSegment {
                at_point: *p,
                percent_along: Percent::One,
            }]),
            Opinion::Point,
        ))
    } else if approx_eq!(
        f64,
        s.length(),
        Segment(s.i, *p).length() + Segment(*p, s.f).length()
    ) {
        Ok(Isxn::Some(
            Opinion::Segment(nonempty![SegmentOpinion::AtPointAlongSegment {
                at_point: *p,
                percent_along: interpolate_2d_checked(s.i, s.f, *p)?,
            }]),
            Opinion::Point,
        ))
    } else {
        Ok(Isxn::None)
    }
}

pub fn segment_intersects_segment(sa: &Segment, sb: &Segment) -> Result<Isxn> {
    if sa == sb || *sa == sb.flip() {
        return Ok(Isxn::Some(
            Opinion::Segment(nonempty![SegmentOpinion::EntireSegment]),
            Opinion::Segment(nonempty![SegmentOpinion::EntireSegment]),
        ));
    }

    if approx_eq!(f64, sa.slope(), sb.slope()) || approx_eq!(f64, sa.slope(), sb.flip().slope()) {
        let isxn_segment: Option<Segment> = match (
            segment_intersects_point(sb, &sa.i)?,
            segment_intersects_point(sb, &sa.f)?,
            segment_intersects_point(sa, &sb.i)?,
            segment_intersects_point(sa, &sb.f)?,
        ) {
            // No collision.
            (Isxn::None, Isxn::None, Isxn::None, Isxn::None) => None,

            // ERR: same
            //
            // |---|
            // |---|
            (Isxn::Some(_, _), Isxn::Some(_, _), Isxn::Some(_, _), Isxn::Some(_, _)) => {
                return Err(anyhow!(
                    "these are the same line; sa==sb should have triggered."
                ));
            }

            // |---|
            // |----|
            // or
            //  |---|
            // |----|
            // or
            //  |---|
            // |-----|
            (Isxn::Some(_, _), Isxn::Some(_, _), _, _) => Some(*sa),

            // |-----|
            // |---|
            // or
            // |-----|
            //  |---|
            // |----|
            //  |---|
            (_, _, Isxn::Some(_, _), Isxn::Some(_, _)) => Some(*sb),

            (Isxn::Some(_, _), Isxn::None, Isxn::None, Isxn::Some(_, _)) => {
                if sa.i == sb.f {
                    //     |---|
                    // |---|
                    let pt = sa.i;
                    return Ok(Isxn::Some(
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
                //   |---|
                // |---|
                Some(Segment(sa.i, sb.f))
            }

            (Isxn::None, Isxn::Some(_, _), Isxn::Some(_, _), Isxn::None) => {
                if sa.f == sb.i {
                    // |---|
                    //     |---|
                    let pt = sa.f;
                    return Ok(Isxn::Some(
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
                // |---|
                //   |---|
                Some(Segment(sb.i, sa.f))
            }

            // Head-to-head collision.
            (Isxn::Some(_, _), Isxn::None, Isxn::Some(_, _), Isxn::None) => {
                let pt = sa.i;
                return Ok(Isxn::Some(
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

            // Tail-to-tail collision.
            (Isxn::None, Isxn::Some(_, _), Isxn::None, Isxn::Some(_, _)) => {
                let pt = sa.f;
                return Ok(Isxn::Some(
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

            _ => {
                return Err(anyhow!("this should not be possible."));
            }
        };

        if let Some(isxn_segment) = isxn_segment {
            if isxn_segment == *sa {
                return Ok(Isxn::Some(
                    Opinion::Segment(nonempty![SegmentOpinion::EntireSegment,]),
                    Opinion::Segment(nonempty![SegmentOpinion::AlongSubsegment(isxn_segment),]),
                ));
            } else if isxn_segment == *sb {
                return Ok(Isxn::Some(
                    Opinion::Segment(nonempty![SegmentOpinion::AlongSubsegment(isxn_segment),]),
                    Opinion::Segment(nonempty![SegmentOpinion::EntireSegment,]),
                ));
            } else {
                return Ok(Isxn::Some(
                    Opinion::Segment(nonempty![SegmentOpinion::AlongSubsegment(isxn_segment),]),
                    Opinion::Segment(nonempty![SegmentOpinion::AlongSubsegment(isxn_segment),]),
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
        return Ok(Isxn::Some(
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

    Ok(Isxn::None)
}

pub fn multiline_intersects_point(ml: &Multiline, p: &Point) -> Result<Isxn> {
    let mut sg_ops: Vec<MultilineOpinion> = vec![];
    for (index, sg) in ml.to_segments().iter().enumerate() {
        if let Isxn::Some(Opinion::Segment(segment_opinions), _) = segment_intersects_point(sg, p)?
        {
            assert_eq!(segment_opinions.len(), 1);
            for segment_opinion in segment_opinions {
                sg_ops.push(MultilineOpinion::from_segment_opinion(
                    index,
                    segment_opinion,
                ));
            }
        }
    }
    sg_ops.dedup();
    match NonEmpty::from_vec(sg_ops) {
        None => Ok(Isxn::None),
        Some(u) => Ok(Isxn::Some(Opinion::Multiline(u), Opinion::Point)),
    }
}

pub fn multiline_intersects_segment(ml: &Multiline, sg: &Segment) -> Result<Isxn> {
    let mut total_ml_ops: Vec<MultilineOpinion> = vec![];
    let mut total_sg_ops: Vec<SegmentOpinion> = vec![];

    for (ml_sg_idx, ml_sg) in ml.to_segments().iter().enumerate() {
        match segment_intersects_segment(ml_sg, sg)? {
            Isxn::Some(Opinion::Segment(ml_sg_ops), Opinion::Segment(sg_ops)) => {
                assert_eq!(ml_sg_ops.len(), 1);
                assert_eq!(sg_ops.len(), 1);

                total_ml_ops.push(MultilineOpinion::from_segment_opinion(
                    ml_sg_idx,
                    ml_sg_ops.head,
                ));
                total_sg_ops.push(sg_ops.head);
            }
            Isxn::Some(_, _) => {
                return Err(anyhow!("segment_intersects_segment should not have returned anything besides Opinion::Segment(_)."));
            }

            // finally Isxn::None.
            Isxn::None => {
                // nothing to do.
            }
        }
    }

    total_ml_ops.dedup();
    total_sg_ops.dedup();

    match (
        NonEmpty::from_vec(total_ml_ops),
        NonEmpty::from_vec(total_sg_ops),
    ) {
        (Some(total_ml_ops), Some(total_sg_ops)) => Ok(Isxn::Some(
            Opinion::Multiline(total_ml_ops),
            Opinion::Segment(total_sg_ops),
        )),
        _ => Ok(Isxn::None),
    }
}

pub fn multiline_intersects_multiline(ml1: &Multiline, ml2: &Multiline) -> Result<Isxn> {
    let mut total_ml1_ops: Vec<MultilineOpinion> = vec![];
    let mut total_ml2_ops: Vec<MultilineOpinion> = vec![];

    for (ml_sg1_idx, ml_sg1) in ml1.to_segments().iter().enumerate() {
        for (ml_sg2_idx, ml_sg2) in ml2.to_segments().iter().enumerate() {
            match segment_intersects_segment(ml_sg1, ml_sg2)? {
                Isxn::Some(Opinion::Segment(ml_sg1_ops), Opinion::Segment(ml_sg2_ops)) => {
                    for ml_sg1_op in ml_sg1_ops.into_iter() {
                        total_ml1_ops.push(MultilineOpinion::from_segment_opinion(ml_sg1_idx, ml_sg1_op));
                    }
                    for ml_sg2_op in ml_sg2_ops.into_iter() {
                        total_ml2_ops.push(MultilineOpinion::from_segment_opinion(ml_sg2_idx, ml_sg2_op));
                    }
                }
                Isxn::Some(_, _) => panic!("segment_intersects_segment should not have returned anything besides Opinion::Segment(_)."),
                Isxn::None => {
                    // nothing to do
                }
            }
        }
    }

    total_ml1_ops.dedup();
    total_ml2_ops.dedup();

    match (
        NonEmpty::from_vec(total_ml1_ops),
        NonEmpty::from_vec(total_ml2_ops),
    ) {
        (Some(total_ml1_ops), Some(total_ml2_ops)) => Ok(Isxn::Some(
            Opinion::Multiline(total_ml1_ops),
            Opinion::Multiline(total_ml2_ops),
        )),
        _ => Ok(Isxn::None),
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
                assert_eq!(
                    point_intersects_point(i, i)?,
                    Isxn::Some(Opinion::Point, Opinion::Point),
                );
            }
            Ok(())
        }

        #[test]
        fn not_the_same() -> Result<()> {
            for i in &[*A, *B, *C] {
                assert_eq!(point_intersects_point(i, &D)?, Isxn::None,);
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
                    segment_intersects_point(&Segment(*i, *f), i)?,
                    Isxn::Some(
                        Opinion::Segment(nonempty![SegmentOpinion::AtPointAlongSegment {
                            at_point: *i,
                            percent_along: Percent::Zero
                        }]),
                        Opinion::Point
                    )
                );
                assert_eq!(
                    segment_intersects_point(&Segment(*i, *f), f)?,
                    Isxn::Some(
                        Opinion::Segment(nonempty![SegmentOpinion::AtPointAlongSegment {
                            at_point: *f,
                            percent_along: Percent::One
                        }]),
                        Opinion::Point
                    )
                );
            }
            Ok(())
        }

        #[test]
        fn halfway_along() -> Result<()> {
            for (i, m, f) in &[(*A, *B, *C), (*A, *E, *I), (*A, *D, *G)] {
                assert_eq!(
                    segment_intersects_point(&Segment(*i, *f), m)?,
                    Isxn::Some(
                        Opinion::Segment(nonempty![SegmentOpinion::AtPointAlongSegment {
                            at_point: *m,
                            percent_along: Percent::Val(0.5)
                        }]),
                        Opinion::Point
                    )
                );
            }
            Ok(())
        }
    }

    mod sg_sg {
        use super::*;
        // use test_case::test_case;

        #[test]
        fn the_same() -> Result<()> {
            for i in &[*A, *B, *C] {
                for j in &[*D, *E, *F] {
                    assert_eq!(
                        segment_intersects_segment(&Segment(*i, *j), &Segment(*i, *j))?,
                        Isxn::Some(
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
                        segment_intersects_segment(&Segment(*i, *j), &Segment(*j, *i))?,
                        Isxn::Some(
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
                    segment_intersects_segment(&sga, &sgb)?,
                    Isxn::Some(
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
                    segment_intersects_segment(&sga, &sgb)?,
                    Isxn::Some(
                        Opinion::Segment(nonempty![SegmentOpinion::AlongSubsegment(subsegment)]),
                        Opinion::Segment(nonempty![SegmentOpinion::EntireSegment])
                    )
                );
                assert_eq!(
                    segment_intersects_segment(&sgb, &sga)?,
                    Isxn::Some(
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
                        segment_intersects_segment(&Segment(p0, p1), &Segment(p1, *p2))?,
                        Isxn::Some(
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
                        segment_intersects_segment(sa, sb)?,
                        Isxn::Some(
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
                        multiline_intersects_point(&ml, &pt)?,
                        Isxn::Some(
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
                        multiline_intersects_point(&ml, &pt)?,
                        Isxn::Some(
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

                assert_eq!(multiline_intersects_point(&ml, unrelated)?, Isxn::None);
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
            assert_eq!(multiline_intersects_segment(&ml, &sg)?, Isxn::None);
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
        fn one_intersection_ml_atpoint_sg(
            ml: Multiline,
            sg: Segment,
            index: usize,
            at_point: Point,
            percent_along: Percent,
        ) -> Result<()> {
            assert_eq!(
                multiline_intersects_segment(&ml, &sg)?,
                Isxn::Some(
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
        fn one_intersection_ml_alongsharedsegment_sg(
            ml: Multiline,
            sg: Segment,
            index: usize,
            at_point: Point,
            ml_pct_along: Percent,
            sg_pct_along: Percent,
        ) -> Result<()> {
            assert_eq!(
                multiline_intersects_segment(&ml, &sg)?,
                Isxn::Some(
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

        // TODO(ambuc): multiline_and_segment special case tests

        // here is the fun stuff.
        #[test]
        fn two_intersections_at_segment_bookends() -> Result<()> {
            let ml = Multiline([*A, *C, *I]);
            let sg = Segment(*A, *I);

            assert_eq!(
                multiline_intersects_segment(&ml, &sg)?,
                Isxn::Some(
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
                )
            );
            Ok(())
        }

        #[test]
        fn two_intersections_at_segment_bookends_2() -> Result<()> {
            let ml = Multiline([*A, *C, *I]);
            let sg = Segment(*B, *F);

            assert_eq!(
                multiline_intersects_segment(&ml, &sg)?,
                Isxn::Some(
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
                )
            );
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

        #[test_case(Multiline([*A, *B, *C]), Multiline([*D, *E, *F]), Isxn::None; "none 01")]
        #[test_case(Multiline([*A, *B, *C]), Multiline([*G, *H, *I]), Isxn::None; "none 02")]
        #[test_case(Multiline([*A, *E, *I]), Multiline([*B, *F]), Isxn::None; "none diagonal")]
        #[test_case(
            Multiline([*A, *B, *C]),
            Multiline([*A, *D, *G]),
            Isxn::Some(
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
            Isxn::Some(
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
            Isxn::Some(
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
            Isxn::Some(
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
            Isxn::Some(
                Opinion::Multiline(nonempty![
                    MultilineOpinion::AtPointAlongSharedSegment { index: 0, at_point: *E, percent_along: Percent::Val(0.5) }
                ]),
                Opinion::Multiline(nonempty![
                    MultilineOpinion::AtPointAlongSharedSegment { index: 0, at_point: *E, percent_along: Percent::Val(0.5) }
                ])
            );
            "crosshairs"
        )]
        // #[test_case(
        //     Multiline([*A, *B, *C]),
        //     Multiline([*A, *B, *E]),
        //     Isxn::None;
        //     "partial collision"
        // )]
        fn isxn(ml1: Multiline, ml2: Multiline, expectation: Isxn) -> Result<()> {
            assert_eq!(multiline_intersects_multiline(&ml1, &ml2)?, expectation);
            Ok(())
        }
    }
}
