//! SVG plotting utilities.
//!
use anyhow::Result;
use plotz_color::BLACK;
use plotz_geometry::{obj::Obj, shapes::txt::Txt, style::Style, *};
use std::fmt::Debug;

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

impl From<(i64, i64)> for Size {
    fn from((width, height): (i64, i64)) -> Self {
        Size {
            width: width.try_into().unwrap(),
            height: height.try_into().unwrap(),
        }
    }
}

fn write_doi_to_context(doi: &Obj, context: &mut cairo::Context) -> Result<()> {
    match &doi {
        Obj::Pt(p) => {
            context.line_to(p.x, p.y);
            context.line_to(p.x + 1.0, p.y + 1.0);
        }
        Obj::Pg(polygon) => {
            //
            for p in &polygon.pts {
                context.line_to(p.x, p.y);
            }
            context.line_to(polygon.pts[0].x, polygon.pts[0].y);
        }
        Obj::Ml(ml) => {
            for p in &ml.pts {
                context.line_to(p.x, p.y);
            }
        }
        Obj::Sg(segment) => {
            context.line_to(segment.i.x, segment.i.y);
            context.line_to(segment.f.x, segment.f.y);
        }
        Obj::Txt(Txt {
            pt,
            inner: txt,
            font_size,
        }) => {
            context.select_font_face("serif", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
            context.set_font_size(*font_size);

            context.move_to(pt.x, pt.y);
            context.show_text(txt)?;
        }
        Obj::Group(group) => {
            for s in group.iter_objects() {
                write_obj_to_context(s, context)?;
            }
        }
        Obj::CurveArc(arc) => {
            context.arc(arc.ctr.x, arc.ctr.y, arc.radius, arc.angle_i, arc.angle_f);
        }
    }
    Ok(())
}

fn write_obj_to_context((obj, style): &(Obj, Style), context: &mut cairo::Context) -> Result<()> {
    if obj.is_empty() {
        return Ok(());
    }

    write_doi_to_context(obj, context)?;

    context.set_source_rgb(style.color.r, style.color.g, style.color.b);
    context.set_line_width(style.thickness);
    context.stroke()?;
    context.set_source_rgb(BLACK.r, BLACK.g, BLACK.b);
    Ok(())
}

/// Writes a single iterator of polygons to an SVG of some size at some path.
pub fn write_layer_to_svg<'a, P: Debug + AsRef<std::path::Path>>(
    size: Size,
    path: P,
    polygons: impl IntoIterator<Item = &'a (Obj, Style)>,
) -> Result<usize> {
    let svg_surface = cairo::SvgSurface::new(size.width as f64, size.height as f64, Some(path))?;
    let mut ctx = cairo::Context::new(&svg_surface)?;
    let mut c = 0_usize;
    for so in polygons {
        write_obj_to_context(so, &mut ctx)?;
        c += 1;
    }
    Ok(c)
}

fn _write_layers_to_svgs<'a, P: Debug + AsRef<std::path::Path>>(
    size: Size,
    paths: impl IntoIterator<Item = P>,
    polygon_layers: impl IntoIterator<Item = impl IntoIterator<Item = &'a (Obj, Style)>>,
) -> Result<()> {
    for (path, polygons) in paths.into_iter().zip(polygon_layers.into_iter()) {
        write_layer_to_svg(size, path, polygons)?;
    }
    Ok(())
}

#[cfg(test)]
mod test_super {
    use super::*;
    use plotz_geometry::{shapes::pg::Pg, style::Style};
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
            vec![&(Obj::Pg(Pg([(0, 0), (0, 1), (1, 0)])), Style::default())],
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
                &(Obj::Pg(Pg([(0, 0), (0, 1), (1, 0)])), Style::default()),
                &(Obj::Pg(Pg([(5, 5), (5, 6), (6, 5)])), Style::default()),
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
