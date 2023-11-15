//! A 2D ray.
#![allow(missing_docs)]

use crate::{
    intersection::IntersectionResult,
    intersects::{segment_intersects_segment, Isxn},
    shapes::{
        point::{Point, PolarPt},
        segment::Segment,
    },
};
use anyhow::Result;
use std::f64::consts::TAU;

/// A ray which emits from a pt and goes in a direction.
#[derive(Copy, Clone)]
pub struct Ray {
    pt: Point,
    angle_out_rad: f64,
}

/// An alternate constructor for rays.
///
/// As a favor, we mod the incoming |angle_out_rad|  by TAU.
#[allow(non_snake_case)]
pub fn Ray(pt: Point, angle_out_rad: f64) -> Ray {
    Ray {
        pt,
        angle_out_rad: angle_out_rad % TAU,
    }
}

impl Ray {
    /// Returns if one ray intersects another.
    pub fn intersects_old(&self, other: &Ray) -> Option<IntersectionResult> {
        let self_sg = Segment(self.pt, self.pt + PolarPt(10.0, self.angle_out_rad));
        let other_sg = Segment(other.pt, other.pt + PolarPt(10.0, other.angle_out_rad));
        self_sg.intersects(&other_sg)
    }

    pub fn intersects(&self, other: &Ray) -> Result<Isxn> {
        segment_intersects_segment(&self.to_sg(10.0), &other.to_sg(10.0))
    }

    pub fn intersects_sg(&self, other: &Segment) -> Result<Isxn> {
        segment_intersects_segment(&self.to_sg(10.0), other)
    }

    /// Returns a version of this ray rotated by |angle| rad.
    pub fn rotate(&self, angle: f64) -> Ray {
        Ray {
            pt: self.pt,
            angle_out_rad: (self.angle_out_rad + angle) % TAU,
        }
    }

    /// returns a Sg - keeps the initial point, and goes a distance |len| along
    /// the ray.
    pub fn to_sg(&self, len: f64) -> Segment {
        Segment {
            i: self.pt,
            f: self.pt + PolarPt(len, self.angle_out_rad),
        }
    }
}
