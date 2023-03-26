//! Default frames.

use {
    crate::draw_obj::DrawObj,
    plotz_color::BLACK,
    plotz_geometry::{point::Pt, polygon::Polygon},
};

/// Makes a frame given (width, height) and (x,y) offset.
pub fn make_frame((w, h): (f64, f64), offset: Pt) -> DrawObj {
    DrawObj::new(Polygon([Pt(0.0, 0.0), Pt(0.0, w), Pt(h, w), Pt(h, 0.0)]).unwrap() + offset)
        .with_color(&BLACK)
        .with_thickness(5.0)
}
