//! Default frames.

use anyhow::Result;
use plotz_geometry::{
    obj::Obj,
    shapes::{pg::Pg, pt::Pt},
    style::Style,
};

/// Makes a frame given (width, height) and (x,y) offset.
pub fn make_frame_pg((w, h): (f64, f64), offset: impl Into<Pt>) -> Result<Pg> {
    let mut p = Pg([(0.0, 0.0), (0.0, w), (h, w), (h, 0.0)])? + offset;
    p.orient_curve_positively();
    Ok(p)
}

/// Makes a frame given (width, height) and (x,y) offset.
pub fn make_frame(wh: (f64, f64), offset: impl Into<Pt>) -> Result<(Obj, Style)> {
    Ok((
        Obj::Pg(make_frame_pg(wh, offset)?),
        Style {
            thickness: 5.0,
            ..Default::default()
        },
    ))
}

/// Makes a frame at (0,0) with dims (w,h) which is set in on all faces by |margin|.
pub fn make_frame_with_margin((w, h): (f64, f64), margin: f64) -> Result<(Obj, Style)> {
    make_frame((w - 2.0 * margin, h - 2.0 * margin), Pt(margin, margin))
}
