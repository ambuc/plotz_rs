use crate::{
    bounded::{Bounded, BoundingBoxError},
    point::Pt,
    polygon::{CropToPolygonError, Polygon, PolygonKind},
    segment::Segment,
};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ShadePolygonError {
    #[error("polygon was open")]
    PolygonIsOpen,
    #[error("could not get bounding box of polygon")]
    BoundingBoxError(#[from] BoundingBoxError),
    #[error("could not crop stroke to polygon")]
    CropError(#[from] CropToPolygonError),
}

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
