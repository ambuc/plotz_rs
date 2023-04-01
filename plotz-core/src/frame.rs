//! Default frames.

use {
    plotz_color::BLACK,
    plotz_geometry::draw_obj::DrawObj,
    plotz_geometry::{point::Pt, polygon::Polygon},
};

/// Makes a frame given (width, height) and (x,y) offset.
pub fn make_frame_pg((w, h): (f64, f64), offset: Pt) -> Polygon {
    let mut p = Polygon([Pt(0.0, 0.0), Pt(0.0, w), Pt(h, w), Pt(h, 0.0)]).unwrap() + offset;
    p.orient_curve();
    p
}

pub fn make_frame(wh: (f64, f64), offset: Pt) -> DrawObj {
    DrawObj::new(make_frame_pg(wh, offset))
        .with_color(&BLACK)
        .with_thickness(5.0)
}
