//! Many draw objs.

use {
    crate::{
        draw_obj::DrawObj,
        draw_obj_inner::DrawObjInner,
        svg::{write_layer_to_svg, Size},
    },
    anyhow::Error,
    itertools::Itertools,
    multimap::MultiMap,
    plotz_color::ColorRGB,
    plotz_geometry::{point::Pt, polygon::Multiline},
};

/// Many draw objs.
#[derive(Debug, Clone)]
pub struct Canvas {
    /// the objs.
    pub dos: Vec<DrawObj>,

    /// the frame, maybe.
    pub frame: Option<DrawObj>,
}

impl Canvas {
    /// ctor from objs
    pub fn from_objs<O: IntoIterator<Item = DrawObj>>(objs: O) -> Canvas {
        Canvas {
            dos: objs.into_iter().collect(),
            frame: None,
        }
    }

    /// with a frame
    pub fn with_frame(self, frame: DrawObj) -> Canvas {
        Canvas {
            frame: Some(frame),
            ..self
        }
    }

    /// Sorts and groups the internal draw objects by color.
    pub fn group_by_color(mut self) -> Vec<(&'static ColorRGB, Vec<DrawObj>)> {
        self.dos.sort_by_key(|d_o| d_o.color);
        self.dos
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
            all.extend(self.dos.clone());
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

    /// join adjacent segments to save on path draw time.
    pub fn join_adjacent_segments(&mut self) {
        self.dos = self
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
                        ret.push(DrawObj::new(Multiline(adjacent_pts).unwrap()).with_color(color));
                    }
                }

                ret
            })
            .collect();
    }
}
