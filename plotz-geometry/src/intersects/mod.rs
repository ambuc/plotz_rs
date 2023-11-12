#![allow(missing_docs)]

pub mod opinion;
pub mod specialcase;

use self::{
    opinion::{MultilineOpinion, Opinion, SegmentOpinion},
    specialcase::{General, MultilineAndSegment, TwoPoints, TwoSegments},
};
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
    SpecialCase(General),
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
    // multiline || ✔️  | ~  |    |
    // ==========++====+====+====+==
    //
    match (a, b) {
        (Obj2::Point(pa), Obj2::Point(pb)) => point_intersects_point(pa, pb),

        (Obj2::Segment(s), Obj2::Point(p)) => segment_intersects_point(s, p),
        (Obj2::Point(p), Obj2::Segment(s)) => segment_intersects_point(s, p).map(Isxn::flip),

        (Obj2::Segment(sa), Obj2::Segment(sb)) => segment_intersects_segment(sa, sb),

        (Obj2::Multiline(ml), Obj2::Point(p)) => multiline_intersects_point(ml, p),
        (Obj2::Point(p), Obj2::Multiline(ml)) => multiline_intersects_point(ml, p).map(Isxn::flip),

        (Obj2::Multiline(ml), Obj2::Segment(sg)) => multiline_intersects_segment(ml, sg),
        (Obj2::Segment(sg), Obj2::Multiline(ml)) => {
            multiline_intersects_segment(ml, sg).map(Isxn::flip)
        }

        (Obj2::Multiline(ml1), Obj2::Multiline(ml2)) => multiline_intersects_multiline(ml1, ml2),

        _ => unimplemented!(),
    }
}

pub fn point_intersects_point(a: &Point, b: &Point) -> Result<Isxn> {
    if a == b {
        Ok(Isxn::SpecialCase(General::TwoPoints(TwoPoints::Same)))
    } else {
        Ok(Isxn::None)
    }
}

pub fn segment_intersects_point(s: &Segment, p: &Point) -> Result<Isxn> {
    if s.i == *p {
        Ok(Isxn::Some(
            Opinion::Segment(nonempty![SegmentOpinion::AlongSegment {
                at_point: *p,
                percent_along: Percent::Zero,
            }]),
            Opinion::Point,
        ))
    } else if s.f == *p {
        Ok(Isxn::Some(
            Opinion::Segment(nonempty![SegmentOpinion::AlongSegment {
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
            Opinion::Segment(nonempty![SegmentOpinion::AlongSegment {
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
    if sa == sb {
        return Ok(Isxn::SpecialCase(General::TwoSegments(TwoSegments::Same)));
    }

    if *sa == sb.flip() {
        return Ok(Isxn::SpecialCase(General::TwoSegments(
            TwoSegments::SameButReversed,
        )));
    }

    let sai_in_sb = matches!(segment_intersects_point(sb, &sa.i)?, Isxn::Some(_, _));
    let saf_in_sb = matches!(segment_intersects_point(sb, &sa.f)?, Isxn::Some(_, _));
    let sbi_in_sa = matches!(segment_intersects_point(sa, &sb.i)?, Isxn::Some(_, _));
    let sbf_in_sa = matches!(segment_intersects_point(sa, &sb.f)?, Isxn::Some(_, _));

    if (sa.slope() == sb.slope() || sa.slope() == sb.flip().slope())
        && ((sai_in_sb && saf_in_sb)
            || (sbi_in_sa && sbf_in_sa)
            || ((sai_in_sb || saf_in_sb) && (sbi_in_sa || sbf_in_sa)))
    {
        return Ok(Isxn::SpecialCase(General::TwoSegments(
            TwoSegments::Colinear,
        )));
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
            Opinion::Segment(nonempty![SegmentOpinion::AlongSegment {
                at_point: pt,
                percent_along: interpolate_2d_checked(sa.i, sa.f, pt)?,
            }]),
            Opinion::Segment(nonempty![SegmentOpinion::AlongSegment {
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
        if let Isxn::Some(Opinion::Segment(sgs), _) = segment_intersects_point(sg, p)? {
            assert_eq!(sgs.len(), 1);
            match sgs.head {
                SegmentOpinion::AlongSegment {
                    at_point,
                    percent_along,
                } => {
                    sg_ops.push(match percent_along {
                        Percent::Zero => MultilineOpinion::AtPoint { index, at_point },
                        Percent::Val(_) => MultilineOpinion::AlongSharedSegment {
                            index,
                            at_point,
                            percent_along,
                        },
                        Percent::One => MultilineOpinion::AtPoint {
                            index: index + 1,
                            at_point,
                        },
                    });
                }
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
            // Handle Isxn::SpecialCase(_),
            Isxn::SpecialCase(General::TwoSegments(sc)) => {
                // return early. not guaranteed to find _all_ unusual
                // special-case intersections.
                return Ok(Isxn::SpecialCase(General::MultilineAndSegment(
                    MultilineAndSegment::SegmentInMultiline {
                        sc,
                        index: ml_sg_idx,
                    },
                )));
            }
            Isxn::SpecialCase(_) => {
                return Err(anyhow!("segment_intersects_segment should not have returned any SpecialCase besides General::TwoSegments(_)."));
            }

            // then Isxn::Some(_),
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

pub fn multiline_intersects_multiline(_: &Multiline, _: &Multiline) -> Result<Isxn> {
    unimplemented!()
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
                    Isxn::SpecialCase(General::TwoPoints(TwoPoints::Same))
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
                        Opinion::Segment(nonempty![SegmentOpinion::AlongSegment {
                            at_point: *i,
                            percent_along: Percent::Zero
                        }]),
                        Opinion::Point
                    )
                );
                assert_eq!(
                    segment_intersects_point(&Segment(*i, *f), f)?,
                    Isxn::Some(
                        Opinion::Segment(nonempty![SegmentOpinion::AlongSegment {
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
                        Opinion::Segment(nonempty![SegmentOpinion::AlongSegment {
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

        #[test]
        fn the_same() -> Result<()> {
            for i in &[*A, *B, *C] {
                for j in &[*D, *E, *F] {
                    assert_eq!(
                        segment_intersects_segment(&Segment(*i, *j), &Segment(*i, *j))?,
                        Isxn::SpecialCase(General::TwoSegments(TwoSegments::Same))
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
                        Isxn::SpecialCase(General::TwoSegments(TwoSegments::SameButReversed))
                    );
                }
            }
            Ok(())
        }

        #[test]
        fn the_same_colinear() -> Result<()> {
            for (i, j, k) in &[(*A, *B, *C), (*A, *E, *I), (*A, *D, *G)] {
                for (sa, sb) in &[
                    (Segment(*i, *j), Segment(*j, *k)),
                    (Segment(*i, *k), Segment(*j, *k)),
                    (Segment(*i, *j), Segment(*i, *k)),
                    (Segment(*j, *k), Segment(*i, *j)),
                ] {
                    for (sa, sb) in &[
                        (sa, sb),
                        (sa, &sb.flip()),
                        (&sa.flip(), sb),
                        (&sa.flip(), &sb.flip()),
                    ] {
                        assert_eq!(
                            segment_intersects_segment(sa, sb)?,
                            Isxn::SpecialCase(General::TwoSegments(TwoSegments::Colinear))
                        );
                    }
                }
            }
            Ok(())
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
                            Opinion::Segment(nonempty![SegmentOpinion::AlongSegment {
                                at_point: p1,
                                percent_along: Percent::One
                            }]),
                            Opinion::Segment(nonempty![SegmentOpinion::AlongSegment {
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
                            Opinion::Segment(nonempty![SegmentOpinion::AlongSegment {
                                at_point: *E,
                                percent_along: Percent::Val(0.5)
                            }]),
                            Opinion::Segment(nonempty![SegmentOpinion::AlongSegment {
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
                            Opinion::Multiline(nonempty![MultilineOpinion::AlongSharedSegment {
                                index: idx,
                                at_point: *pt,
                                percent_along: Percent::Val(0.5)
                            }]),
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

        //   ^
        //   |
        //   A  B  C
        //   |
        //   D  E  F
        //   |
        // --G--H--I->
        //   |

        #[test]
        fn no_intersections() -> Result<()> {
            // let ml = Multiline([*A, *C, *I]);
            // let sg = Segment(*G, *H);
            // assert_eq!(multiline_intersects_segment(&ml, &sg)?, Isxn::None);
            Ok(())
        }

        #[test]
        fn one_intersection_at_segment_start() -> Result<()> {
            Ok(())
        }
        #[test]
        fn one_intersection_at_segment_midpoint() -> Result<()> {
            Ok(())
        }
        #[test]
        fn one_intersection_at_segment_end() -> Result<()> {
            Ok(())
        }
        #[test]
        fn two_intersections_at_segment_bookends() -> Result<()> {
            Ok(())
        }

        #[test]
        fn two_intersections_along_segment() -> Result<()> {
            Ok(())
        }
    }
}