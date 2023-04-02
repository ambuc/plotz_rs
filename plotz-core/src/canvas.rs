//! Many draw objs.

use {
    crate::{
        bucket::Bucket,
        svg::{write_layer_to_svg, Size},
    },
    anyhow::{anyhow, Error},
    float_ord::FloatOrd,
    plotz_geometry::{
        bounded::{streaming_bbox, Bounded},
        draw_obj::DrawObj,
        draw_obj_inner::DrawObjInner,
        point::Pt,
        polygon::Polygon,
        traits::*,
    },
    std::collections::HashMap,
    tracing::trace,
};

type CanvasMap = HashMap<Option<Bucket>, Vec<DrawObj>>;

/// Many draw objs.
#[derive(Debug, Clone)]
pub struct Canvas {
    /// the objs.
    pub dos_by_bucket: CanvasMap,

    /// the frame, maybe.
    pub frame: Option<DrawObj>,
}

impl Canvas {
    pub fn new() -> Canvas {
        Canvas {
            dos_by_bucket: CanvasMap::new(),
            frame: None,
        }
    }

    /// ctor from objs
    pub fn from_objs<O: IntoIterator<Item = DrawObj>>(objs: O) -> Canvas {
        Canvas {
            dos_by_bucket: CanvasMap::from([(None, objs.into_iter().collect())]),
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

    pub fn objs_iter(&self) -> impl Iterator<Item = &DrawObjInner> {
        self.dos_by_bucket
            .iter()
            .flat_map(|(_bucket, dos)| dos)
            .map(|d_o| &d_o.obj)
    }

    pub fn objs_iter_mut(&mut self) -> impl Iterator<Item = &mut DrawObjInner> {
        self.dos_by_bucket
            .iter_mut()
            .flat_map(|(_bucket, dos)| dos)
            .map(|d_o| &mut d_o.obj)
    }

    pub fn mutate_all(&mut self, f: impl Fn(&mut Pt)) {
        self.objs_iter_mut().for_each(|obj| {
            obj.mutate(&f);
        })
    }

    pub fn translate_all(&mut self, f: impl Fn(&mut dyn TranslatableAssign)) {
        self.objs_iter_mut().for_each(|obj| {
            f(obj);
        });
    }

    pub fn scale_all(&mut self, f: impl Fn(&mut dyn ScalableAssign)) {
        self.objs_iter_mut().for_each(|obj| {
            f(obj);
        });
    }

    pub fn get_bbox(&self) -> Polygon {
        streaming_bbox(self.objs_iter()).expect("bbox not found")
    }

    /// returns true on success
    pub fn scale_to_fit_frame(&mut self) -> Result<(), Error> {
        {
            let frame_bbox = self.frame.clone().ok_or(anyhow!("no frame"))?.bbox()?;
            let inner_bbox = streaming_bbox(
                self.dos_by_bucket
                    .iter()
                    .map(|(_bucket, dos)| dos)
                    .flatten(),
            )?;

            let w_scale = frame_bbox.width() / inner_bbox.width();
            let s_scale = frame_bbox.height() / inner_bbox.height();
            let scale = std::cmp::min(FloatOrd(w_scale), FloatOrd(s_scale)).0;

            self.dos_by_bucket.iter_mut().for_each(|(_bucket, dos)| {
                for d_o in dos.iter_mut() {
                    *d_o *= scale;
                }
            });
        }

        {
            let frame_bbox = self.frame.clone().ok_or(anyhow!("no frame"))?.bbox()?;
            let inner_bbox = streaming_bbox(self.dos_by_bucket.values().flatten())?;

            let translate_diff = frame_bbox.bbox_center() - inner_bbox.bbox_center();

            self.dos_by_bucket.iter_mut().for_each(|(_bucket, dos)| {
                dos.iter_mut().for_each(|draw_obj| {
                    draw_obj.mutate(|pt: &mut Pt| {
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
        trace!("Writing to canvas.");
        {
            let name = format!("{}_all.svg", prefix);
            let mut all: Vec<DrawObj> = vec![];
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
            if let Some(frame) = self.frame.clone() {
                let _ = write_layer_to_svg(size, format!("{}_{}.svg", prefix, "frame"), &[frame]);
            }
        }

        // dos
        {
            for (i, (_bucket, dos)) in self.dos_by_bucket.iter().enumerate() {
                let _num = write_layer_to_svg(size, format!("{}_{}.svg", prefix, i), dos)
                    .expect("failed to write");
            }
        }

        Ok(())
    }
}
