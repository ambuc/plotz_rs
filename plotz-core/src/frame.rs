//! Default frames.

use plotz_geometry::{obj2::Obj2, style::Style};

use plotz_geometry::shapes::{pg2::Pg2, pt2::Pt2};

/// Makes a frame given (width, height) and (x,y) offset.
pub fn make_frame_pg((w, h): (f64, f64), offset: Pt2) -> Pg2 {
    let mut p = Pg2([Pt2(0, 0), Pt2(0, w), Pt2(h, w), Pt2(h, 0)]) + offset;
    p.orient_curve_positively();
    p
}

/// Makes a frame given (width, height) and (x,y) offset.
pub fn make_frame(wh: (f64, f64), offset: Pt2) -> (Obj2, Style) {
    (
        Obj2::Pg2(make_frame_pg(wh, offset)),
        Style {
            thickness: 5.0,
            ..Default::default()
        },
    )
}

/// Makes a frame at (0,0) with dims (w,h) which is set in on all faces by |margin|.
pub fn make_frame_with_margin((w, h): (f64, f64), margin: f64) -> (Obj2, Style) {
    make_frame((w - 2.0 * margin, h - 2.0 * margin), Pt2(margin, margin))
}
