#![allow(unused)]

use crate::layout::{Layout, Settings};
use crate::{geom::*, layout::AnnotatedPlacedTiles};
use indicatif::ProgressBar;
use itertools::Itertools;
use plotz_color::BLACK;
use plotz_geometry::obj2::Obj2;
use plotz_geometry::style::Style;
use plotz_geometry::{
    shading::{shade_config::ShadeConfig, shade_polygon},
    shapes::{
        pg2::{multiline::Multiline, Pg2},
        pt2::Pt2,
        sg2::Sg2,
    },
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

fn pts_eq_within(a: Pt2, b: Pt2, epsilon: f64) -> bool {
    a.dist(&b) < epsilon
}
fn vals_eq_within(a: f64, b: f64, epsilon: f64) -> bool {
    (a - b).abs() < epsilon
}

fn chase(apts: &AnnotatedPlacedTiles) -> Vec<(Obj2, Style)> {
    // first of all, we're guaranteed that every element in so2s is a strap. nothing else.
    let mut inputs: Vec<Sg2> = apts.straps.iter().map(|(_, sg2)| *sg2).collect();

    let mut outputs: Vec<(Obj2, Style)> = vec![];
    let epsilon = 0.001;

    // collect links in the chain. implicitly going sg.i -> sg.f.
    while let Some(first) = inputs.pop() {
        let mut segments: Vec<Sg2> = vec![first];

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

            let cand_sg: Sg2 = inputs.remove(next_idx); // get next sg

            let next_sg = if pts_eq_within(cand_sg.i, last.f, epsilon) {
                cand_sg
            } else {
                cand_sg.flip()
            };

            segments.push(next_sg); // use next_sg
        }

        let mut pts = segments.iter().map(|sg2| sg2.i).collect::<Vec<_>>();
        pts.push(segments.first().unwrap().i);

        // and then make a multiline, and add it to our final outputs list.
        outputs.push((
            Obj2::Pg2(Multiline(pts).unwrap()),
            Style {
                color: plotz_color::take_random_colors(1).next().unwrap(),
                thickness: 3.0,
                ..Default::default()
            },
        ));
    }

    outputs
}

fn postprocess(display: &Display, apts: AnnotatedPlacedTiles) -> Vec<(Obj2, Style)> {
    let mut v: Vec<(Obj2, Style)> = vec![];

    display.0.iter().for_each(|inst| match inst {
        Instr::StrapsOriginal(thickness) => {
            v.extend(apts.clone().straps.into_iter().map(|(girih, sg2)| {
                (
                    Obj2::Sg2(sg2),
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
            v.extend(apts.clone().outlines.into_iter().map(|(_, pg2)| {
                // scale
                (
                    Obj2::Pg2(pg2),
                    Style {
                        color: &BLACK,
                        thickness: *thickness,
                        ..Default::default()
                    },
                )
            }))
        }
        Instr::TileShaded(shade_config) => {
            v.extend(apts.clone().outlines.iter().flat_map(|(girih, pg2)| {
                shade_polygon(shade_config, pg2)
                    .unwrap()
                    .into_iter()
                    .map(|shade| {
                        (
                            Obj2::Sg2(shade),
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

pub fn run() -> Vec<(Obj2, Style)> {
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
            let pg2 = tile.to_naive_pg2();
            PlacedTile { pg2, tile }
        },
    );

    layout.run();

    postprocess(&d, layout.to_annotated_placed_tiles())
}
