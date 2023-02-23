//! An annotated object with color and thickness.

use plotz_color::{ColorRGB, BLACK};
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

impl ColoredObj {
    /// from an object.
    pub fn from_obj(obj: Obj) -> ColoredObj {
        ColoredObj {
            obj,
            color: BLACK,
            thickness: 1.0,
        }
    }

    /// from a polygon.
    pub fn from_polygon(p: Polygon) -> ColoredObj {
        Self::from_obj(Obj::Polygon(p))
    }

    /// from a segment.
    pub fn from_segment(s: Segment) -> ColoredObj {
        Self::from_obj(Obj::Segment(s))
    }

    /// with a color.
    pub fn with_color(self, color: ColorRGB) -> ColoredObj {
        ColoredObj {
            obj: self.obj,
            color: color,
            thickness: self.thickness,
        }
    }
}
