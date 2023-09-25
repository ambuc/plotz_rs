use crate::geom::{all_girih_tiles, Constraint, Girih, PlacedTile, Tile};
use plotz_geometry::{
    shapes::{pt2::Pt2, sg2::Sg2},
    styled_obj2::StyledObj2,
};
use rand::seq::SliceRandom;

pub struct Settings {
    //
}

struct Layout {
    placed_tiles: Vec<PlacedTile>,
}
impl Layout {
    fn new(pt: PlacedTile) -> Layout {
        Layout {
            placed_tiles: vec![pt],
        }
    }
    fn to_styledobjs(self) -> impl Iterator<Item = StyledObj2> {
        self.placed_tiles
            .into_iter()
            .flat_map(|pt| pt.to_styledobjs())
    }
    fn next() -> () {
        //
    }
}

pub fn run(settings: &Settings) -> impl Iterator<Item = StyledObj2> {
    let mut layout = Layout::new(Tile::new(all_girih_tiles()[0]).place(Constraint {
        src_index: 0,
        target: Sg2(Pt2(0, 0), Pt2(1, 0)),
    }));

    layout.to_styledobjs()
}
