//! Shading and crosshatching algorithms.
use crate::{
    bounded::{Bounded, BoundingBoxError},
    crop::{CropToPolygonError, Croppable},
    point::Pt,
    polygon::{Polygon, PolygonKind},
    segment::Segment,
};

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
pub struct ShadeConfig {
    /// The gap between lines
    pub gap: f64,
    /// The slope of a line
    pub slope: f64,
    /// The thickness of a line (SVG only.)
    pub thickness: f64,
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

    let epsilon = Pt(0.000_001_f64, 0.000_001_f64);
    let bbox = polygon.bbox()?;
    let mut segments: Vec<Segment> = vec![];

    let mut slope_height = bbox.width() / config.slope;
    if config.slope < 0.0 {
        slope_height *= -1.0;
    }

    let mut i: Pt = bbox.bl_bound() - Pt(0.0, slope_height) - epsilon;
    let mut f: Pt = bbox.br_bound() + epsilon;

    while [i, f].iter().any(|p| p.y.0 <= bbox.top_bound()) {
        let full_stroke = Segment(i, f);
        let cropped_strokes = full_stroke.crop_to(polygon)?;
        segments.extend(cropped_strokes.iter());
        // segments.push(full_stroke);
        i.y.0 += config.gap;
        f.y.0 += config.gap;
    }
    Ok(segments)
}

#[cfg(test)]
mod tests {
    use crate::crop::PointLoc;
    use float_cmp::approx_eq;

    use super::*;

    fn approx_eq_pt(a: Pt, b: Pt) {
        approx_eq!(f64, a.x.0, b.x.0);
        approx_eq!(f64, a.y.0, b.y.0);
    }

    fn approx_eq_segment(a: &Segment, b: &Segment) {
        approx_eq_pt(a.i, b.i);
        approx_eq_pt(a.f, b.f);
    }

    fn approx_eq_segments(a: Vec<Segment>, b: Vec<Segment>) {
        assert_eq!(a.len(), b.len(), "a {:?} b {:?}", a, b);
        for (i, j) in a.iter().zip(b.iter()) {
            approx_eq_segment(i, j);
        }
    }

    #[test]
    fn test_shade_square_gap_1_0_slope_1_0() {
        let frame = Polygon([Pt(0, 0), Pt(1, 0), Pt(1, 1), Pt(0, 1)]).unwrap();

        // +-----+-----/
        // | .   | . / |
        // | .   | /   |
        // +-----/-----+
        // |   / | .   |
        // | /   | .   |
        // /-----+-----+
        approx_eq_segments(
            shade_polygon(
                &ShadeConfig {
                    gap: 1.0,
                    slope: 1.0,
                    thickness: 1.0,
                },
                &frame,
            )
            .unwrap(),
            vec![Segment(Pt(0, 0), Pt(1, 1))],
        );
    }

    #[test]
    fn test_shade_square_gap_0_5_slope_1_0() {
        let frame = Polygon([Pt(0, 0), Pt(1, 0), Pt(1, 1), Pt(0, 1)]).unwrap();

        // +-----/-----/
        // | . / | . / |
        // | /   | /   |
        // /-----/-----/
        // |   / | . / |
        // | /   | /   |
        // /-----/-----+
        approx_eq_segments(
            shade_polygon(
                &ShadeConfig {
                    gap: 0.5,
                    slope: 1.0,
                    thickness: 1.0,
                },
                &frame,
            )
            .unwrap(),
            vec![
                Segment(Pt(0.5, 0.0), Pt(1.0, 0.5)),
                Segment(Pt(0.0, 0.0), Pt(1.0, 1.0)),
                Segment(Pt(0.0, 0.5), Pt(0.5, 1.0)),
            ],
        );
    }

    #[test]
    fn test_shade_square_gap_0_5_slope_0_5() {
        let frame = Polygon([Pt(0, 0), Pt(1, 0), Pt(1, 1), Pt(0, 1)]).unwrap();

        // +-----/-----/
        // | . / | . / |
        // | /   | /   |
        // / .   / .   /
        // | . / | . / |
        // | /   | /   |
        // /-----/-----/
        // | . / | . / |
        // | /   | /   |
        // / .   / .   /
        // | . / | . / |
        // | /   | /   |
        // /-----/-----+
        approx_eq_segments(
            shade_polygon(
                &ShadeConfig {
                    gap: 0.5,
                    slope: 0.5,
                    thickness: 1.0,
                },
                &frame,
            )
            .unwrap(),
            vec![
                Segment(Pt(0.5, 0.0), Pt(1.0, 0.25)),
                Segment(Pt(0.0, 0.0), Pt(1.0, 0.5)),
                Segment(Pt(0.0, 0.25), Pt(1.0, 0.75)),
                Segment(Pt(0.0, 0.5), Pt(1.0, 1.0)),
                Segment(Pt(0.0, 0.75), Pt(0.5, 1.0)),
            ],
        );
    }

    #[test]
    fn test_parks() {
        let real_area_01 = Polygon([
            Pt(228.17, 202.35),
            Pt(231.21, 212.64),
            Pt(232.45, 228.76),
            Pt(231.67, 257.09),
            Pt(230.63, 265.17),
            Pt(263.66, 335.37),
            Pt(261.85, 336.27),
            Pt(295.65, 404.87),
            Pt(298.24, 409.14),
            Pt(302.39, 413.67),
            Pt(305.92, 412.20),
            Pt(309.33, 417.90),
            Pt(311.03, 417.06),
            Pt(312.99, 420.06),
            Pt(318.55, 420.99),
            Pt(322.66, 420.45),
            Pt(325.57, 419.13),
            Pt(343.70, 406.83),
            Pt(336.17, 404.87),
            Pt(230.61, 185.93),
            Pt(228.83, 189.47),
            Pt(227.19, 195.84),
            Pt(228.17, 202.35),
        ])
        .unwrap();
        let real_area_02 = Polygon([
            Pt(37.83, 551.40),
            Pt(38.10, 555.84),
            Pt(115.16, 549.86),
            Pt(146.58, 546.32),
            Pt(169.84, 541.22),
            Pt(190.05, 535.88),
            Pt(207.79, 529.04),
            Pt(222.12, 522.34),
            Pt(233.62, 513.70),
            Pt(242.35, 505.33),
            Pt(249.12, 493.23),
            Pt(234.33, 487.89),
            Pt(222.85, 500.58),
            Pt(212.50, 507.13),
            Pt(200.93, 511.30),
            Pt(189.45, 514.39),
            Pt(137.68, 523.69),
            Pt(140.99, 532.34),
            Pt(48.62, 549.59),
            Pt(37.83, 551.40),
        ])
        .unwrap();
        let real_area_03 = Polygon([
            Pt(223.76, 256.64),
            Pt(225.95, 252.56),
            Pt(226.22, 240.08),
            Pt(225.95, 228.40),
            Pt(224.53, 215.67),
            Pt(222.33, 205.41),
            Pt(208.05, 212.91),
            Pt(190.47, 181.04),
            Pt(188.78, 181.85),
            Pt(181.85, 169.36),
            Pt(165.84, 175.85),
            Pt(181.86, 204.54),
            Pt(223.76, 256.64),
        ])
        .unwrap();

        for area in [&real_area_01, &real_area_02, &real_area_03] {
            for segment in shade_polygon(
                &ShadeConfig {
                    gap: 5.0,
                    slope: 10.0,
                    thickness: 1.0,
                },
                &area,
            )
            .unwrap()
            {
                for pt in [&segment.i, &segment.f] {
                    assert!(!matches!(area.contains_pt(pt).unwrap(), PointLoc::Outside));
                }
            }
        }
    }
}
