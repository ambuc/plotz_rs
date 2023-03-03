//! An annotated object with color and thickness.

use crate::svg::{write_layer_to_svg, Size};
use anyhow::Error;
use itertools::Itertools;
use multimap::MultiMap;
use plotz_color::{ColorRGB, BLACK};
use plotz_geometry::bounded::Bounded;
use plotz_geometry::point::Pt;
use plotz_geometry::polygon::{Multiline, Polygon};
use plotz_geometry::segment::Segment;

/// Either a polygon or a segment.
#[derive(Debug, PartialEq, Clone)]
pub enum DrawObjInner {
    /// A point.
    Point(Pt),
    /// A polygon.
    Polygon(Polygon),
    /// A segment.
    Segment(Segment),
}
impl DrawObjInner {
    /// Returns true if the object is empty (i.e. zero points)
    pub fn is_empty(&self) -> bool {
        match self {
            DrawObjInner::Point(p) => false,
            DrawObjInner::Polygon(p) => p.pts.is_empty(),
            DrawObjInner::Segment(_) => false,
        }
    }
}
impl Bounded for DrawObjInner {
    fn right_bound(&self) -> f64 {
        match self {
            DrawObjInner::Point(p) => p.right_bound(),
            DrawObjInner::Polygon(p) => p.right_bound(),
            DrawObjInner::Segment(s) => s.right_bound(),
        }
    }

    fn left_bound(&self) -> f64 {
        match self {
            DrawObjInner::Point(p) => p.left_bound(),
            DrawObjInner::Polygon(p) => p.left_bound(),
            DrawObjInner::Segment(s) => s.left_bound(),
        }
    }

    fn top_bound(&self) -> f64 {
        match self {
            DrawObjInner::Point(p) => p.top_bound(),
            DrawObjInner::Polygon(p) => p.top_bound(),
            DrawObjInner::Segment(s) => s.top_bound(),
        }
    }

    fn bottom_bound(&self) -> f64 {
        match self {
            DrawObjInner::Point(p) => p.bottom_bound(),
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
    pub color: &'static ColorRGB,
    /// The thickness.
    pub thickness: f64,
}

impl DrawObj {
    /// from an object.
    pub fn from_obj(obj: DrawObjInner) -> DrawObj {
        DrawObj {
            obj,
            color: &BLACK,
            thickness: 1.0,
        }
    }

    /// from a pt
    pub fn from_pt(p: Pt) -> DrawObj {
        Self::from_obj(DrawObjInner::Point(p))
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
            color: color,
            thickness: self.thickness,
        }
    }
}

/// Many draw objs.
#[derive(Debug, Clone)]
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
    pub fn group_by_color(mut self) -> Vec<(&'static ColorRGB, Vec<DrawObj>)> {
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
                // join adjacent segments here
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
                DrawObjInner::Point(p) => {
                    f(p);
                }
                DrawObjInner::Polygon(p) => {
                    p.pts.iter_mut().for_each(|pt| f(pt));
                }
                DrawObjInner::Segment(s) => {
                    f(&mut s.i);
                    f(&mut s.f);
                }
            })
    }

    /// joins
    pub fn join_adjacent_segments(&mut self) {
        // seg
        let mut colors_and_draw_objs: Vec<(&'static ColorRGB, Vec<DrawObj>)> =
            self.clone().group_by_color();

        let new_paths: Vec<DrawObj> = colors_and_draw_objs
            .into_iter()
            .flat_map(|(color, draw_obj_vec)| {
                //
                let mut pts_to_pts: MultiMap<Pt, Pt> = MultiMap::new();

                for draw_obj in draw_obj_vec.iter() {
                    match draw_obj.obj {
                        DrawObjInner::Segment(s) => {
                            pts_to_pts.insert(s.i, s.f);
                        }
                        DrawObjInner::Point(_) | DrawObjInner::Polygon(_) => {
                            // do nothing
                        }
                    }
                }

                let mut new_paths: Vec<DrawObj> = vec![];

                while !pts_to_pts.is_empty() {
                    let mut adjacent_pts: Vec<Pt> = vec![];

                    let mut key: Pt = pts_to_pts.keys().next().unwrap().clone();
                    adjacent_pts.push(key);

                    while let Some(val) = pts_to_pts.get_vec_mut(&key).and_then(|v| v.pop()) {
                        adjacent_pts.push(val);
                        key = val;
                    }
                    pts_to_pts.remove(&key);

                    if adjacent_pts.len() >= 2 {
                        println!("pts: {:?}", adjacent_pts);
                        new_paths.push(
                            DrawObj::from_polygon(Multiline(adjacent_pts).unwrap())
                                .with_color(color),
                        );
                    }
                }

                new_paths
            })
            .collect();

        // rejoin
        self.draw_objs = new_paths;
    }
}
