//! Many objects.

use crate::{
    bar::make_bar,
    bucket::Bucket,
    svg::{write_layer_to_svg, Size},
};
use anyhow::Result;
use float_ord::FloatOrd;
use indicatif::*;
use itertools::Itertools;
use plotz_geometry::{
    bounded::{streaming_bbox, Bounded, Bounds},
    obj::Obj,
    shapes::pt::Pt,
    style::Style,
    *,
};
use rayon::iter::*;
use std::collections::HashMap;
use tracing::trace;

type CanvasMap = HashMap<Option<Bucket>, Vec<(Obj, Style)>>;

/// Many objects.
#[derive(Debug, Clone)]
pub struct Canvas {
    /// the objs.
    pub dos_by_bucket: CanvasMap,

    /// the frame, maybe.
    pub frame: Option<(Obj, Style)>,
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new()
    }
}

impl Canvas {
    /// Create a new Canvas.
    pub fn new() -> Canvas {
        Canvas {
            dos_by_bucket: CanvasMap::new(),
            frame: None,
        }
    }

    /// ctor from objs
    pub fn from_objs<O: IntoIterator<Item = (Obj, Style)>>(objs: O, autobucket: bool) -> Canvas {
        let objs_vec: Vec<(Obj, Style)> = objs.into_iter().collect();
        if autobucket {
            trace!(
                "Creating Canvas(autobucket=true) from {:?} objects",
                objs_vec.len()
            );
            let mut c = Canvas::new();
            for (b, objs) in &objs_vec.into_iter().group_by(|(_obj, style)| style.color) {
                c.dos_by_bucket
                    .entry(Some(Bucket::Color(b)))
                    .or_default()
                    .extend(objs);
            }
            c
        } else {
            trace!(
                "Creating Canvas(autobucket=false) from {:?} objects",
                objs_vec.len()
            );
            Canvas {
                dos_by_bucket: CanvasMap::from([(None, objs_vec)]),
                frame: None,
            }
        }
    }

    /// with a frame
    pub fn with_frame(self, frame: (Obj, Style)) -> Canvas {
        Canvas {
            frame: Some(frame),
            ..self
        }
    }

    /// Returns an iterator of Object2dInner.
    pub fn objs_iter(&self) -> impl Iterator<Item = &impl Bounded> {
        self.dos_by_bucket
            .iter()
            .flat_map(|(_bucket, dos)| dos)
            .map(|(obj, _style)| obj)
    }

    /// Returns an iterator of mutable Object2dInner.
    pub fn objs_iter_mut(&mut self) -> impl Iterator<Item = &mut Obj> {
        self.dos_by_bucket
            .iter_mut()
            .flat_map(|(_bucket, dos)| dos)
            .map(|(obj, _style)| obj)
    }

    /// Mutates every object in the canvas according to some |f|.
    pub fn mutate_all(&mut self, f: impl Fn(&mut Pt)) {
        self.objs_iter_mut().for_each(|o| o.iter_mut().for_each(&f))
    }

    /// Translates every object in the canvas according to some |f|.
    pub fn translate_all(&mut self, f: impl Fn(&mut dyn TranslatableAssign)) {
        self.objs_iter_mut().for_each(|obj| {
            f(obj);
        });
    }

    /// Scales every object in the canvas according to some |f|.
    pub fn scale_all(&mut self, f: impl Fn(&mut dyn ScalableAssign)) {
        self.objs_iter_mut().for_each(|obj| {
            f(obj);
        });
    }

    /// returns true on success
    pub fn scale_to_fit_frame(mut self) -> Result<Self> {
        {
            let frame_bounds = self.frame.as_ref().unwrap().0.bounds();
            let inner_bounds = streaming_bbox(
                self.dos_by_bucket
                    .iter()
                    .flat_map(|(_bucket, dos)| dos)
                    .map(|(obj, _style)| obj),
            )?;

            let buffer = 0.9;
            let w_scale = frame_bounds.w() / inner_bounds.w();
            let s_scale = frame_bounds.h() / inner_bounds.h();
            let scale = std::cmp::min(FloatOrd(w_scale), FloatOrd(s_scale)).0 * buffer;

            self.dos_by_bucket.iter_mut().for_each(|(_bucket, dos)| {
                for (obj, _style) in dos.iter_mut() {
                    *obj *= scale;
                }
            });
        }

        {
            let frame_bounds = self.frame.as_ref().unwrap().0.bounds();
            let inner_bounds = streaming_bbox(
                self.dos_by_bucket
                    .values()
                    .flatten()
                    .map(|(obj, _style)| obj),
            )?;

            let translate_diff = frame_bounds.center() - inner_bounds.center();

            self.dos_by_bucket.iter_mut().for_each(|(_bucket, dos)| {
                dos.iter_mut().for_each(|(obj, _style)| {
                    obj.iter_mut().for_each(|pt: &mut Pt| {
                        *pt += translate_diff;
                    });
                });
            });
        }
        Ok(self)
    }

    /// writes out to a set of SVGs at a prefix.
    pub fn write_to_svg(self, size: impl Into<Size>, prefix: &str) -> Result<()> {
        let size = size.into();
        // all
        {
            trace!("Writing to all.");
            let name = format!("{}_all.svg", prefix);
            let mut all: Vec<(Obj, Style)> = vec![];
            if let Some(frame) = self.frame.clone() {
                all.push(frame);
            }
            for dos in self.dos_by_bucket.values() {
                all.extend(dos.clone());
            }
            let pgs: Vec<(Obj, Style)> = all.into_iter().collect::<Vec<_>>();
            write_layer_to_svg(size, name, &pgs)?;
        }

        // frame
        if let Some((inner, style)) = self.frame.clone() {
            trace!("Writing frame.");
            let _ = write_layer_to_svg(
                size,
                format!("{}_{}.svg", prefix, "frame"),
                &[(inner, style)],
            );
        }

        let length = self.dos_by_bucket.len();
        self.dos_by_bucket
            .into_iter()
            .enumerate()
            .collect_vec()
            .par_iter()
            .progress_with(make_bar(length, "writing svg..."))
            .for_each(|(i, (_bucket, os))| {
                let _num = write_layer_to_svg(size, format!("{}_{}.svg", prefix, i), os)
                    .expect("failed to write");
            });

        Ok(())
    }
}

impl Bounded for Canvas {
    fn bounds(&self) -> Bounds {
        streaming_bbox(self.objs_iter()).expect("bbox not found")
    }
}
