#![allow(unused)]

use plotz_color::{BLUE, GREEN, RED, YELLOW};

use {
    argh::FromArgs,
    plotz_color::{take_random_colors, ColorRGB},
    plotz_core::{
        draw_obj::{DrawObj, DrawObjInner, DrawObjs},
        frame::make_frame,
        svg::Size,
    },
    plotz_geometry::{
        bounded::Bounded,
        point::Pt,
        polygon::Polygon,
        shading_02::{shade_polygon, ShadeConfig},
    },
    rand::{prelude::SliceRandom, Rng},
};

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
    pub fn north(&self) -> Fill {
        self.1
    }
    pub fn east(&self) -> Fill {
        self.2
    }
    pub fn south(&self) -> Fill {
        self.3
    }
    pub fn west(&self) -> Fill {
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

    let constraint_west = if (i > 0 && (0..x).contains(&(i - 1))) {
        Some(grid[i - 1][j].unwrap().east())
    } else {
        None
    };

    let constraint_north = if (j > 0 && (0..y).contains(&(j - 1))) {
        Some(grid[i][j - 1].unwrap().south())
    } else {
        None
    };

    let mut candidates: Vec<Tile> = match (constraint_west, constraint_north) {
        (None, None) => TILES.to_vec(),
        (Some(west), None) => TILES
            .iter()
            .filter(|t| t.west() == west)
            .copied()
            .collect::<Vec<Tile>>(),
        (None, Some(north)) => TILES
            .iter()
            .filter(|t| t.north() == north)
            .copied()
            .collect::<Vec<Tile>>(),
        (Some(west), Some(north)) => TILES
            .iter()
            .filter(|t| t.west() == west && t.north() == north)
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

fn draw_tile(
    cell: Tile,
    (row_idx, col_idx): (usize, usize),
    palette: &[&'static ColorRGB],
) -> Vec<DrawObj> {
    let diff = 0.05;
    let zero = 0.0 + diff;
    let one = 1;
    let two = 2.0 - diff;
    let p00 = Pt(zero, zero);
    let p02 = Pt(zero, two);
    let p11 = Pt(one, one);
    let p20 = Pt(two, zero);
    let p22 = Pt(two, two);
    [
        (Polygon([p00, p20, p11]).unwrap(), cell.north().as_usize()),
        (Polygon([p20, p11, p22]).unwrap(), cell.east().as_usize()),
        (Polygon([p22, p11, p02]).unwrap(), cell.south().as_usize()),
        (Polygon([p02, p11, p00]).unwrap(), cell.west().as_usize()),
    ]
    .iter()
    .flat_map(|(polygon, fill_index)| {
        let p = polygon + Pt(2.0 * row_idx as f64, 2.0 * col_idx as f64);

        vec![DrawObj::from_polygon(p).with_color(palette[*fill_index])]
    })
    .collect()
}

fn main() {
    let args: Args = argh::from_env();

    let image_width: f64 = 600.0;
    let grid_cardinality = 12.0;
    let margin = 50.0;

    let grid: Vec<Vec<Tile>> = fill_grid(grid_cardinality as usize, grid_cardinality as usize);

    let palette: Vec<&'static ColorRGB> = vec![&RED, &YELLOW, &GREEN, &BLUE];

    let mut draw_obj_vec = vec![];

    for (row_idx, row) in grid.iter().enumerate() {
        for (col_idx, cell) in row.iter().enumerate() {
            draw_obj_vec.extend(draw_tile(*cell, (row_idx, col_idx), &palette));
        }
    }

    let mut draw_objs = DrawObjs::from_objs(draw_obj_vec)
        .with_frame(make_frame((image_width, image_width), Pt(margin, margin)));

    let scale = image_width / 2.0 / grid_cardinality;

    draw_objs
        .draw_objs
        .iter_mut()
        .for_each(|d_o| match &mut d_o.obj {
            DrawObjInner::Polygon(p) => {
                *p *= scale;
                *p += Pt(margin, margin);
            }
            DrawObjInner::Segment(s) => {
                *s *= scale;
                *s += Pt(margin, margin);
            }
        });

    let () = draw_objs
        .write_to_svg(
            Size {
                width: (image_width + margin) as usize,
                height: (image_width + margin) as usize,
            },
            &args.output_path_prefix,
        )
        .expect("write");
}
