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
        point::Pt,
        traits::Mutable,
    },
    std::collections::HashMap,
};

/// Many draw objs.
#[derive(Debug, Clone)]
pub struct Canvas {
    /// the objs.
    pub dos: HashMap<Option<Bucket>, Vec<DrawObj>>,

    /// the frame, maybe.
    pub frame: Option<DrawObj>,
}

impl Canvas {
    /// ctor from objs
    pub fn from_objs<O: IntoIterator<Item = DrawObj>>(objs: O) -> Canvas {
        Canvas {
            dos: HashMap::from([(None, objs.into_iter().collect())]),
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

    /// returns true on success
    pub fn scale_to_fit_frame(&mut self) -> Result<(), Error> {
        {
            let frame_bbox = self.frame.clone().ok_or(anyhow!("no frame"))?.bbox()?;
            let inner_bbox =
                streaming_bbox(self.dos.iter().map(|(_bucket, layers)| layers).flatten())?;

            let w_scale = frame_bbox.width() / inner_bbox.width();
            let s_scale = frame_bbox.height() / inner_bbox.height();
            let scale = std::cmp::min(FloatOrd(w_scale), FloatOrd(s_scale)).0;

            self.dos.iter_mut().for_each(|(_bucket, layers)| {
                for layer in layers.iter_mut() {
                    *layer *= scale;
                }
            });
        }

        {
            let frame_bbox = self.frame.clone().ok_or(anyhow!("no frame"))?.bbox()?;
            let inner_bbox = streaming_bbox(self.dos.values().flatten())?;

            let translate_diff = frame_bbox.bbox_center() - inner_bbox.bbox_center();

            self.dos.iter_mut().for_each(|(_bucket, layers)| {
                layers.iter_mut().for_each(|draw_obj| {
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
        {
            let name = format!("{}_all.svg", prefix);
            let mut all: Vec<DrawObj> = vec![];
            if let Some(frame) = self.frame.clone() {
                all.push(frame);
            }
            for layers in self.dos.values() {
                all.extend(layers.clone());
            }
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
            for (i, (_bucket, layers)) in self.dos.iter().enumerate() {
                let _num = write_layer_to_svg(size, format!("{}_{}.svg", prefix, i), layers)
                    .expect("failed to write");
            }
        }

        Ok(())
    }
}
