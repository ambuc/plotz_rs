//! Many objects.
#![allow(missing_docs)]

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
    shapes::point::Point,
    style::Style,
    *,
};
use rayon::iter::*;
use std::collections::HashMap;
use tracing::*;
use typed_builder::TypedBuilder;

type CanvasMap = HashMap<Option<Bucket>, Vec<(Obj, Style)>>;

pub fn to_canvas_map<O: IntoIterator<Item = (Obj, Style)>>(objs: O, autobucket: bool) -> CanvasMap {
    let mut cm = CanvasMap::new();

    if autobucket {
        for (b, objs) in &objs.into_iter().group_by(|(_obj, style)| style.color) {
            cm.entry(Some(Bucket::Color(b))).or_default().extend(objs);
        }
    } else {
        cm.extend([(None, objs.into_iter().collect())])
    }

    cm
}

/// Many objects.
#[derive(Debug, Clone, Default, TypedBuilder)]
pub struct Canvas {
    #[builder(default)]
    pub dos_by_bucket: CanvasMap,

    #[builder(default, setter(strip_option))]
    pub frame: Option<(Obj, Style)>,
}

impl Canvas {
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
    pub fn mutate_all(&mut self, f: impl Fn(&mut Point)) {
        self.objs_iter_mut().for_each(|o| o.iter_mut().for_each(&f))
    }

    /// returns true on success
    pub fn scale_to_fit_frame(mut self) -> Result<Self> {
        {
            let frame_bounds = self.frame.as_ref().unwrap().0.bounds()?;
            let inner_bounds = streaming_bbox(
                self.dos_by_bucket
                    .iter()
                    .flat_map(|(_bucket, dos)| dos)
                    .map(|(obj, _style)| obj),
            )?;

            let buffer = 0.9;
            let w_scale = frame_bounds.x_span() / inner_bounds.x_span();
            let s_scale = frame_bounds.y_span() / inner_bounds.y_span();
            let scale = std::cmp::min(FloatOrd(w_scale), FloatOrd(s_scale)).0 * buffer;

            self.dos_by_bucket.iter_mut().for_each(|(_bucket, dos)| {
                for (obj, _style) in dos.iter_mut() {
                    *obj *= scale;
                }
            });
        }

        {
            let frame_bounds = self.frame.as_ref().unwrap().0.bounds()?;
            let inner_bounds = streaming_bbox(
                self.dos_by_bucket
                    .values()
                    .flatten()
                    .map(|(obj, _style)| obj),
            )?;

            let translate_diff = frame_bounds.center() - inner_bounds.center();

            self.dos_by_bucket.iter_mut().for_each(|(_bucket, dos)| {
                dos.iter_mut().for_each(|(obj, _style)| {
                    obj.iter_mut().for_each(|pt: &mut Point| {
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
    fn bounds(&self) -> Result<Bounds> {
        streaming_bbox(self.objs_iter())
    }
}
