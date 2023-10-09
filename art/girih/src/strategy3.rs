use plotz_geometry::{obj2::Obj2, style::Style};

use crate::{
    geom::{all_girih_tiles_in_random_order, PlacedTile, Tile},
    layout::{Layout, Settings},
};

pub fn run() -> Vec<(Obj2, Style)> {
    let girih = all_girih_tiles_in_random_order()[0];
    let tile = Tile::new(girih);
    let pg2 = tile.to_naive_pg2();
    let init_tile = PlacedTile { pg2, tile };

    let settings = Settings {
        num_iterations: 30,
        is_deterministic: false,
    };
    let mut layout = Layout::new(settings, init_tile);

    layout.run();

    let mut result = vec![];

    let apts = layout.to_annotated_placed_tiles();
    for (_girih, pg2) in apts.outlines {
        //
        result.push((Obj2::Pg2(pg2), Style::default()));
    }

    //

    result
}
