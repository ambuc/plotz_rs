//! Crop logic for polygons.

use {crate::isxn::IsxnResult, std::fmt::Debug};

/// An IsxnResult which knows the polygon segments of its two lines.
#[derive(PartialEq, Copy, Clone)]
pub struct AnnotatedIsxnResult {
    pub isxn_result: IsxnResult,
    pub a_segment_idx: usize,
    pub b_segment_idx: usize,
}

impl Debug for AnnotatedIsxnResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} on [segment #{:?} of a, segment #{:?} of b]",
            self.isxn_result, self.a_segment_idx, self.b_segment_idx
        )
    }
}
