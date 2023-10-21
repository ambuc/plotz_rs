//! A shortcut for constructing a multiline.

use super::{CurveOrientation, Pg, PolygonKind};
use crate::shapes::pt::Pt;
use anyhow::{anyhow, Result};

/// Constructor for multilines. Multilines must have at least one line, so they
/// must have two or more points. Constructing a multiline from one or fewer
/// points will result in a MultilineConstructorError.
#[allow(non_snake_case)]
pub fn Multiline(a: impl IntoIterator<Item = impl Into<Pt>>) -> Result<Pg> {
    let pts: Vec<Pt> = a.into_iter().map(|x| x.into()).collect();
    if pts.len() <= 1 {
        return Err(anyhow!("one or fewer points"));
    }

    let mut p = Pg {
        pts,
        kind: PolygonKind::Open,
    };
    if p.get_curve_orientation() == Some(CurveOrientation::Negative) {
        p.orient_curve_positively();
    }
    Ok(p)
}
