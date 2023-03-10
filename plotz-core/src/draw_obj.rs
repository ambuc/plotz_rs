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
    /// A character to be printed in SVG, at a point.
    Char(Pt, char),
    /// A group of other drawobjects.
    Group(Vec<DrawObjInner>),
}

impl DrawObjInner {
    /// Returns true if the object is empty (i.e. zero points)
    pub fn is_empty(&self) -> bool {
        match self {
            DrawObjInner::Polygon(p) => p.pts.is_empty(),
            DrawObjInner::Point(_) | DrawObjInner::Segment(_) | DrawObjInner::Char(_, _) => false,
            DrawObjInner::Group(dois) => dois.iter().all(|doi| doi.is_empty()),
        }
    }
    /// mutate a doi.
    pub fn mutate(&mut self, f: impl Fn(&mut Pt)) {
        match self {
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
            DrawObjInner::Char(pt, _ch) => {
                f(pt);
            }
            DrawObjInner::Group(dois) => {
                for doi in dois {
                    doi.mutate(&f);
                }
            }
        }
    }

    /// to iterator
    pub fn iter(&self) -> Box<dyn Iterator<Item = &Pt> + '_> {
        match &self {
            DrawObjInner::Char(ref pt, _) => Box::new(std::iter::once(pt)),
            DrawObjInner::Point(ref pt) => Box::new(std::iter::once(pt)),
            DrawObjInner::Polygon(pg) => Box::new(pg.pts.iter()),
            DrawObjInner::Segment(sg) => {
                Box::new(std::iter::once(&sg.i).chain(std::iter::once(&sg.f)))
            }
            DrawObjInner::Group(dois) => Box::new(dois.iter().map(|doi| doi.iter()).flatten()),
        }
    }
}
impl Bounded for DrawObjInner {
    fn right_bound(&self) -> f64 {
        match self {
            DrawObjInner::Point(p) => p.right_bound(),
            DrawObjInner::Polygon(p) => p.right_bound(),
            DrawObjInner::Segment(s) => s.right_bound(),
            DrawObjInner::Char(p, _ch) => p.right_bound(),
            DrawObjInner::Group(dos) => {
                dos.iter()
                    .map(|doi| float_ord::FloatOrd(doi.right_bound()))
                    .max()
                    .unwrap()
                    .0
            }
        }
    }

    fn left_bound(&self) -> f64 {
        match self {
            DrawObjInner::Point(p) => p.left_bound(),
            DrawObjInner::Polygon(p) => p.left_bound(),
            DrawObjInner::Segment(s) => s.left_bound(),
            DrawObjInner::Char(p, _ch) => p.left_bound(),
            DrawObjInner::Group(dos) => {
                dos.iter()
                    .map(|doi| float_ord::FloatOrd(doi.left_bound()))
                    .min()
                    .unwrap()
                    .0
            }
        }
    }

    fn top_bound(&self) -> f64 {
        match self {
            DrawObjInner::Point(p) => p.top_bound(),
            DrawObjInner::Polygon(p) => p.top_bound(),
            DrawObjInner::Segment(s) => s.top_bound(),
            DrawObjInner::Char(p, _ch) => p.top_bound(),
            DrawObjInner::Group(dos) => {
                dos.iter()
                    .map(|doi| float_ord::FloatOrd(doi.top_bound()))
                    .min()
                    .unwrap()
                    .0
            }
        }
    }

    fn bottom_bound(&self) -> f64 {
        match self {
            DrawObjInner::Point(p) => p.bottom_bound(),
            DrawObjInner::Polygon(p) => p.bottom_bound(),
            DrawObjInner::Segment(s) => s.bottom_bound(),
            DrawObjInner::Char(p, _ch) => p.bottom_bound(),
            DrawObjInner::Group(dos) => {
                dos.iter()
                    .map(|doi| float_ord::FloatOrd(doi.bottom_bound()))
                    .max()
                    .unwrap()
                    .0
            }
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

    /// from a character.
    pub fn from_char(p: Pt, ch: char) -> DrawObj {
        Self::from_obj(DrawObjInner::Char(p, ch))
    }

    /// from a group.
    pub fn from_group(dos: impl IntoIterator<Item = DrawObjInner>) -> DrawObj {
        Self::from_obj(DrawObjInner::Group(dos.into_iter().collect::<Vec<_>>()))
    }

    // builders

    /// with a color.
    pub fn with_color(self, color: &'static ColorRGB) -> DrawObj {
        DrawObj { color, ..self }
    }

    /// with a thickness.
    pub fn with_thickness(self, thickness: f64) -> DrawObj {
        DrawObj { thickness, ..self }
    }

    /// to iterator
    pub fn iter(&self) -> Box<dyn Iterator<Item = &Pt> + '_> {
        self.obj.iter()
    }
}

/// Many draw objs.
#[derive(Debug, Clone)]
pub struct DrawObjs {
    /// the objs.
    pub draw_obj_vec: Vec<DrawObj>,

    /// the frame, maybe.
    pub frame: Option<DrawObj>,
}

impl DrawObjs {
    /// ctor from objs
    pub fn from_objs<O: IntoIterator<Item = DrawObj>>(objs: O) -> DrawObjs {
        DrawObjs {
            draw_obj_vec: objs.into_iter().collect(),
            frame: None,
        }
    }

    /// with a frame
    pub fn with_frame(self, frame: DrawObj) -> DrawObjs {
        DrawObjs {
            frame: Some(frame),
            ..self
        }
    }

    /// Sorts and groups the internal draw objects by color.
    pub fn group_by_color(mut self) -> Vec<(&'static ColorRGB, Vec<DrawObj>)> {
        self.draw_obj_vec.sort_by_key(|d_o| d_o.color);
        self.draw_obj_vec
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
            all.extend(self.draw_obj_vec.clone());
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
        self.draw_obj_vec
            .iter_mut()
            .for_each(|d_o| d_o.obj.mutate(&f));
    }

    /// join adjacent segments to save on path draw time.
    pub fn join_adjacent_segments(&mut self) {
        self.draw_obj_vec = self
            .clone()
            .group_by_color()
            .into_iter()
            .flat_map(|(color, draw_obj_vec)| {
                //
                let mut mmap: MultiMap<Pt, Pt> = MultiMap::new();

                draw_obj_vec.iter().for_each(|d_o| {
                    match d_o.obj {
                        DrawObjInner::Segment(s) => {
                            mmap.insert(s.i, s.f);
                        }
                        DrawObjInner::Polygon(ref p) => {
                            for s in p.to_segments() {
                                mmap.insert(s.i, s.f);
                            }
                        }
                        DrawObjInner::Point(_) | DrawObjInner::Char(_, _) => {
                            // do nothing
                        }
                        DrawObjInner::Group(_) => {
                            // do nothing
                            // TODO handle groups
                        }
                    }
                });

                let mut ret: Vec<DrawObj> = vec![];

                while !mmap.is_empty() {
                    let mut adjacent_pts: Vec<Pt> = vec![];

                    let mut p = *mmap.keys().next().unwrap();
                    adjacent_pts.push(p);

                    while let Some(next) = mmap.get_vec_mut(&p).and_then(|v| v.pop()) {
                        adjacent_pts.push(next);
                        p = next;
                    }
                    mmap.remove(&p);

                    if adjacent_pts.len() >= 2 {
                        ret.push(
                            DrawObj::from_polygon(Multiline(adjacent_pts).unwrap())
                                .with_color(color),
                        );
                    }
                }

                ret
            })
            .collect();
    }
}
