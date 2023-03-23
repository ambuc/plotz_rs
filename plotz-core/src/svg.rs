//! SVG plotting utilities.
//!
use crate::{
    char::Char,
    draw_obj::{DrawObj, DrawObjInner},
};
use plotz_color::BLACK;

use plotz_geometry::{plottable::PlotContext, point::Pt, polygon::PolygonKind};
use std::fmt::Debug;
use thiserror::Error;

/// The size of a canvas.
#[derive(Debug, Copy, Clone)]
pub struct Size {
    /// width
    pub width: usize,
    /// height
    pub height: usize,
}
impl Size {
    /// the height or width, whichever is larger.
    pub fn max(&self) -> usize {
        std::cmp::max(self.width, self.height)
    }
    /// the height or width, whichever is smaller.
    pub fn min(&self) -> usize {
        std::cmp::min(self.width, self.height)
    }
}

/// A general error which might be encountered while writing an SVG.
#[derive(Debug, Error)]
pub enum SvgWriteError {
    /// cairo error
    #[error("cairo error")]
    CairoError(#[from] cairo::Error),
}

struct ContextHolder {
    context: cairo::Context,
}

impl PlotContext for ContextHolder {
    fn line_to(&mut self, pt: &Pt) {
        self.context.line_to(pt.x.0, pt.y.0);
    }
    fn move_to(&mut self, pt: &Pt) {
        self.context.move_to(pt.x.0, pt.y.0);
    }
    fn select_font_face(&mut self, family: &str) {
        self.context
            .select_font_face(family, cairo::FontSlant::Normal, cairo::FontWeight::Bold);
    }
    fn set_font_size(&mut self, size: f64) {
        self.context.set_font_size(size);
    }
    fn show_text(&mut self, text: &str) {
        self.context.show_text(text).expect("show text failed");
    }
    fn arc(&mut self, xc: f64, yc: f64, radius: f64, angle1: f64, angle2: f64) {
        self.context.arc(xc, yc, radius, angle1, angle2);
    }
}

fn write_doi_to_context(
    doi: &DrawObjInner,
    context: &mut cairo::Context,
) -> Result<(), SvgWriteError> {
    match &doi {
        DrawObjInner::Point(p) => {
            context.line_to(p.x.0, p.y.0);
            context.line_to(p.x.0 + 1.0, p.y.0 + 1.0);
        }
        DrawObjInner::Polygon(polygon) => {
            //
            for p in &polygon.pts {
                context.line_to(p.x.0, p.y.0);
            }
            if polygon.kind == PolygonKind::Closed {
                context.line_to(polygon.pts[0].x.0, polygon.pts[0].y.0);
            }
        }
        DrawObjInner::Segment(segment) => {
            context.line_to(segment.i.x.0, segment.i.y.0);
            context.line_to(segment.f.x.0, segment.f.y.0);
        }
        DrawObjInner::Char(Char { pt, chr }) => {
            context.select_font_face("serif", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
            context.set_font_size(12.0);

            context.move_to(pt.x.0, pt.y.0);
            context.show_text(&chr.to_string()).expect("show text");
        }
        DrawObjInner::Group(dois) => {
            for doi in dois.iter_dois() {
                write_doi_to_context(&doi, context).expect("write");
            }
        }
        DrawObjInner::CurveArc(arc) => {
            context.arc(
                arc.ctr.x.0,
                arc.ctr.y.0,
                arc.radius.0,
                arc.angle_1.0,
                arc.angle_2.0,
            );
        }
    }
    Ok(())
}

fn write_obj_to_context(co: &DrawObj, context: &mut cairo::Context) -> Result<(), SvgWriteError> {
    if co.obj.is_empty() {
        return Ok(());
    }

    write_doi_to_context(&co.obj, context)?;

    context.set_source_rgb(co.color.r, co.color.g, co.color.b);
    context.set_line_width(co.thickness);
    context.stroke()?;
    context.set_source_rgb(BLACK.r, BLACK.g, BLACK.b);
    Ok(())
}

/// Writes a single iterator of polygons to an SVG of some size at some path.
pub fn write_layer_to_svg<'a, P: Debug + AsRef<std::path::Path>>(
    size: Size,
    path: P,
    polygons: impl IntoIterator<Item = &'a DrawObj>,
) -> Result<usize, SvgWriteError> {
    let svg_surface = cairo::SvgSurface::new(size.width as f64, size.height as f64, Some(path))?;
    let mut ctx = cairo::Context::new(&svg_surface)?;
    let mut c = 0_usize;
    for p in polygons {
        write_obj_to_context(p, &mut ctx)?;
        c += 1;
    }
    Ok(c)
}

fn _write_layers_to_svgs<'a, P: Debug + AsRef<std::path::Path>>(
    size: Size,
    paths: impl IntoIterator<Item = P>,
    polygon_layers: impl IntoIterator<Item = impl IntoIterator<Item = &'a DrawObj>>,
) -> Result<(), SvgWriteError> {
    for (path, polygons) in paths.into_iter().zip(polygon_layers.into_iter()) {
        write_layer_to_svg(size, path, polygons)?;
    }
    Ok(())
}

#[cfg(test)]
mod test_super {
    use super::*;
    use crate::draw_obj::DrawObj;
    use plotz_geometry::{point::Pt, polygon::Polygon};
    use tempdir::TempDir;

    #[test]
    fn test_write_empty_layer_to_svg() {
        let tmp_dir = TempDir::new("example").unwrap();
        let path = tmp_dir.path().join("out.svg");

        write_layer_to_svg(
            Size {
                width: 1024,
                height: 1024,
            },
            path.to_str().unwrap(),
            vec![],
        )
        .unwrap();

        let actual = std::fs::read_to_string(path).unwrap();
        assert!(actual.contains("width=\"1024pt\""));
        assert!(actual.contains("height=\"1024pt\""));
        assert!(actual.contains("<g id=\""));
        assert!(!actual.contains("<path"));
    }

    #[test]
    fn test_write_simple_layer_to_svg() {
        let tmp_dir = TempDir::new("example").unwrap();
        let path = tmp_dir.path().join("out.svg");

        write_layer_to_svg(
            Size {
                width: 1024,
                height: 1024,
            },
            path.to_str().unwrap(),
            vec![&DrawObj {
                color: &BLACK,
                obj: DrawObjInner::Polygon(Polygon([Pt(0, 0), Pt(0, 1), Pt(1, 0)]).unwrap()),
                thickness: 1.0,
            }],
        )
        .unwrap();

        let actual = std::fs::read_to_string(path).unwrap();
        assert!(actual.contains("width=\"1024pt\""));
        assert!(actual.contains("height=\"1024pt\""));
        assert!(actual.contains("<g id=\""));
        assert!(actual.contains("<path style=\"fill:none;stroke-width:1;stroke-linecap:butt;stroke-linejoin:miter;stroke:rgb(0%,0%,0%);stroke-opacity:1;stroke-miterlimit:10;\" d=\"M 0 0 L 0 1 L 1 0 L 0 0 \"/>"));
    }

    #[test]
    fn test_write_complex_layer_to_svg() {
        let tmp_dir = TempDir::new("example").unwrap();
        let path = tmp_dir.path().join("out.svg");

        write_layer_to_svg(
            Size {
                width: 1024,
                height: 1024,
            },
            path.to_str().unwrap(),
            vec![
                &DrawObj {
                    color: &BLACK,
                    obj: DrawObjInner::Polygon(Polygon([Pt(0, 0), Pt(0, 1), Pt(1, 0)]).unwrap()),
                    thickness: 1.0,
                },
                &DrawObj {
                    color: &BLACK,
                    obj: DrawObjInner::Polygon(Polygon([Pt(5, 5), Pt(5, 6), Pt(6, 5)]).unwrap()),
                    thickness: 1.0,
                },
            ],
        )
        .unwrap();

        let actual = std::fs::read_to_string(path).unwrap();
        assert!(actual.contains("width=\"1024pt\""));
        assert!(actual.contains("height=\"1024pt\""));
        assert!(actual.contains("<g id=\""));
        assert!(actual.contains("<path style=\"fill:none;stroke-width:1;stroke-linecap:butt;stroke-linejoin:miter;stroke:rgb(0%,0%,0%);stroke-opacity:1;stroke-miterlimit:10;\" d=\"M 0 0 L 0 1 L 1 0 L 0 0 \"/>"));
        assert!(actual.contains("<path style=\"fill:none;stroke-width:1;stroke-linecap:butt;stroke-linejoin:miter;stroke:rgb(0%,0%,0%);stroke-opacity:1;stroke-miterlimit:10;\" d=\"M 5 5 L 5 6 L 6 5 L 5 5 \"/>"));
    }
}
