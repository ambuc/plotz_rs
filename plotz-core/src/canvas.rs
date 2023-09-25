//! Many objects.

use {
    crate::{
        bucket::Bucket,
        svg::{write_layer_to_svg, Size},
    },
    anyhow::{anyhow, Error},
    float_ord::FloatOrd,
    itertools::Itertools,
    plotz_geometry::{
        bounded::{streaming_bbox, Bounded, Bounds},
        obj2::Obj2,
        shapes::pt2::Pt2,
        styled_obj2::StyledObj2,
        traits::*,
    },
    std::collections::HashMap,
    tracing::trace,
};

type CanvasMap = HashMap<Option<Bucket>, Vec<StyledObj2>>;

/// Many objects.
#[derive(Debug, Clone)]
pub struct Canvas {
    /// the objs.
    pub dos_by_bucket: CanvasMap,

    /// the frame, maybe.
    pub frame: Option<StyledObj2>,
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
    pub fn from_objs<O: IntoIterator<Item = StyledObj2>>(objs: O, autobucket: bool) -> Canvas {
        let objs_vec: Vec<_> = objs.into_iter().collect();
        if autobucket {
            trace!(
                "Creating Canvas(autobucket=true) from {:?} objects",
                objs_vec.len()
            );
            let mut c = Canvas::new();
            for (b, objs) in &objs_vec.into_iter().group_by(|d_o| d_o.style.color) {
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
    pub fn with_frame(self, frame: StyledObj2) -> Canvas {
        Canvas {
            frame: Some(frame),
            ..self
        }
    }

    /// Returns an iterator of Object2dInner.
    pub fn objs_iter(&self) -> impl Iterator<Item = &Obj2> {
        self.dos_by_bucket
            .iter()
            .flat_map(|(_bucket, dos)| dos)
            .map(|d_o| &d_o.inner)
    }

    /// Returns an iterator of mutable Object2dInner.
    pub fn objs_iter_mut(&mut self) -> impl Iterator<Item = &mut Obj2> {
        self.dos_by_bucket
            .iter_mut()
            .flat_map(|(_bucket, dos)| dos)
            .map(|d_o| &mut d_o.inner)
    }

    /// Mutates every object in the canvas according to some |f|.
    pub fn mutate_all(&mut self, f: impl Fn(&mut Pt2)) {
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

    // returns true on success
    fn scale_to_fit_frame(mut self) -> Result<Self, Error> {
        {
            let frame_bounds = self.frame.clone().ok_or(anyhow!("no frame"))?.bounds();
            let inner_bounds = streaming_bbox(
                self.dos_by_bucket
                    .iter()
                    .map(|(_bucket, dos)| dos)
                    .flatten(),
            )?;

            let buffer = 0.9;
            let w_scale = frame_bounds.width() / inner_bounds.width();
            let s_scale = frame_bounds.height() / inner_bounds.height();
            let scale = std::cmp::min(FloatOrd(w_scale), FloatOrd(s_scale)).0 * buffer;

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
                    obj.mutate(|pt: &mut Pt2| {
                        *pt += translate_diff;
                    });
                });
            });
        }
        Ok(self)
    }

    /// Scales the contents to fit the frame, or dies.
    pub fn scale_to_fit_frame_or_die(self) -> Self {
        self.scale_to_fit_frame()
            .expect("failed to scale to fit frame")
    }

    // writes out to a set of SVGs at a prefix.
    fn write_to_svg(self, size: impl Into<Size>, prefix: &str) -> Result<(), Error> {
        let size = size.into();
        // all
        {
            trace!("Writing to all.");
            let name = format!("{}_all.svg", prefix);
            let mut all: Vec<StyledObj2> = vec![];
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

    /// writes out to a set of SVGs at a prefix, or dies.
    pub fn write_to_svg_or_die(self, size: impl Into<Size>, prefix: &str) {
        self.write_to_svg(size, prefix).expect("write")
    }
}

impl Bounded for Canvas {
    fn bounds(&self) -> Bounds {
        streaming_bbox(self.objs_iter()).expect("bbox not found")
    }
}
