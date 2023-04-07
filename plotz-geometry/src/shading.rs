//! Shading and crosshatching algorithms. Updated version.

use crate::{
    bounded::{Bounded, BoundingBoxError},
    crop::{Croppable, CropToPolygonError},
    point::Pt,
    polygon::{Polygon, PolygonKind},
    segment::Segment,
};
use typed_builder::TypedBuilder;

/// A general error arising from shading a polygon.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ShadePolygonError {
    /// The frame polygon was open, so shading its internal area is underspecified.
    #[error("The frame polygon was open, so shading its internal area is underspecified.")]
    PolygonIsOpen,
    /// An error arose trying to compute the bounding box of a polygon to shade.
    #[error("An error arose trying to compute the bounding box of a polygon to shade.")]
    BoundingBoxError(#[from] BoundingBoxError),
    /// An error arose trying to crop some stroke to a bounding polygon.
    #[error("An error arose trying to crop some stroke to a bounding polygon.")]
    CropError(#[from] CropToPolygonError),
}

/// Config for controlling crosshatching.
#[derive(Debug, Clone, TypedBuilder)]
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
    #[builder(default = 1.0)]
    pub thickness: f64,

    /// whether or not to zig zag.
    #[builder(default = false)]
    pub switchback: bool,
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

    let bounds = polygon.bounds();
    let mut segments: Vec<Segment> = vec![];

    let xnudge = Pt(1.0, -1.0);
    let ynudge = Pt(-1.0, 1.0);
    let mut line = if config.slope > 0.0 {
        Segment(
            bounds.tl_bound() - xnudge,
            bounds.tl_bound() + Pt(bounds.width(), bounds.width() * config.slope) + xnudge,
        )
    } else {
        Segment(
            bounds.tr_bound() - ynudge,
            bounds.tr_bound()
                + Pt(-1.0 * bounds.width(), -1.0 * bounds.width() * config.slope)
                + ynudge,
        )
    };

    let step = compute_vertical_step(config.gap, config.slope);

    while line.i.y > float_ord::FloatOrd(bounds.bottom_bound())
        || line.f.y > float_ord::FloatOrd(bounds.bottom_bound())
    {
        let cropped_strokes = line.crop_to(polygon)?;
        segments.extend(cropped_strokes.iter());
        // segments.push(line);

        line -= Pt(0.0, step);
    }

    if config.switchback {
        Ok(segments
            .iter()
            .zip(segments.iter().skip(1))
            .zip([true, false].iter().cycle())
            .map(|((sa, sb), should_alternate)| {
                if *should_alternate {
                    Segment(sa.i, sb.f)
                } else {
                    Segment(sb.i, sa.f)
                }
            })
            .collect())
    } else {
        Ok(segments)
    }
}
