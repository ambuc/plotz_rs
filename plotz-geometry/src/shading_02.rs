//! Shading and crosshatching algorithms. Updated version.

use crate::{
    bounded::{Bounded, BoundingBoxError},
    point::Pt,
    polygon::{CropToPolygonError, PointLoc, Polygon, PolygonKind},
    segment::Segment,
    shading::ShadePolygonError,
    //
};

/// Config for controlling crosshatching.
pub struct ShadeConfig {
    /// The gap between lines.
    pub gap: f64,

    /// The slope of a line.
    /// zero is flat.
    /// 1.0 is diagonal northeast (southwest).
    /// -1.0 is diagonal northwest (southeast).
    /// infinity is straight up-and-down.
    pub slope: f64,

    /// The thickness of a line (SVG only.)
    pub thickness: f64,
}

fn compute_vertical_step(gap: f64, slope: f64) -> f64 {
    gap * (slope.powi(2) + 1.0).sqrt()
}

/// Gap controls how far to step between crosshatched lines
/// Slope controls the angle of the lines.
pub fn shade_polygon(
    config: &ShadeConfig,
    polygon: &Polygon,
) -> Result<Vec<Segment>, ShadePolygonError> {
    if polygon.kind == PolygonKind::Open {
        return Err(ShadePolygonError::PolygonIsOpen);
    }

    let bbox = polygon.bbox()?;
    let mut segments: Vec<Segment> = vec![];

    let mut line = if config.slope > 0.0 {
        Segment(
            bbox.tl_bound(),
            bbox.tl_bound() + Pt(bbox.width(), bbox.width() * config.slope),
        )
    } else {
        Segment(
            bbox.tr_bound(),
            bbox.tr_bound() + Pt(-1.0 * bbox.width(), -1.0 * bbox.width() * config.slope),
        )
    };

    let step = compute_vertical_step(config.gap, config.slope);

    while line.i.y > float_ord::FloatOrd(bbox.bottom_bound())
        || line.f.y > float_ord::FloatOrd(bbox.bottom_bound())
    {
        let cropped_strokes = polygon.as_frame_to_segment(&line)?;
        segments.extend(cropped_strokes.iter());

        line -= Pt(0.0, step);
    }

    Ok(segments)
}
