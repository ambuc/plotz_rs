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
            Opinion::Segment(Percent::Val(interpolate_2d_checked(s.i, s.f, *p)?)),
            Opinion::Point,
        ))
    } else {
        Ok(Isxn::None)
    }
}
