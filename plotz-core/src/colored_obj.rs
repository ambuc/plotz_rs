use plotz_color::ColorRGB;
use plotz_geometry::bounded::Bounded;
use plotz_geometry::polygon::Polygon;
use plotz_geometry::segment::Segment;

#[derive(Debug, PartialEq)]
pub enum Obj {
    Polygon(Polygon),
    Segment(Segment),
}
impl Obj {
    pub fn is_empty(&self) -> bool {
        match self {
            Obj::Polygon(p) => p.pts.is_empty(),
            Obj::Segment(_) => false,
        }
    }
}
impl Bounded for Obj {
    fn right_bound(&self) -> f64 {
        match self {
            Obj::Polygon(p) => p.right_bound(),
            Obj::Segment(s) => s.right_bound(),
        }
    }

    fn left_bound(&self) -> f64 {
        match self {
            Obj::Polygon(p) => p.left_bound(),
            Obj::Segment(s) => s.left_bound(),
        }
    }

    fn top_bound(&self) -> f64 {
        match self {
            Obj::Polygon(p) => p.top_bound(),
            Obj::Segment(s) => s.top_bound(),
        }
    }

    fn bottom_bound(&self) -> f64 {
        match self {
            Obj::Polygon(p) => p.bottom_bound(),
            Obj::Segment(s) => s.bottom_bound(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ColoredObj {
    pub obj: Obj,
    pub color: ColorRGB,
}
