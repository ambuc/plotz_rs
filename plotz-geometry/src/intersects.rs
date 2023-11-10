#![allow(missing_docs)]

use crate::{
    interpolate::interpolate_2d_checked,
    obj2::Obj2,
    shapes::{point::Point, segment::Segment},
    utils::Percent,
};
use anyhow::Result;
use float_cmp::approx_eq;

pub enum PolygonIntersectionResult {
    AtPointWithIndex(usize),
    AlongSegmentWithIndex(usize, Percent),
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Opinion {
    Point,
    Segment(
        // The point at which it occurred.
        Point,
        // The percentage of the way along this segment which it occurred.
        Percent,
    ),
    Polygon(),
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum SpecialCase {
    PointsAreTheSame,
    LineSegmentsAreTheSame,
    LineSegmentsAreTheSameButReversed,
    LineSegmentsAreColinear,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Isxn {
    SpecialCase(SpecialCase),
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

pub fn intersects(a: &Obj2, b: &Obj2) -> Result<Isxn> {
    match (a, b) {
        (Obj2::Point(pa), Obj2::Point(pb)) => intersects_pt_pt(pa, pb),

        (Obj2::Segment(s), Obj2::Point(p)) => intersects_sg_pt(s, p),
        (Obj2::Point(p), Obj2::Segment(s)) => intersects_sg_pt(s, p).map(Isxn::flip),

        (Obj2::Segment(sa), Obj2::Segment(sb)) => intersects_sg_sg(sa, sb),

        _ => unimplemented!(),
    }
}

pub fn intersects_pt_pt(a: &Point, b: &Point) -> Result<Isxn> {
    if a == b {
        Ok(Isxn::SpecialCase(SpecialCase::PointsAreTheSame))
    } else {
        Ok(Isxn::None)
    }
}

pub fn intersects_sg_pt(s: &Segment, p: &Point) -> Result<Isxn> {
    if s.i == *p {
        Ok(Isxn::Some(
            Opinion::Segment(*p, Percent::Zero),
            Opinion::Point,
        ))
    } else if s.f == *p {
        Ok(Isxn::Some(
            Opinion::Segment(*p, Percent::One),
            Opinion::Point,
        ))
    } else if approx_eq!(
        f64,
        s.abs(),
        Segment(s.i, *p).abs() + Segment(*p, s.f).abs()
    ) {
        Ok(Isxn::Some(
            Opinion::Segment(*p, interpolate_2d_checked(s.i, s.f, *p)?),
            Opinion::Point,
        ))
    } else {
        Ok(Isxn::None)
    }
}

pub fn intersects_sg_sg(sa: &Segment, sb: &Segment) -> Result<Isxn> {
    if sa == sb {
        return Ok(Isxn::SpecialCase(SpecialCase::LineSegmentsAreTheSame));
    }

    if *sa == sb.flip() {
        return Ok(Isxn::SpecialCase(
            SpecialCase::LineSegmentsAreTheSameButReversed,
        ));
    }

    let sai_in_sb = matches!(intersects_sg_pt(sb, &sa.i)?, Isxn::Some(_, _));
    let saf_in_sb = matches!(intersects_sg_pt(sb, &sa.f)?, Isxn::Some(_, _));
    let sbi_in_sa = matches!(intersects_sg_pt(sa, &sb.i)?, Isxn::Some(_, _));
    let sbf_in_sa = matches!(intersects_sg_pt(sa, &sb.f)?, Isxn::Some(_, _));

    dbg!(sa.slope(), sb.slope());
    if (sa.slope() == sb.slope() || sa.slope() == sb.flip().slope())
        && ((sai_in_sb && saf_in_sb)
            || (sbi_in_sa && sbf_in_sa)
            || ((sai_in_sb || saf_in_sb) && (sbi_in_sa || sbf_in_sa)))
    {
        return Ok(Isxn::SpecialCase(SpecialCase::LineSegmentsAreColinear));
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
            Opinion::Segment(pt, interpolate_2d_checked(sa.i, sa.f, pt)?),
            Opinion::Segment(pt, interpolate_2d_checked(sb.i, sb.f, pt)?),
        ));
    }

    Ok(Isxn::None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::lazy_static;

    mod sg_sg {
        use super::*;

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

        #[test]
        fn the_same() -> Result<()> {
            for i in &[*A, *B, *C] {
                for j in &[*D, *E, *F] {
                    assert_eq!(
                        intersects_sg_sg(&Segment(*i, *j), &Segment(*i, *j))?,
                        Isxn::SpecialCase(SpecialCase::LineSegmentsAreTheSame)
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
                        intersects_sg_sg(&Segment(*i, *j), &Segment(*j, *i))?,
                        Isxn::SpecialCase(SpecialCase::LineSegmentsAreTheSameButReversed)
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
                            intersects_sg_sg(sa, sb)?,
                            Isxn::SpecialCase(SpecialCase::LineSegmentsAreColinear)
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
                        intersects_sg_sg(&Segment(p0, p1), &Segment(p1, *p2))?,
                        Isxn::Some(
                            Opinion::Segment(p1, Percent::One),
                            Opinion::Segment(p1, Percent::Zero)
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
                        intersects_sg_sg(sa, sb)?,
                        Isxn::Some(
                            Opinion::Segment(*E, Percent::Val(0.5)),
                            Opinion::Segment(*E, Percent::Val(0.5))
                        )
                    );
                }
            }

            Ok(())
        }
    }
}
