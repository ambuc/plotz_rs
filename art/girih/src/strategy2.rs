#![allow(unused)]

use crate::{
    geom::*,
    layout::{AnnotatedPlacedTiles, Layout, Settings},
};
use anyhow::Result;
use indicatif::ProgressBar;
use itertools::Itertools;
use plotz_color::BLACK;
use plotz_geometry::{
    obj::Obj,
    shading::{shade_config::ShadeConfig, shade_polygon},
    shapes::{multiline::Ml, point::Pt, polygon::Pg, segment::Sg},
    style::Style,
};
use rand::seq::SliceRandom;
use std::f64::consts::TAU;

#[derive(Debug)]
enum Instr {
    StrapsOriginal(/*thickness */ f64),
    StrapsChasing,
    TilesOutline { thickness: f64 },
    TileShaded(ShadeConfig),
}

#[derive(Debug)]
struct Display(Vec<Instr>);

fn pts_eq_within(a: Pt, b: Pt, epsilon: f64) -> bool {
    a.dist(&b) < epsilon
}
fn vals_eq_within(a: f64, b: f64, epsilon: f64) -> bool {
    (a - b).abs() < epsilon
}

fn chase(apts: &AnnotatedPlacedTiles) -> Vec<(Obj, Style)> {
    // first of all, we're guaranteed that every element in so2s is a strap. nothing else.
    let mut inputs: Vec<Sg> = apts.straps.iter().map(|(_, sg)| *sg).collect();

    let mut outputs: Vec<(Obj, Style)> = vec![];
    let epsilon = 0.001;

    // collect links in the chain. implicitly going sg.i -> sg.f.
    while let Some(first) = inputs.pop() {
        let mut segments: Vec<Sg> = vec![first];

        'l: loop {
            let last = segments.last().unwrap();

            let positions = inputs
                .iter()
                .positions(|cand_sg| {
                    pts_eq_within(cand_sg.i, last.f, epsilon)
                        || pts_eq_within(cand_sg.f, last.f, epsilon)
                })
                .collect::<Vec<usize>>();

            let next_idx: usize = match positions[..] {
                [] => break 'l,
                [next_idx] => next_idx,
                _ => *positions
                    .iter()
                    .find(|cand_idx| {
                        vals_eq_within(inputs[**cand_idx].ray_angle(), last.ray_angle(), epsilon)
                    })
                    .unwrap(),
            };

            let cand_sg: Sg = inputs.remove(next_idx); // get next sg

            let next_sg = if pts_eq_within(cand_sg.i, last.f, epsilon) {
                cand_sg
            } else {
                cand_sg.flip()
            };

            segments.push(next_sg); // use next_sg
        }

        let mut pts = segments.iter().map(|sg| sg.i).collect::<Vec<_>>();
        pts.push(segments.first().unwrap().i);

        // and then make a multiline, and add it to our final outputs list.
        outputs.push((
            Ml(pts).into(),
            Style {
                color: plotz_color::take_random_colors(1).next().unwrap(),
                thickness: 3.0,
                ..Default::default()
            },
        ));
    }

    outputs
}

fn postprocess(display: &Display, apts: AnnotatedPlacedTiles) -> Vec<(Obj, Style)> {
    let mut v: Vec<(Obj, Style)> = vec![];

    display.0.iter().for_each(|inst| match inst {
        Instr::StrapsOriginal(thickness) => {
            v.extend(apts.clone().straps.into_iter().map(|(girih, sg)| {
                (
                    Obj::Sg(sg),
                    Style {
                        color: girih.color(),
                        thickness: *thickness,
                        ..Default::default()
                    },
                )
            }))
        }
        Instr::StrapsChasing => v.extend(chase(&apts)),
        Instr::TilesOutline { thickness } => {
            v.extend(apts.clone().outlines.into_iter().map(|(_, pg)| {
                // scale
                (
                    Obj::Pg(pg),
                    Style {
                        color: BLACK,
                        thickness: *thickness,
                        ..Default::default()
                    },
                )
            }))
        }
        Instr::TileShaded(shade_config) => {
            v.extend(apts.clone().outlines.iter().flat_map(|(girih, pg)| {
                shade_polygon(shade_config, pg)
                    .unwrap()
                    .into_iter()
                    .map(|shade| {
                        (
                            Obj::Sg(shade),
                            Style {
                                color: girih.color(),
                                ..Default::default()
                            },
                        )
                    })
            }))
        }
    });

    v
}

pub fn run() -> Result<Vec<(Obj, Style)>> {
    let d = Display(vec![
        // Instr::StrapsOriginal(2.0),
        Instr::TilesOutline { thickness: 1.0 },
        Instr::StrapsChasing,
        Instr::TileShaded(
            ShadeConfig::builder()
                .gap(0.05)
                .slope(1.0)
                .switchback(false)
                .build(),
        ),
    ]);
    let mut layout = Layout::new(
        Settings {
            num_iterations: 30,
            is_deterministic: false,
        },
        {
            let girih = all_girih_tiles_in_random_order()[0];
            let tile = Tile::new(girih);
            let pg2 = tile.to_naive_pg();
            PlacedTile { pg: pg2, tile }
        },
    );

    layout.run()?;

    Ok(postprocess(&d, layout.to_annotated_placed_tiles()?))
}
