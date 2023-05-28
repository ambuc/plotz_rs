//! Default frames.

use {
    plotz_color::BLACK,
    plotz_geometry::{
        shapes::{pg2::Pg2, point::Pt},
        styled_obj2::StyledObj2,
    },
};

/// Makes a frame given (width, height) and (x,y) offset.
pub fn make_frame_pg((w, h): (f64, f64), offset: Pt) -> Pg2 {
    let mut p = Pg2([Pt(0.0, 0.0), Pt(0.0, w), Pt(h, w), Pt(h, 0.0)]) + offset;
    p.orient_curve_positively();
    p
}

/// Makes a frame given (width, height) and (x,y) offset.
pub fn make_frame(wh: (f64, f64), offset: Pt) -> StyledObj2 {
    StyledObj2::new(make_frame_pg(wh, offset))
        .with_color(&BLACK)
        .with_thickness(5.0)
}

/// Makes a frame at (0,0) with dims (w,h) which is set in on all faces by |margin|.
pub fn make_frame_with_margin((w, h): (f64, f64), margin: f64) -> StyledObj2 {
    make_frame((w - 2.0 * margin, h - 2.0 * margin), Pt(margin, margin))
}
