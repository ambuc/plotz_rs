use plotz_geometry::{obj2::Obj2, style::Style};

use argh::FromArgs;
use plotz_color::*;
use plotz_core::{canvas::Canvas, frame::make_frame, svg::Size};
use plotz_geometry::{
    shading::{shade_config::ShadeConfig, shade_polygon},
    shapes::{
        pg2::{multiline::Multiline, Pg2},
        pt2::Pt2,
    },
};
use rand::prelude::SliceRandom;
use std::f64::consts::*;

#[derive(FromArgs)]
#[argh(description = "...")]
struct Args {
    #[argh(option, description = "output path")]
    output_path_prefix: String,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Fill {
    Green,
    Red,
    Blue,
    White,
}

impl Fill {
    pub fn as_usize(&self) -> usize {
        match self {
            Fill::Green => 0,
            Fill::Red => 1,
            Fill::Blue => 2,
            Fill::White => 3,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
struct Tile(usize, Fill, Fill, Fill, Fill); // north, east, south, west
impl Tile {
    pub fn id(&self) -> usize {
        self.0
    }
    pub fn n(&self) -> Fill {
        self.1
    }
    pub fn e(&self) -> Fill {
        self.2
    }
    pub fn s(&self) -> Fill {
        self.3
    }
    pub fn w(&self) -> Fill {
        self.4
    }
}

const TILES: [Tile; 11] = [
    Tile(0, Fill::Red, Fill::Red, Fill::Red, Fill::Green), // 1
    Tile(1, Fill::Blue, Fill::Red, Fill::Blue, Fill::Green), // 2
    Tile(2, Fill::Red, Fill::Green, Fill::Green, Fill::Green), // 3
    Tile(3, Fill::White, Fill::Blue, Fill::Red, Fill::Blue), // 4
    Tile(4, Fill::Blue, Fill::Blue, Fill::White, Fill::Blue), // 5
    Tile(5, Fill::White, Fill::White, Fill::Red, Fill::White), // 6
    Tile(6, Fill::Red, Fill::Green, Fill::Blue, Fill::White), // 7
    Tile(7, Fill::Blue, Fill::White, Fill::Blue, Fill::Red), // 8
    Tile(8, Fill::Blue, Fill::Red, Fill::White, Fill::Red), // 9
    Tile(9, Fill::Green, Fill::Green, Fill::Blue, Fill::Red), // 10
    Tile(10, Fill::Red, Fill::White, Fill::Red, Fill::Green), // 11
];

// returns true if placement was successful
fn try_step(
    rng: &mut rand::rngs::ThreadRng,
    (x, y): (usize, usize),
    grid: &mut Vec<Vec<Option<Tile>>>,
    (i, j): (usize, usize),
) -> bool {
    assert!(grid[i][j].is_none());

    let constraint_west = if i > 0 && (0..x).contains(&(i - 1)) {
        Some(grid[i - 1][j].unwrap().e())
    } else {
        None
    };

    let constraint_north = if j > 0 && (0..y).contains(&(j - 1)) {
        Some(grid[i][j - 1].unwrap().s())
    } else {
        None
    };

    let mut candidates: Vec<Tile> = match (constraint_west, constraint_north) {
        (None, None) => TILES.to_vec(),
        (Some(west), None) => TILES
            .iter()
            .filter(|t| t.w() == west)
            .copied()
            .collect::<Vec<Tile>>(),
        (None, Some(north)) => TILES
            .iter()
            .filter(|t| t.n() == north)
            .copied()
            .collect::<Vec<Tile>>(),
        (Some(west), Some(north)) => TILES
            .iter()
            .filter(|t| t.w() == west && t.n() == north)
            .copied()
            .collect::<Vec<Tile>>(),
    };

    if candidates.is_empty() {
        return false;
    }
    candidates.shuffle(rng);

    let next: Option<(usize, usize)> = match (i + 1, j + 1) {
        (tx, ty) if tx == x && ty == y => None,
        (tx, _) if tx == x => Some((0, j + 1)),
        (_, _) => Some((i + 1, j)),
    };

    match next {
        None => {
            grid[i][j] = Some(candidates[0]);
            true
        }
        Some(next) => candidates.iter().any(|c| {
            grid[i][j] = Some(*c);
            if !try_step(rng, (x, y), grid, next) {
                grid[i][j] = None;
                false
            } else {
                true
            }
        }),
    }
}

fn fill_grid(x: usize, y: usize) -> Vec<Vec<Tile>> {
    let mut rng = rand::thread_rng();
    let mut grid: Vec<Vec<Option<Tile>>> = vec![vec![None; y]; x];

    assert!(try_step(&mut rng, (x, y), &mut grid, (0, 0)));

    grid.iter()
        .map(|row| row.iter().map(|cell| cell.unwrap()).collect())
        .collect()
}

fn draw_tile(cell: Tile, (row_idx, col_idx): (usize, usize)) -> Vec<(Obj2, Style)> {
    [
        (cell.id(), cell.n(), 0.0 * FRAC_PI_2),
        (cell.id(), cell.w(), -1.0 * FRAC_PI_2),
        (cell.id(), cell.s(), -2.0 * FRAC_PI_2),
        (cell.id(), cell.e(), -3.0 * FRAC_PI_2),
    ]
    .into_iter()
    .flat_map(|(cell_id, cell, rot)| {
        let mut ret = vec![];
        ret.push({
            let mut pg: Pg2 = match cell {
                Fill::Blue => Multiline([(0.25, 0.0), (0.5, 0.25), (0.75, 0.0)]).unwrap(),
                Fill::Green => {
                    Multiline([(0.25, 0.0), (0.25, 0.25), (0.75, 0.25), (0.75, 0.0)]).unwrap()
                }
                Fill::Red => Multiline([
                    (0.25, 0.0),
                    (5.0 / 16.0, 3.0 / 16.0),
                    (0.5, 0.25),
                    (11.0 / 16.0, 3.0 / 16.0),
                    (0.75, 0.0),
                ])
                .unwrap(),
                Fill::White => Multiline([
                    (0.25, 0.0),
                    (7.0 / 16.0, 1.0 / 16.0),
                    (0.5, 0.25),
                    (9.0 / 16.0, 1.0 / 16.0),
                    (0.75, 0.0),
                ])
                .unwrap(),
            };

            pg *= 2.0;
            pg.rotate(&Pt2(1, 1), rot);
            pg += (2.0 * row_idx as f64, 2.0 * col_idx as f64);

            (
                Obj2::Pg2(pg),
                Style {
                    color: [&BLUE, &GREEN, &RED, &YELLOW][cell.as_usize()],
                    ..Default::default()
                },
            )
        });
        ret.extend({
            let mut pg: Pg2 = Pg2([(0.1, 0.1), (0.5, 0.5), (0.9, 0.1)]);

            pg *= 2.0;
            pg.rotate(&Pt2(1, 1), rot);
            pg += (2.0 * row_idx as f64, 2.0 * col_idx as f64);

            shade_polygon(&ShadeConfig::builder().gap(0.05).slope(0.0).build(), &pg)
                .unwrap()
                .iter()
                .map(|sg| {
                    (
                        Obj2::Sg2(*sg),
                        Style {
                            color: [
                                &ALICEBLUE,      // 1
                                &BLUEVIOLET,     // 2
                                &CORNFLOWERBLUE, // 3
                                &DODGERBLUE,     // 4
                                &FIREBRICK,      // 5
                                &GOLD,           // 6
                                &HOTPINK,        // 7
                                &KHAKI,          // 8
                                &LAVENDER,       // 9
                                &MAGENTA,        // 10
                                &NAVY,           // 11
                            ][cell_id],
                            ..Default::default()
                        },
                    )
                })
                .collect::<Vec<_>>()
        });
        ret
    })
    .collect()
}

fn main() {
    let args: Args = argh::from_env();

    let image_width: f64 = 600.0;
    let grid_cardinality = 16_usize;
    let margin = 50.0;

    let grid: Vec<Vec<Tile>> = fill_grid(grid_cardinality, grid_cardinality);

    let mut obj_vec: Vec<(Obj2, Style)> = vec![];

    for (row_idx, row) in grid.iter().enumerate() {
        for (col_idx, cell) in row.iter().enumerate() {
            obj_vec.extend(draw_tile(*cell, (row_idx, col_idx)).into_iter());
        }
    }

    let mut objs = Canvas::from_objs(obj_vec.into_iter(), /*autobucket=*/ false)
        .with_frame(make_frame((image_width, image_width), Pt2(margin, margin)));

    let scale = image_width / 2.0 / (grid_cardinality as f64);

    objs.dos_by_bucket.iter_mut().for_each(|(_bucket, layers)| {
        layers.iter_mut().for_each(|(ref mut obj2, _style)| {
            *obj2 *= scale;
            *obj2 += (margin, margin);
        });
    });

    objs.write_to_svg_or_die(
        Size {
            width: (image_width + 2.0 * margin) as usize,
            height: (image_width + 2.0 * margin) as usize,
        },
        &args.output_path_prefix,
    );
}
