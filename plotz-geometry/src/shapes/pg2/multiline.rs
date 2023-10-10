//! A shortcut for constructing a multiline.

use super::{CurveOrientation, Pg2, PolygonKind};
use crate::shapes::pt2::Pt2;
use thiserror::Error;

/// A general error arising from trying to construct a Multiline.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum MultilineConstructorError {
    /// It is not possible to construct a multiline from one or fewer points.
    #[error("It is not possible to construct a multiline from one or fewer points.")]
    OneOrFewerPoints,
}

/// Constructor for multilines. Multilines must have at least one line, so they
/// must have two or more points. Constructing a multiline from one or fewer
/// points will result in a MultilineConstructorError.
#[allow(non_snake_case)]
pub fn Multiline(
    a: impl IntoIterator<Item = impl Into<Pt2>>,
) -> Result<Pg2, MultilineConstructorError> {
    let pts: Vec<Pt2> = a.into_iter().map(|x| x.into()).collect();
    if pts.len() <= 1 {
        return Err(MultilineConstructorError::OneOrFewerPoints);
    }

    let mut p = Pg2 {
        pts,
        kind: PolygonKind::Open,
    };
    if p.get_curve_orientation() == Some(CurveOrientation::Negative) {
        p.orient_curve_positively();
    }
    Ok(p)
}
