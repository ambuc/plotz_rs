use crate::{
    geom::{all_girih_tiles_in_random_order, PlacedTile, Tile},
    layout::{Layout, Settings},
};
use anyhow::Result;
use plotz_geometry::{obj::Obj, style::Style};

pub fn run() -> Result<Vec<(Obj, Style)>> {
    let girih = all_girih_tiles_in_random_order()[0];
    let tile = Tile::new(girih);
    let pg = tile.to_naive_pg();
    let init_tile = PlacedTile { pg, tile };

    let settings = Settings {
        num_iterations: 30,
        is_deterministic: false,
    };
    let mut layout = Layout::new(settings, init_tile);

    layout.run()?;

    let mut result = vec![];

    let apts = layout.to_annotated_placed_tiles()?;
    for (_girih, pg) in apts.outlines {
        //
        result.push((Obj::Pg(pg), Style::default()));
    }

    Ok(result)
}
