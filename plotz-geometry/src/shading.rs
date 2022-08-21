//! Shading and crosshatching algorithms.
use crate::{
    bounded::{Bounded, BoundingBoxError},
    point::Pt,
    polygon::{CropToPolygonError, Polygon, PolygonKind},
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

// Gap controls how far to step between crosshatched lines
// Slope controls the angle of the lines.
fn _shade_polygon(
    gap: f64,
    slope: f64,
    polygon: &Polygon,
) -> Result<Vec<Segment>, ShadePolygonError> {
    if polygon.kind == PolygonKind::Open {
        return Err(ShadePolygonError::PolygonIsOpen);
    }

    let epsilon = Pt(0.000001_f64, 0.000001_f64);
    let bbox = polygon.bbox()?;
    let mut segments: Vec<Segment> = vec![];

    let slope_height = bbox.width() / slope;
    let mut i: Pt = bbox.bl_bound() - Pt(0.0, slope_height) - epsilon;
    let mut f: Pt = bbox.br_bound();

    while [i, f].iter().any(|p| p.y.0 <= bbox.top_bound()) {
        let full_stroke = Segment(i, f);
        let cropped_strokes = polygon.as_frame_to_segment(&full_stroke)?;
        segments.extend(cropped_strokes.iter());
        i.y.0 += gap;
        f.y.0 += gap;
    }
    Ok(segments)
}

#[cfg(test)]
mod tests {
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
            _shade_polygon(/*gap */ 1.0, /*slope=*/ 1.0, &frame).unwrap(),
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
            _shade_polygon(/*gap */ 0.5, /*slope=*/ 1.0, &frame).unwrap(),
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
            _shade_polygon(/*gap */ 0.5, /*slope=*/ 0.5, &frame).unwrap(),
            vec![
                Segment(Pt(0.5, 0.0), Pt(1.0, 0.25)),
                Segment(Pt(0.0, 0.0), Pt(1.0, 0.5)),
                Segment(Pt(0.0, 0.25), Pt(1.0, 0.75)),
                Segment(Pt(0.0, 0.5), Pt(1.0, 1.0)),
                Segment(Pt(0.0, 0.75), Pt(0.5, 1.0)),
            ],
        );
    }
}
