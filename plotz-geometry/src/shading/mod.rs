//! Shading and crosshatching algorithms. Updated version.

pub mod shade_config;

use crate::{
    bounded::Bounded,
    crop::Croppable,
    shading::shade_config::ShadeConfig,
    shapes::{pg::Pg, pt::Pt, sg::Sg},
};
use anyhow::Result;
use float_ord::FloatOrd;

fn compute_vertical_step(gap: f64, slope: f64) -> f64 {
    gap * (slope.powi(2) + 1.0).sqrt()
}

/// Gap controls how far to step between crosshatched lines
/// Slope controls the angle of the lines.
pub fn shade_polygon(config: &ShadeConfig, polygon: &Pg) -> Result<Vec<Sg>> {
    let bounds = polygon.bounds()?;
    let mut segments: Vec<Sg> = vec![];

    let xnudge = Pt(1, -1);
    let ynudge = Pt(-1, 1);
    let mut line = if config.slope > 0.0 {
        Sg(
            bounds.tl() - xnudge,
            bounds.tl() + (bounds.w(), bounds.w() * config.slope) + xnudge,
        )
    } else {
        Sg(
            bounds.tr() - ynudge,
            bounds.tr() + (-1.0 * bounds.w(), -1.0 * bounds.w() * config.slope) + ynudge,
        )
    };

    let step = compute_vertical_step(config.gap, config.slope);

    while FloatOrd(line.i.y) > FloatOrd(bounds.b()) || FloatOrd(line.f.y) > FloatOrd(bounds.b()) {
        let cropped_strokes = line.crop_to(polygon)?;
        segments.extend(cropped_strokes.iter());
        // segments.push(line);

        line -= (0, step);
    }

    if config.switchback {
        Ok(segments
            .iter()
            .zip(segments.iter().skip(1))
            .zip([true, false].iter().cycle())
            .map(|((sa, sb), should_alternate)| {
                if *should_alternate {
                    Sg(sa.i, sb.f)
                } else {
                    Sg(sb.i, sa.f)
                }
            })
            .collect())
    } else {
        Ok(segments)
    }
}
