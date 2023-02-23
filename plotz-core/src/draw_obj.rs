//! An annotated object with color and thickness.

use plotz_color::{ColorRGB, BLACK};
use plotz_geometry::bounded::Bounded;
use plotz_geometry::polygon::Polygon;
use plotz_geometry::segment::Segment;

/// Either a polygon or a segment.
#[derive(Debug, PartialEq)]
pub enum DrawObjInner {
    /// A polygon.
    Polygon(Polygon),
    /// A segment.
    Segment(Segment),
}
impl DrawObjInner {
    /// Returns true if the object is empty (i.e. zero points)
    pub fn is_empty(&self) -> bool {
        match self {
            DrawObjInner::Polygon(p) => p.pts.is_empty(),
            DrawObjInner::Segment(_) => false,
        }
    }
}
impl Bounded for DrawObjInner {
    fn right_bound(&self) -> f64 {
        match self {
            DrawObjInner::Polygon(p) => p.right_bound(),
            DrawObjInner::Segment(s) => s.right_bound(),
        }
    }

    fn left_bound(&self) -> f64 {
        match self {
            DrawObjInner::Polygon(p) => p.left_bound(),
            DrawObjInner::Segment(s) => s.left_bound(),
        }
    }

    fn top_bound(&self) -> f64 {
        match self {
            DrawObjInner::Polygon(p) => p.top_bound(),
            DrawObjInner::Segment(s) => s.top_bound(),
        }
    }

    fn bottom_bound(&self) -> f64 {
        match self {
            DrawObjInner::Polygon(p) => p.bottom_bound(),
            DrawObjInner::Segment(s) => s.bottom_bound(),
        }
    }
}

/// An object with a color and thickness.
#[derive(Debug, PartialEq)]
pub struct DrawObj {
    /// The object.
    pub obj: DrawObjInner,
    /// The color.
    pub color: ColorRGB,
    /// The thickness.
    pub thickness: f64,
}

impl DrawObj {
    /// from an object.
    pub fn from_obj(obj: DrawObjInner) -> DrawObj {
        DrawObj {
            obj,
            color: BLACK,
            thickness: 1.0,
        }
    }

    /// from a polygon.
    pub fn from_polygon(p: Polygon) -> DrawObj {
        Self::from_obj(DrawObjInner::Polygon(p))
    }

    /// from a segment.
    pub fn from_segment(s: Segment) -> DrawObj {
        Self::from_obj(DrawObjInner::Segment(s))
    }

    /// with a color.
    pub fn with_color(self, color: ColorRGB) -> DrawObj {
        DrawObj {
            obj: self.obj,
            color,
            thickness: self.thickness,
        }
    }
}
