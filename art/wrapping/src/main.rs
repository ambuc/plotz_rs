use plotz_geometry::style::Style;

use anyhow::Result;
use argh::FromArgs;
use plotz_core::{
    canvas::{self, Canvas},
    frame::make_frame,
    svg::Size,
};
use plotz_geometry::{
    obj2::Obj2,
    shapes::{curve::CurveArc, point::Point, segment::Segment},
};
use rand::{distributions::Standard, prelude::Distribution, Rng};
use std::f64::consts::*;

#[derive(FromArgs)]
#[argh(description = "...")]
struct Args {
    #[argh(option, description = "output path")]
    output_path_prefix: String,
}

impl Distribution<Tile> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Tile {
        match rng.gen_range(0..=6) {
            0 => Tile::Cross,
            1 => Tile::OverUnder,
            2 => Tile::Swerve,
            3 => Tile::Clover,
            4 => Tile::CloverIn,
            5 => Tile::Clover3,
            _ => Tile::Clover2,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum Tile {
    Cross,
    OverUnder,
    Swerve,
    Clover,
    CloverIn,
    Clover3,
    Clover2,
}
impl Tile {
    fn to_dos(&self) -> Vec<(Obj2, Style)> {
        self.to_dois()
            .into_iter()
            .map(|obj| {
                (
                    obj,
                    Style {
                        thickness: 2.0,
                        ..Default::default()
                    },
                )
            })
            .collect::<Vec<_>>()
    }

    // scaled to a unit square.
    fn to_dois(&self) -> Vec<Obj2> {
        let _a = Point(0, 0);
        let _b = Point(0.25, 0);
        let c = Point(0.5, 0);
        let _d = Point(0.75, 0);
        let _e = Point(1, 0);
        let _f = Point(0, 0.25);
        let g = Point(0.25, 0.25);
        let h = Point(0.5, 0.25);
        let i = Point(0.75, 0.25);
        let _j = Point(1, 0.25);
        let k = Point(0, 0.5);
        let l = Point(0.25, 0.5);
        let _m = Point(0.5, 0.5);
        let n = Point(0.75, 0.5);
        let o = Point(1, 0.5);
        let _p = Point(0, 0.75);
        let q = Point(0.25, 0.75);
        let r = Point(0.5, 0.75);
        let s = Point(0.75, 0.75);
        let _t = Point(1, 0.75);
        let _u = Point(0, 1);
        let _v = Point(0.25, 1);
        let w = Point(0.5, 1);
        let _x = Point(0.75, 1);
        let _y = Point(1, 1);
        match self {
            Tile::Cross => {
                vec![Segment(k, o).into(), Segment(c, w).into()]
            }
            Tile::OverUnder => {
                vec![
                    Segment(k, l).into(),
                    Segment(n, o).into(),
                    Segment(c, w).into(),
                ]
            }
            Tile::Swerve => {
                vec![
                    Segment(k, l).into(),
                    CurveArc(g, 0.0..=FRAC_PI_2, 0.25).into(),
                    Segment(c, h).into(),
                    Segment(n, o).into(),
                    CurveArc(s, PI..=(3.0 * FRAC_PI_2), 0.25).into(),
                    Segment(r, w).into(),
                ]
            }
            Tile::Clover => {
                vec![
                    Segment(c, h).into(),
                    Segment(k, l).into(),
                    Segment(n, o).into(),
                    Segment(r, w).into(),
                    CurveArc(g, 0.0..=FRAC_PI_2, 0.25).into(),
                    CurveArc(i, FRAC_PI_2..=PI, 0.25).into(),
                    CurveArc(s, PI..=(3.0 * FRAC_PI_2), 0.25).into(),
                    CurveArc(q, (3.0 * FRAC_PI_2)..=TAU, 0.25).into(),
                ]
                //
            }
            Tile::CloverIn => {
                vec![
                    //
                    Segment(k, o).into(),
                    Segment(c, h).into(),
                    Segment(r, w).into(),
                    CurveArc(g, 0.0..=FRAC_PI_2, 0.25).into(),
                    CurveArc(q, (3.0 * FRAC_PI_2)..=TAU, 0.25).into(),
                ]
            }
            Tile::Clover3 => {
                vec![
                    Segment(c, h).into(),
                    Segment(k, l).into(),
                    Segment(n, o).into(),
                    Segment(r, w).into(),
                    CurveArc(g, 0.0..=FRAC_PI_2, 0.25).into(),
                    CurveArc(i, FRAC_PI_2..=PI, 0.25).into(),
                    CurveArc(s, PI..=(3.0 * FRAC_PI_2), 0.25).into(),
                ]
            }
            Tile::Clover2 => {
                vec![
                    Segment(c, w).into(),
                    Segment(k, l).into(),
                    Segment(n, o).into(),
                    CurveArc(g, 0.0..=FRAC_PI_2, 0.25).into(),
                    CurveArc(s, PI..=(3.0 * FRAC_PI_2), 0.25).into(),
                ]
            }
        }
    }
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();

    let image_width: f64 = 500.0;
    let margin = 10.0;

    let mut obj_vec: Vec<(Obj2, Style)> = vec![];

    let width = 10;
    let height = 10;
    for dx in 0..=width {
        for dy in 0..=height {
            let tile: Tile = rand::random();
            obj_vec.extend(
                tile.to_dos()
                    .into_iter()
                    .map(|(obj, style)| (obj + (dx, dy), style)),
            );
        }
    }

    Canvas::builder()
        .dos_by_bucket(canvas::to_canvas_map(obj_vec, /*autobucket=*/ false))
        .frame(make_frame(
            (image_width, image_width),
            Point(margin, margin),
        )?)
        .build()
        .scale_to_fit_frame()?
        .write_to_svg(
            Size {
                width: (image_width + 2.0 * margin) as usize,
                height: (image_width + 2.0 * margin) as usize,
            },
            &args.output_path_prefix,
        )?;
    Ok(())
}
