//! Default frames.

use {
    crate::draw_obj::{DrawObj, DrawObjInner},
    plotz_color::BLACK,
    plotz_geometry::{point::Pt, polygon::Polygon},
};

/// Makes a frame given (width, height) and (x,y) offset.
pub fn make_frame((w, h): (f64, f64), offset: Pt) -> DrawObj {
    DrawObj {
        obj: DrawObjInner::Polygon(
            Polygon([Pt(0.0, 0.0), Pt(0.0, w), Pt(h, w), Pt(h, 0.0)]).unwrap() + offset,
        ),
        color: &BLACK,
        thickness: 5.0,
    }
}
