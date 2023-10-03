use plotz_geometry::styled_obj2::StyledObj2;

use crate::{
    geom::{all_girih_tiles_in_random_order, PlacedTile, Tile},
    layout::{AnnotatedPlacedTiles, Layout, Settings},
};

pub fn run() -> Vec<StyledObj2> {
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
    for (girih, pg2) in apts.outlines {
        //
        result.push(StyledObj2::new(pg2));
    }

    //

    result
}
