//! An annotated object with color and thickness.

use crate::svg::{write_layer_to_svg, Size};
use anyhow::Error;
use itertools::Itertools;
use plotz_color::{ColorRGB, BLACK};
use plotz_geometry::bounded::Bounded;
use plotz_geometry::point::Pt;
use plotz_geometry::polygon::Polygon;
use plotz_geometry::segment::Segment;

/// Either a polygon or a segment.
#[derive(Debug, PartialEq, Clone)]
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
#[derive(Debug, PartialEq, Clone)]
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
    pub fn with_color(self, color: &'static ColorRGB) -> DrawObj {
        DrawObj {
            obj: self.obj,
            color: *color,
            thickness: self.thickness,
        }
    }
}

/// Many draw objs.
pub struct DrawObjs {
    /// the objs.
    pub draw_objs: Vec<DrawObj>,
    /// the frame, maybe.
    pub frame: Option<DrawObj>,
}

impl DrawObjs {
    /// ctor from objs
    pub fn from_objs<O: IntoIterator<Item = DrawObj>>(objs: O) -> DrawObjs {
        DrawObjs {
            draw_objs: objs.into_iter().collect(),
            frame: None,
        }
    }

    /// with a frame
    pub fn with_frame(self, frame: DrawObj) -> DrawObjs {
        DrawObjs {
            draw_objs: self.draw_objs,
            frame: Some(frame),
        }
    }

    /// Sorts and groups the internal draw objects by color.
    pub fn group_by_color(mut self) -> Vec<(ColorRGB, Vec<DrawObj>)> {
        self.draw_objs.sort_by_key(|d_o| d_o.color);
        self.draw_objs
            .into_iter()
            .group_by(|a| a.color)
            .into_iter()
            .map(|(color, group)| (color, group.into_iter().collect::<Vec<_>>()))
            .collect()
    }

    /// writes out to a set of SVGs at a prefix.
    pub fn write_to_svg(self, size: Size, prefix: &str) -> Result<(), Error> {
        // all
        {
            let name = format!("{}_all.svg", prefix);
            let mut all = vec![];
            if let Some(frame) = self.frame.clone() {
                all.push(frame);
            }
            all.extend(self.draw_objs.clone());
            write_layer_to_svg(size, name, &all)?;
        }

        // frame
        {
            if let Some(frame) = self.frame.clone() {
                let _ = write_layer_to_svg(size, format!("{}_{}.svg", prefix, "frame"), &[frame]);
            }
        }

        // layers
        {
            for (i, (_color, draw_obj_vec)) in self.group_by_color().into_iter().enumerate() {
                let _num = write_layer_to_svg(size, format!("{}_{}.svg", prefix, i), &draw_obj_vec)
                    .expect("failed to write");
            }
        }

        Ok(())
    }

    /// apply a fn to each pt.
    pub fn mutate(&mut self, f: impl Fn(&mut Pt)) {
        self.draw_objs
            .iter_mut()
            .for_each(|d_o| match &mut d_o.obj {
                DrawObjInner::Polygon(p) => {
                    p.pts.iter_mut().for_each(|pt| f(pt));
                }
                DrawObjInner::Segment(s) => {
                    f(&mut s.i);
                    f(&mut s.f);
                }
            })
    }
}
