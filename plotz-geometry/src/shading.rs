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
fn shade_polygon(
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

    // slope = rise / run
    let slope_height = bbox.width() / slope;
    let mut i: Pt = bbox.tr_bound() - Pt(bbox.width(), slope_height) - epsilon;
    let mut f: Pt = i + Pt(bbox.width(), slope_height) + epsilon;
    if slope < 0.0 {
        i += Pt(0.0, slope_height);
        f += Pt(0.0, slope_height);
    }

    while [i.y, f.y].iter().any(|x| x.0 < bbox.bottom_bound()) {
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

    fn approx_eq_segments<'a>(
        a: impl Iterator<Item = &'a Segment>,
        b: impl Iterator<Item = &'a Segment>,
    ) {
        for (i, j) in a.zip(b) {
            approx_eq_segment(i, j);
        }
    }

    #[test]
    fn test_shade_square() {
        // ^ y
        // |
        // 4 - - + - - + - - + - - + - - +
        // |xxxxx|xxxxx|xxxxx| .   |xxxxx|
        // |xxxxx|xxxxx|xxxxx| .   |xxxxx|
        // 3 - - + - - + - - + - - + - - +
        // |xxxxx| .   |xxxxx| .   |xxxxx|
        // |xxxxx| .   |xxxxx| .   |xxxxx|
        // 2OOOOOOOOOOOOOOOOOOOOOOOOOOOOOO
        // |xxxxx| .   |xxxxx| .   |xxxxx|
        // |xxxxx| .   |xxxxx| .   |xxxxx|
        // 1 - - + - - + - - + - - + - - +
        // |xxxxx| .   |xxxxx|xxxxx|xxxxx|
        // |xxxxx| .   |xxxxx|xxxxx|xxxxx|
        // 0 - - 1 - - 2 - - 3 - - 4 - - 5 -> x

        let frame = Polygon([
            Pt(0, 0),
            Pt(1, 0),
            Pt(1, 3),
            Pt(2, 3),
            Pt(2, 0),
            Pt(5, 0),
            Pt(5, 4),
            Pt(4, 4),
            Pt(4, 1),
            Pt(3, 1),
            Pt(3, 5),
            Pt(0, 5),
        ])
        .unwrap();

        approx_eq_segments(
            shade_polygon(/*gap=*/ 1.0, /*slope=*/ 1.0, &frame)
                .unwrap()
                .iter(),
            vec![Segment(Pt(0, 0), Pt(1, 1)), Segment(Pt(2, 2), Pt(3, 3))].iter(),
        );
    }
}
