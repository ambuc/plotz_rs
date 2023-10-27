use plotz_geometry::style::Style;

use anyhow::Result;
use argh::FromArgs;
use plotz_core::{
    canvas::{self, Canvas},
    frame::make_frame,
    svg::Size,
};
use plotz_geometry::{
    obj::Obj,
    shapes::{curve::CurveArc, pt::Pt, sg::Sg},
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
    fn to_dos(&self) -> Vec<(Obj, Style)> {
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
    fn to_dois(&self) -> Vec<Obj> {
        let _a = Pt(0, 0);
        let _b = Pt(0.25, 0);
        let c = Pt(0.5, 0);
        let _d = Pt(0.75, 0);
        let _e = Pt(1, 0);
        let _f = Pt(0, 0.25);
        let g = Pt(0.25, 0.25);
        let h = Pt(0.5, 0.25);
        let i = Pt(0.75, 0.25);
        let _j = Pt(1, 0.25);
        let k = Pt(0, 0.5);
        let l = Pt(0.25, 0.5);
        let _m = Pt(0.5, 0.5);
        let n = Pt(0.75, 0.5);
        let o = Pt(1, 0.5);
        let _p = Pt(0, 0.75);
        let q = Pt(0.25, 0.75);
        let r = Pt(0.5, 0.75);
        let s = Pt(0.75, 0.75);
        let _t = Pt(1, 0.75);
        let _u = Pt(0, 1);
        let _v = Pt(0.25, 1);
        let w = Pt(0.5, 1);
        let _x = Pt(0.75, 1);
        let _y = Pt(1, 1);
        match self {
            Tile::Cross => {
                vec![Sg(k, o).into(), Sg(c, w).into()]
            }
            Tile::OverUnder => {
                vec![Sg(k, l).into(), Sg(n, o).into(), Sg(c, w).into()]
            }
            Tile::Swerve => {
                vec![
                    Sg(k, l).into(),
                    CurveArc(g, 0.0..=FRAC_PI_2, 0.25).into(),
                    Sg(c, h).into(),
                    Sg(n, o).into(),
                    CurveArc(s, PI..=(3.0 * FRAC_PI_2), 0.25).into(),
                    Sg(r, w).into(),
                ]
            }
            Tile::Clover => {
                vec![
                    Sg(c, h).into(),
                    Sg(k, l).into(),
                    Sg(n, o).into(),
                    Sg(r, w).into(),
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
                    Sg(k, o).into(),
                    Sg(c, h).into(),
                    Sg(r, w).into(),
                    CurveArc(g, 0.0..=FRAC_PI_2, 0.25).into(),
                    CurveArc(q, (3.0 * FRAC_PI_2)..=TAU, 0.25).into(),
                ]
            }
            Tile::Clover3 => {
                vec![
                    Sg(c, h).into(),
                    Sg(k, l).into(),
                    Sg(n, o).into(),
                    Sg(r, w).into(),
                    CurveArc(g, 0.0..=FRAC_PI_2, 0.25).into(),
                    CurveArc(i, FRAC_PI_2..=PI, 0.25).into(),
                    CurveArc(s, PI..=(3.0 * FRAC_PI_2), 0.25).into(),
                ]
            }
            Tile::Clover2 => {
                vec![
                    Sg(c, w).into(),
                    Sg(k, l).into(),
                    Sg(n, o).into(),
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

    let mut obj_vec: Vec<(Obj, Style)> = vec![];

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

    let c = Canvas {
        dos_by_bucket: canvas::to_canvas_map(obj_vec, /*autobucket=*/ false),
        frame: Some(make_frame((image_width, image_width), Pt(margin, margin))),
    };
    c.scale_to_fit_frame()?.write_to_svg(
        Size {
            width: (image_width + 2.0 * margin) as usize,
            height: (image_width + 2.0 * margin) as usize,
        },
        &args.output_path_prefix,
    )?;
    Ok(())
}
