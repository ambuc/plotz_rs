//! A 2D ray.

use std::f64::consts::TAU;

use crate::{
    isxn::IsxnResult,
    shapes::{
        pt2::{PolarPt, Pt2},
        sg2::Sg2,
    },
};

/// A ray which emits from a pt and goes in a direction.
pub struct Ry2 {
    pt: Pt2,
    angle_out_rad: f64,
}

/// An alternate constructor for rays.
///
/// As a favor, we mod the incoming |angle_out_rad|  by TAU.
#[allow(non_snake_case)]
pub fn Ry2(pt: Pt2, angle_out_rad: f64) -> Ry2 {
    Ry2 {
        pt,
        angle_out_rad: angle_out_rad % TAU,
    }
}

impl Ry2 {
    /// Returns if one ray intersects another.
    pub fn intersects(&self, other: &Ry2) -> Option<IsxnResult> {
        let self_sg = Sg2(self.pt, self.pt + PolarPt(10.0, self.angle_out_rad));
        let other_sg = Sg2(other.pt, other.pt + PolarPt(10.0, other.angle_out_rad));
        self_sg.intersects(&other_sg)
    }

    /// Returns if one ray intersects a segment.
    pub fn intersects_sg(&self, other: &Sg2) -> Option<IsxnResult> {
        let self_sg = Sg2(self.pt, self.pt + PolarPt(10.0, self.angle_out_rad));
        self_sg.intersects(&other)
    }
}
