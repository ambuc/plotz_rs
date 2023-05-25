//! Crop logic for polygons.

use {
    super::TryPolygon,
    crate::{
        crop::{CropType, PointLoc},
        isxn::{Intersection, IsxnResult, Pair, Which},
        point::Pt,
        polygon::Polygon,
    },
    approx::*,
    float_ord::FloatOrd,
    itertools::Itertools,
    petgraph::{
        dot::{Config, Dot},
        prelude::DiGraphMap,
        Direction,
        Direction::{Incoming, Outgoing},
    },
    std::fmt::Debug,
    tracing::*,
    typed_builder::TypedBuilder,
};

/// An IsxnResult which knows the polygon segments of its two lines.
#[derive(PartialEq, Copy, Clone)]
pub struct AnnotatedIsxnResult {
    pub isxn_result: IsxnResult,
    pub a_segment_idx: usize,
    pub b_segment_idx: usize,
}

impl Debug for AnnotatedIsxnResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let AnnotatedIsxnResult {
            isxn_result,
            a_segment_idx,
            b_segment_idx,
        } = self;
        write!(
            f,
            "{:?} on [segment #{:?} of a, segment #{:?} of b]",
            isxn_result, a_segment_idx, b_segment_idx
        )
    }
}
