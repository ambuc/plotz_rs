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
