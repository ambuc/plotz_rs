use crate::colored_polygon::ColoredPolygon;
use plotz_color::{ColorRGB, BLACK};
use plotz_geometry::polygon::{Polygon, PolygonKind};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SvgWriteError {
    #[error("cairo error")]
    CairoError(#[from] cairo::Error),
}

fn write_polygon_to_context(
    p: &ColoredPolygon,
    context: &mut cairo::Context,
) -> Result<(), SvgWriteError> {
    if p.polygon.pts.len() == 0 {
        return Ok(());
    }

    for p in &p.polygon.pts {
        context.line_to(p.x.0, p.y.0);
    }
    if p.polygon.kind == PolygonKind::Closed {
        context.line_to(p.polygon.pts[0].x.0, p.polygon.pts[0].y.0);
    }
    context.set_source_rgb(p.color.0, p.color.1, p.color.2);
    context.stroke()?;
    context.set_source_rgb(BLACK.0, BLACK.1, BLACK.2);
    Ok(())
}

fn write_layer_to_svg<P: AsRef<std::path::Path>>(
    (width, height): (f64, f64),
    path: P,
    polygons: impl IntoIterator<Item = ColoredPolygon>,
) -> Result<(), SvgWriteError> {
    let svg_surface = cairo::SvgSurface::new(width, height, Some(path))?;
    let mut ctx = cairo::Context::new(&svg_surface)?;
    for p in polygons {
        write_polygon_to_context(&p, &mut ctx)?;
    }
    Ok(())
}

pub fn write_layers_to_svgs<P: AsRef<std::path::Path>>(
    (width, height): (f64, f64),
    paths: impl IntoIterator<Item = P>,
    polygon_layers: impl IntoIterator<Item = impl IntoIterator<Item = ColoredPolygon>>,
) -> Result<(), SvgWriteError> {
    for (path, polygons) in paths.into_iter().zip(polygon_layers.into_iter()) {
        write_layer_to_svg((width, height), path, polygons)?;
    }
    Ok(())
}

#[cfg(test)]
mod test_super {
    use super::*;
    use crate::colored_polygon::ColoredPolygon;
    use plotz_geometry::{point::Pt, polygon::Polygon};
    use tempdir::TempDir;

    #[test]
    fn test_write_empty_layer_to_svg() {
        let tmp_dir = TempDir::new("example").unwrap();
        let path = tmp_dir.path().join("out.svg");

        write_layer_to_svg((1024.0, 1024.0), path.to_str().unwrap(), vec![]).unwrap();

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
            (1024.0, 1024.0),
            path.to_str().unwrap(),
            vec![ColoredPolygon {
                color: BLACK,
                polygon: Polygon([Pt(0, 0), Pt(0, 1), Pt(1, 0)]).unwrap(),
            }],
        )
        .unwrap();

        let actual = std::fs::read_to_string(path).unwrap();
        assert!(actual.contains("width=\"1024pt\""));
        assert!(actual.contains("height=\"1024pt\""));
        assert!(actual.contains("<g id=\""));
        assert!(actual.contains("<path style=\"fill:none;stroke-width:2;stroke-linecap:butt;stroke-linejoin:miter;stroke:rgb(0%,0%,0%);stroke-opacity:1;stroke-miterlimit:10;\" d=\"M 0 0 L 0 1 L 1 0 L 0 0 \"/>"));
    }

    #[test]
    fn test_write_complex_layer_to_svg() {
        let tmp_dir = TempDir::new("example").unwrap();
        let path = tmp_dir.path().join("out.svg");

        write_layer_to_svg(
            (1024.0, 1024.0),
            path.to_str().unwrap(),
            vec![
                ColoredPolygon {
                    color: BLACK,
                    polygon: Polygon([Pt(0, 0), Pt(0, 1), Pt(1, 0)]).unwrap(),
                },
                ColoredPolygon {
                    color: BLACK,
                    polygon: Polygon([Pt(5, 5), Pt(5, 6), Pt(6, 5)]).unwrap(),
                },
            ],
        )
        .unwrap();

        let actual = std::fs::read_to_string(path).unwrap();
        assert!(actual.contains("width=\"1024pt\""));
        assert!(actual.contains("height=\"1024pt\""));
        assert!(actual.contains("<g id=\""));
        assert!(actual.contains("<path style=\"fill:none;stroke-width:2;stroke-linecap:butt;stroke-linejoin:miter;stroke:rgb(0%,0%,0%);stroke-opacity:1;stroke-miterlimit:10;\" d=\"M 0 0 L 0 1 L 1 0 L 0 0 \"/>"));
        assert!(actual.contains("<path style=\"fill:none;stroke-width:2;stroke-linecap:butt;stroke-linejoin:miter;stroke:rgb(0%,0%,0%);stroke-opacity:1;stroke-miterlimit:10;\" d=\"M 5 5 L 5 6 L 6 5 L 5 5 \"/>"));
    }

    #[test]
    fn test_write_complex_layers_to_svg() {
        let tmp_dir = TempDir::new("example").unwrap();
        let path1 = tmp_dir.path().join("out1.svg");
        let path2 = tmp_dir.path().join("out2.svg");

        write_layers_to_svgs(
            (1024.0, 1024.0),
            [path1.to_str().unwrap(), path2.to_str().unwrap()],
            vec![
                vec![ColoredPolygon {
                    color: BLACK,
                    polygon: Polygon([Pt(0, 0), Pt(0, 1), Pt(1, 0)]).unwrap(),
                }],
                vec![ColoredPolygon {
                    color: BLACK,
                    polygon: Polygon([Pt(5, 5), Pt(5, 6), Pt(6, 5)]).unwrap(),
                }],
            ],
        )
        .unwrap();

        let actual1 = std::fs::read_to_string(path1).unwrap();
        assert!(actual1.contains("width=\"1024pt\""));
        assert!(actual1.contains("height=\"1024pt\""));
        assert!(actual1.contains("<g id=\""));
        assert!(actual1.contains("<path style=\"fill:none;stroke-width:2;stroke-linecap:butt;stroke-linejoin:miter;stroke:rgb(0%,0%,0%);stroke-opacity:1;stroke-miterlimit:10;\" d=\"M 0 0 L 0 1 L 1 0 L 0 0 \"/>"));
        assert!(!actual1.contains("<path style=\"fill:none;stroke-width:2;stroke-linecap:butt;stroke-linejoin:miter;stroke:rgb(0%,0%,0%);stroke-opacity:1;stroke-miterlimit:10;\" d=\"M 5 5 L 5 6 L 6 5 L 5 5 \"/>"));

        let actual2 = std::fs::read_to_string(path2).unwrap();
        assert!(actual2.contains("width=\"1024pt\""));
        assert!(actual2.contains("height=\"1024pt\""));
        assert!(actual2.contains("<g id=\""));
        assert!(!actual2.contains("<path style=\"fill:none;stroke-width:2;stroke-linecap:butt;stroke-linejoin:miter;stroke:rgb(0%,0%,0%);stroke-opacity:1;stroke-miterlimit:10;\" d=\"M 0 0 L 0 1 L 1 0 L 0 0 \"/>"));
        assert!(actual2.contains("<path style=\"fill:none;stroke-width:2;stroke-linecap:butt;stroke-linejoin:miter;stroke:rgb(0%,0%,0%);stroke-opacity:1;stroke-miterlimit:10;\" d=\"M 5 5 L 5 6 L 6 5 L 5 5 \"/>"));
    }
}
