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
    Segment(Percent),
    Polygon(),
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum SpecialCase {
    PointsAreTheSame,
    LineSegmentsAreTheSame,
    LineSegmentsAreTheSameButReversed,
    LineSegmentsAreColinear,
}

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
        Ok(Isxn::Some(Opinion::Segment(Percent::Zero), Opinion::Point))
    } else if s.f == *p {
        Ok(Isxn::Some(Opinion::Segment(Percent::One), Opinion::Point))
    } else if approx_eq!(
        f64,
        s.abs(),
        Segment(s.i, *p).abs() + Segment(*p, s.f).abs()
    ) {
        Ok(Isxn::Some(
            Opinion::Segment(interpolate_2d_checked(s.i, s.f, *p)?),
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

    #[allow(clippy::nonminimal_bool)]
    if (sai_in_sb && saf_in_sb)
        || (sbi_in_sa && sbf_in_sa)
        || (saf_in_sb && sbi_in_sa)
        || (sbf_in_sa && saf_in_sb)
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
            Opinion::Segment(interpolate_2d_checked(sa.i, sa.f, pt)?),
            Opinion::Segment(interpolate_2d_checked(sb.i, sb.f, pt)?),
        ));
    }

    Ok(Isxn::None)
}
