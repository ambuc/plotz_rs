//! An annotated object with color and thickness.

use plotz_color::ColorRGB;
use plotz_geometry::bounded::Bounded;
use plotz_geometry::polygon::Polygon;
use plotz_geometry::segment::Segment;

/// Either a polygon or a segment.
#[derive(Debug, PartialEq)]
pub enum Obj {
    /// A polygon.
    Polygon(Polygon),
    /// A segment.
    Segment(Segment),
}
impl Obj {
    /// Returns true if the object is empty (i.e. zero points)
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

/// An object with a color and thickness.
#[derive(Debug, PartialEq)]
pub struct ColoredObj {
    /// The object.
    pub obj: Obj,
    /// The color.
    pub color: ColorRGB,
    /// The thickness.
    pub thickness: f64,
}
