use crate::geom::{all_girih_tiles, Constraint, Girih, PlacedTile, Tile};
use plotz_geometry::{
    shapes::{pt2::Pt2, sg2::Sg2},
    styled_obj2::StyledObj2,
};
use rand::seq::SliceRandom;

pub struct Settings {
    //
}

pub fn run(settings: &Settings) -> Vec<StyledObj2> {
    let c = Constraint {
        src_index: 0,
        target: Sg2(Pt2(0, 0), Pt2(1, 0)),
    };
    let placed_tile: PlacedTile = Tile::new(all_girih_tiles()[0]).place(c);

    let mut v = vec![];
    v.extend(placed_tile.to_styledobjs());
    v
}
