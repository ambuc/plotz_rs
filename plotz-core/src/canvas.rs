//! Many objects.

use itertools::Itertools;

use {
    crate::{
        bucket::Bucket,
        svg::{write_layer_to_svg, Size},
    },
    anyhow::{anyhow, Error},
    float_ord::FloatOrd,
    plotz_geometry::{
        bounded::{streaming_bbox, Bounded, Bounds},
        object2d::Object2d,
        object2d_inner::Object2dInner,
        point::Pt,
        traits::*,
    },
    std::collections::HashMap,
    tracing::trace,
};

type CanvasMap = HashMap<Option<Bucket>, Vec<Object2d>>;

/// Many objects.
#[derive(Debug, Clone)]
pub struct Canvas {
    /// the objs.
    pub dos_by_bucket: CanvasMap,

    /// the frame, maybe.
    pub frame: Option<Object2d>,
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
    pub fn from_objs<O: IntoIterator<Item = Object2d> + ExactSizeIterator>(
        objs: O,
        autobucket: bool,
    ) -> Canvas {
        if autobucket {
            trace!(
                "Creating Canvas(autobucket=true) from {:?} objects",
                objs.len()
            );
            let mut c = Canvas::new();
            for (b, objs) in &objs.into_iter().group_by(|d_o| d_o.color) {
                c.dos_by_bucket
                    .entry(Some(Bucket::Color(b)))
                    .or_default()
                    .extend(objs);
            }
            c
        } else {
            trace!(
                "Creating Canvas(autobucket=false) from {:?} objects",
                objs.len()
            );
            Canvas {
                dos_by_bucket: CanvasMap::from([(None, objs.into_iter().collect())]),
                frame: None,
            }
        }
    }

    /// with a frame
    pub fn with_frame(self, frame: Object2d) -> Canvas {
        Canvas {
            frame: Some(frame),
            ..self
        }
    }

    /// Returns an iterator of Object2dInner.
    pub fn objs_iter(&self) -> impl Iterator<Item = &Object2dInner> {
        self.dos_by_bucket
            .iter()
            .flat_map(|(_bucket, dos)| dos)
            .map(|d_o| &d_o.inner)
    }

    /// Returns an iterator of mutable Object2dInner.
    pub fn objs_iter_mut(&mut self) -> impl Iterator<Item = &mut Object2dInner> {
        self.dos_by_bucket
            .iter_mut()
            .flat_map(|(_bucket, dos)| dos)
            .map(|d_o| &mut d_o.inner)
    }

    /// Mutates every object in the canvas according to some |f|.
    pub fn mutate_all(&mut self, f: impl Fn(&mut Pt)) {
        self.objs_iter_mut().for_each(|obj| {
            obj.mutate(&f);
        })
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
    pub fn scale_to_fit_frame(&mut self) -> Result<(), Error> {
        {
            let frame_bounds = self.frame.clone().ok_or(anyhow!("no frame"))?.bounds();
            let inner_bounds = streaming_bbox(
                self.dos_by_bucket
                    .iter()
                    .map(|(_bucket, dos)| dos)
                    .flatten(),
            )?;

            let w_scale = frame_bounds.width() / inner_bounds.width();
            let s_scale = frame_bounds.height() / inner_bounds.height();
            let scale = std::cmp::min(FloatOrd(w_scale), FloatOrd(s_scale)).0;

            self.dos_by_bucket.iter_mut().for_each(|(_bucket, dos)| {
                for d_o in dos.iter_mut() {
                    *d_o *= scale;
                }
            });
        }

        {
            let frame_bounds = self.frame.clone().ok_or(anyhow!("no frame"))?.bounds();
            let inner_bounds = streaming_bbox(self.dos_by_bucket.values().flatten())?;

            let translate_diff = frame_bounds.bbox_center() - inner_bounds.bbox_center();

            self.dos_by_bucket.iter_mut().for_each(|(_bucket, dos)| {
                dos.iter_mut().for_each(|obj| {
                    obj.mutate(|pt: &mut Pt| {
                        *pt += translate_diff;
                    });
                });
            });
        }

        Ok(())
    }

    /// writes out to a set of SVGs at a prefix.
    pub fn write_to_svg(self, size: Size, prefix: &str) -> Result<(), Error> {
        // all
        {
            trace!("Writing to all.");
            let name = format!("{}_all.svg", prefix);
            let mut all: Vec<Object2d> = vec![];
            if let Some(frame) = self.frame.clone() {
                all.push(frame);
            }
            for dos in self.dos_by_bucket.values() {
                all.extend(dos.clone());
            }
            write_layer_to_svg(size, name, &all)?;
        }

        // frame
        {
            trace!("Writing frame.");
            if let Some(frame) = self.frame.clone() {
                let _ = write_layer_to_svg(size, format!("{}_{}.svg", prefix, "frame"), &[frame]);
            }
        }

        // dos
        {
            for (i, (_bucket, dos)) in self.dos_by_bucket.iter().enumerate() {
                trace!("Writing layer: {:?}", i);
                let _num = write_layer_to_svg(size, format!("{}_{}.svg", prefix, i), dos)
                    .expect("failed to write");
            }
        }

        Ok(())
    }
}

impl Bounded for Canvas {
    fn bounds(&self) -> Bounds {
        streaming_bbox(self.objs_iter()).expect("bbox not found")
    }
}
