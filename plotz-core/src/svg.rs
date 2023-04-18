//! SVG plotting utilities.
//!
use {
    plotz_color::BLACK,
    plotz_geometry::polygon::PolygonKind,
    plotz_geometry::{object2d::Object2d, object2d_inner::Object2dInner, txt::Txt},
    std::fmt::Debug,
    thiserror::Error,
};

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

fn write_doi_to_context(
    doi: &Object2dInner,
    context: &mut cairo::Context,
) -> Result<(), SvgWriteError> {
    match &doi {
        Object2dInner::Point(p) => {
            context.line_to(p.x.0, p.y.0);
            context.line_to(p.x.0 + 1.0, p.y.0 + 1.0);
        }
        Object2dInner::Polygon(polygon) => {
            //
            for p in &polygon.pts {
                context.line_to(p.x.0, p.y.0);
            }
            if polygon.kind == PolygonKind::Closed {
                context.line_to(polygon.pts[0].x.0, polygon.pts[0].y.0);
            }
        }
        Object2dInner::Segment(segment) => {
            context.line_to(segment.i.x.0, segment.i.y.0);
            context.line_to(segment.f.x.0, segment.f.y.0);
        }
        Object2dInner::Char(Txt { pt, inner: txt }) => {
            context.select_font_face("serif", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
            context.set_font_size(5.0);

            context.move_to(pt.x.0, pt.y.0);
            context.show_text(txt).expect("show text");
        }
        Object2dInner::Group(group) => {
            for obj in group.iter_objects() {
                write_obj_to_context(obj, context).expect("write");
            }
        }
        Object2dInner::CurveArc(arc) => {
            context.arc(
                arc.ctr.x.0,
                arc.ctr.y.0,
                arc.radius,
                arc.angle_i,
                arc.angle_f,
            );
        }
    }
    Ok(())
}

fn write_obj_to_context(co: &Object2d, context: &mut cairo::Context) -> Result<(), SvgWriteError> {
    if co.inner.is_empty() {
        return Ok(());
    }

    write_doi_to_context(&co.inner, context)?;

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
    polygons: impl IntoIterator<Item = &'a Object2d>,
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
    polygon_layers: impl IntoIterator<Item = impl IntoIterator<Item = &'a Object2d>>,
) -> Result<(), SvgWriteError> {
    for (path, polygons) in paths.into_iter().zip(polygon_layers.into_iter()) {
        write_layer_to_svg(size, path, polygons)?;
    }
    Ok(())
}

#[cfg(test)]
mod test_super {
    use super::*;
    use plotz_geometry::object2d::Object2d;
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
            vec![&Object2d {
                color: &BLACK,
                inner: Object2dInner::Polygon(Polygon([Pt(0, 0), Pt(0, 1), Pt(1, 0)]).unwrap()),
                thickness: 1.0,
            }],
        )
        .unwrap();

        let actual = std::fs::read_to_string(path).unwrap();
        assert!(actual.contains("width=\"1024pt\""));
        assert!(actual.contains("height=\"1024pt\""));
        assert!(actual.contains("<g id=\""));
        // assert!(actual.contains("<path style=\"fill:none;stroke-width:1;stroke-linecap:butt;stroke-linejoin:miter;stroke:rgb(0%,0%,0%);stroke-opacity:1;stroke-miterlimit:10;\" d=\"M 0 0 L 0 1 L 1 0 L 0 0 \"/>"));
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
                &Object2d {
                    color: &BLACK,
                    inner: Object2dInner::Polygon(Polygon([Pt(0, 0), Pt(0, 1), Pt(1, 0)]).unwrap()),
                    thickness: 1.0,
                },
                &Object2d {
                    color: &BLACK,
                    inner: Object2dInner::Polygon(Polygon([Pt(5, 5), Pt(5, 6), Pt(6, 5)]).unwrap()),
                    thickness: 1.0,
                },
            ],
        )
        .unwrap();

        let actual = std::fs::read_to_string(path).unwrap();
        assert!(actual.contains("width=\"1024pt\""));
        assert!(actual.contains("height=\"1024pt\""));
        assert!(actual.contains("<g id=\""));
        // assert!(actual.contains("<path style=\"fill:none;stroke-width:1;stroke-linecap:butt;stroke-linejoin:miter;stroke:rgb(0%,0%,0%);stroke-opacity:1;stroke-miterlimit:10;\" d=\"M 0 0 L 0 1 L 1 0 L 0 0 \"/>"));
        // assert!(actual.contains("<path style=\"fill:none;stroke-width:1;stroke-linecap:butt;stroke-linejoin:miter;stroke:rgb(0%,0%,0%);stroke-opacity:1;stroke-miterlimit:10;\" d=\"M 5 5 L 5 6 L 6 5 L 5 5 \"/>"));
    }
}
