use plotz_color::ColorRGB;
use plotz_geometry::polygon::Polygon;

#[derive(Debug)]
pub struct ColoredPolygon {
    pub polygon: Polygon,
    pub color: ColorRGB,
}
