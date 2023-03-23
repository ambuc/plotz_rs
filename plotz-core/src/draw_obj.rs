//! An annotated object with color and thickness.

use crate::{
    char::Char,
    group::Group,
    svg::{write_layer_to_svg, Size},
};
use anyhow::Error;
use itertools::Itertools;
use multimap::MultiMap;
use plotz_color::{ColorRGB, BLACK};
use plotz_geometry::{
    bounded::Bounded,
    curve::CurveArc,
    point::Pt,
    polygon::{Multiline, Polygon},
    segment::Segment,
    traits::{Mutable, YieldPoints, YieldPointsMut},
};

/// Either a polygon or a segment.
#[derive(Debug, PartialEq, Clone)]
pub enum DrawObjInner {
    /// A point.
    Point(Pt),
    /// A polygon.
    Polygon(Polygon),
    /// A segment.
    Segment(Segment),
    /// An arc.
    CurveArc(CurveArc),
    /// A character to be printed in SVG, at a point.
    Char(Char),
    /// A group of other drawobjects.
    Group(Group),
}

impl DrawObjInner {
    /// Returns true if the object is empty (i.e. zero points)
    pub fn is_empty(&self) -> bool {
        match self {
            DrawObjInner::Polygon(p) => p.pts.is_empty(),
            DrawObjInner::Group(dois) => dois.iter_dois().all(|doi| doi.is_empty()),
            DrawObjInner::Point(_)
            | DrawObjInner::Segment(_)
            | DrawObjInner::Char(_)
            | DrawObjInner::CurveArc(_) => false,
        }
    }

    pub fn inner_impl_bounded(&self) -> &dyn Bounded {
        match self {
            DrawObjInner::Char(ch) => ch,
            DrawObjInner::CurveArc(arc) => arc,
            DrawObjInner::Group(dos) => dos,
            DrawObjInner::Point(p) => p,
            DrawObjInner::Polygon(pg) => pg,
            DrawObjInner::Segment(s) => s,
        }
    }
    pub fn inner_impl_yield_points(&self) -> Option<&dyn YieldPoints> {
        match self {
            DrawObjInner::Point(p) => Some(p),
            DrawObjInner::Char(ch) => Some(ch),
            DrawObjInner::CurveArc(_) => None,
            DrawObjInner::Group(g) => Some(g),
            DrawObjInner::Polygon(pg) => Some(pg),
            DrawObjInner::Segment(sg) => Some(sg),
        }
    }
    pub fn inner_impl_yield_points_mut(&mut self) -> Option<&mut dyn YieldPointsMut> {
        match self {
            DrawObjInner::Point(p) => Some(p),
            DrawObjInner::Char(ch) => Some(ch),
            DrawObjInner::CurveArc(_) => None,
            DrawObjInner::Group(g) => Some(g),
            DrawObjInner::Polygon(pg) => Some(pg),
            DrawObjInner::Segment(sg) => Some(sg),
        }
    }
}

impl YieldPoints for DrawObjInner {
    fn yield_pts(&self) -> Option<Box<dyn Iterator<Item = &Pt> + '_>> {
        self.inner_impl_yield_points().and_then(|yp| yp.yield_pts())
    }
}

impl YieldPointsMut for DrawObjInner {
    fn yield_pts_mut(&mut self) -> Option<Box<dyn Iterator<Item = &mut Pt> + '_>> {
        self.inner_impl_yield_points_mut()
            .and_then(|ypm| ypm.yield_pts_mut())
    }
}

impl Mutable for DrawObjInner {}

impl Bounded for DrawObjInner {
    fn right_bound(&self) -> f64 {
        self.inner_impl_bounded().right_bound()
    }

    fn left_bound(&self) -> f64 {
        self.inner_impl_bounded().left_bound()
    }

    fn top_bound(&self) -> f64 {
        self.inner_impl_bounded().top_bound()
    }

    fn bottom_bound(&self) -> f64 {
        self.inner_impl_bounded().bottom_bound()
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
    pub fn from_char(pt: Pt, chr: char) -> DrawObj {
        Self::from_obj(DrawObjInner::Char(Char { pt, chr }))
    }

    /// from a group.
    pub fn from_group(dos: impl IntoIterator<Item = DrawObjInner>) -> DrawObj {
        Self::from_obj(DrawObjInner::Group(Group::new(dos)))
    }

    pub fn from_curve_arc(curve_arc: CurveArc) -> DrawObj {
        Self::from_obj(DrawObjInner::CurveArc(curve_arc))
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
}

impl YieldPoints for DrawObj {
    fn yield_pts(&self) -> Option<Box<dyn Iterator<Item = &Pt> + '_>> {
        self.obj
            .inner_impl_yield_points()
            .and_then(|yp| yp.yield_pts())
    }
}

impl YieldPointsMut for DrawObj {
    fn yield_pts_mut(&mut self) -> Option<Box<dyn Iterator<Item = &mut Pt> + '_>> {
        self.obj
            .inner_impl_yield_points_mut()
            .and_then(|ypm| ypm.yield_pts_mut())
    }
}

impl Mutable for DrawObj {}

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
        self.draw_obj_vec.iter_mut().for_each(|d_o| {
            d_o.obj.mutate(&f);
        });
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
                        DrawObjInner::Point(_) | DrawObjInner::Char(_) => {
                            // do nothing
                        }
                        DrawObjInner::Group(_) => {
                            // do nothing
                            // TODO handle groups
                        }
                        DrawObjInner::CurveArc(_arc) => {
                            // do nothing
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
