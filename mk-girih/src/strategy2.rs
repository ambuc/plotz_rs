use crate::geom::{all_girih_tiles, Girih, PlacedTile, Tile};
use plotz_geometry::{shapes::sg2::Sg2, styled_obj2::StyledObj2};
use rand::seq::SliceRandom;

pub struct Settings {
    //
}

pub fn run(settings: &Settings) -> Vec<StyledObj2> {
    let placed_tile: PlacedTile = Tile::new(all_girih_tiles()[0]).place();

    let mut v = vec![];
    v.extend(placed_tile.to_styledobjs());
    v
}
