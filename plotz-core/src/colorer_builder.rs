use crate::bucket::{Area, Bucket, Path};
use crate::colorer::DefaultColorer;
use plotz_color::*;

#[derive(Debug)]
pub struct DefaultColorerBuilder {
    statements: Vec<(Bucket, ColorRGB)>,
}
impl DefaultColorerBuilder {
    pub fn new() -> DefaultColorerBuilder {
        DefaultColorerBuilder { statements: vec![] }
    }
    pub fn build(&self) -> DefaultColorer {
        DefaultColorer::new(self.statements.clone())
    }
    fn add(&mut self, bucket: Bucket, color: ColorRGB) {
        self.statements.push((bucket, color));
    }
    fn add_path(&mut self, path: Path, color: ColorRGB) {
        self.add(Bucket::Path(path), color);
    }
    fn add_area(&mut self, area: Area, color: ColorRGB) {
        self.add(Bucket::Area(area), color);
    }
    fn add_path_highway(&mut self) -> &mut Self {
        self.add_path(Path::Highway1, BLACK);
        self.add_path(Path::Highway2, DARKGRAY);
        self.add_path(Path::Highway3, GRAY);
        self.add_path(Path::Highway4, DARKGREY);
        self
    }
    fn add_path_non_highway(&mut self) -> &mut Self {
        self.add_path(Path::Cycleway, DARKGREY);
        self.add_path(Path::Pedestrian, DARKGREY);
        self.add_path(Path::Rail, DARKGREY);
        self.add_path(Path::Boundary, DARKGREY);
        self
    }
    fn add_areas(&mut self) -> &mut Self {
        self.add_area(Area::Beach, TAN);
        self.add_area(Area::Building, DARKGREY);
        self.add_area(Area::Business, DARKGREY);
        self.add_area(Area::Fun, DARKGREEN);
        self.add_area(Area::NaturalRock, DARKGRAY);
        self.add_area(Area::Park, GREEN);
        self.add_area(Area::Rail, ORANGE);
        self.add_area(Area::Tree, BROWN);
        self.add_area(Area::Water, LIGHTBLUE);
        self
    }
    pub fn default() -> DefaultColorer {
        DefaultColorerBuilder::new()
            .add_path_highway()
            .add_path_non_highway()
            .add_areas()
            .build()
    }
}
