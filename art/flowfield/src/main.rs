use argh::FromArgs;
use indicatif::ParallelProgressIterator;
use plotz_color::*;
use plotz_core::{canvas::Canvas, frame::make_frame, svg::Size};
use plotz_geometry::{
    crop::Croppable,
    obj2::Obj2,
    shapes::{
        curve::CurveArc,
        pg2::{multiline::Multiline, Pg2},
        pt2::Pt2,
        sg2::Sg2,
    },
    style::Style,
};
use rand::thread_rng;
use rand::Rng;
use rayon::prelude::*;
use std::{f64::consts::*, ops::Range};

const ARROW_RANGE: Range<f64> = (-1.0 * MAX_ARROW_SIZE)..MAX_ARROW_SIZE;
const CLUSTER_DISTANCE: f64 = 400.0;
const CLUSTER_RANGE: Range<f64> = (-1.0 * CLUSTER_DISTANCE)..CLUSTER_DISTANCE;
const GRID_GRANULARITY: usize = 20;
const MAX_ARROW_SIZE: f64 = 70.0;
const MOMENTUM: f64 = 1000.0;
const NUM_CLUSTERS: usize = 8;
const NUM_PTS_PER_CLUSTER: usize = 300;
const NUM_STEPS_RANGE: Range<usize> = 100..500;
const PRINT_ARROWS: bool = false;

#[derive(FromArgs)]
#[argh(description = "...")]
struct Args {
    #[argh(option, description = "output path")]
    output_path_prefix: String,
}

fn main() {
    let uniform_shift: Pt2 = Pt2(0.0, 0.0);

    let args: Args = argh::from_env();

    let mut dos: Vec<(Obj2, Style)> = vec![];
    let mgn = 25.0;

    let frame = make_frame(
        (800.0 - 2.0 * mgn, 1000.0 - 2.0 * mgn),
        /*offset=*/ Pt2(mgn, mgn),
    );

    let mut arrows_store: Vec<Sg2> = vec![];

    for i in (0..=(900 / GRID_GRANULARITY)).map(|n| n * GRID_GRANULARITY) {
        for j in (0..=(700 / GRID_GRANULARITY)).map(|n| n * GRID_GRANULARITY) {
            let dx = thread_rng().gen_range(ARROW_RANGE.clone());
            let dy = thread_rng().gen_range(ARROW_RANGE.clone());
            let arrow_i = Pt2(i as f64, j as f64);
            let arrow_f = arrow_i + Pt2(dx, dy) + uniform_shift;
            let arrow = Sg2(arrow_i, arrow_f);
            arrows_store.push(arrow);
            if PRINT_ARROWS {
                dos.extend([
                    (
                        Obj2::Sg2(arrow),
                        Style {
                            thickness: 2.0,
                            ..Default::default()
                        },
                    ),
                    (
                        Obj2::CurveArc(CurveArc(arrow_f, 0.0..=TAU, /*radius=*/ 2.0)),
                        Style {
                            thickness: 1.0,
                            color: &RED,
                            ..Default::default()
                        },
                    ),
                    (
                        Obj2::CurveArc(CurveArc(arrow_f, 0.0..=TAU, /*radius=*/ 2.0)),
                        Style {
                            thickness: 1.0,
                            color: &GREEN,
                            ..Default::default()
                        },
                    ),
                ]);
            }
        }
    }

    dos.extend(
        (0..NUM_CLUSTERS)
            .into_par_iter()
            .progress()
            .flat_map(|_| {
                let cluster_color = random_color();
                let rx = thread_rng().gen_range(0..=900);
                let ry = thread_rng().gen_range(0..=700);
                let cluster_center = Pt2(rx, ry);

                (0..NUM_PTS_PER_CLUSTER)
                    .into_par_iter()
                    .progress()
                    .map(|_| {
                        let rx = thread_rng().gen_range(CLUSTER_RANGE.clone());
                        let ry = thread_rng().gen_range(CLUSTER_RANGE.clone());
                        let pt = cluster_center + Pt2(rx, ry);

                        let mut history = vec![pt];
                        let num_steps = thread_rng().gen_range(NUM_STEPS_RANGE.clone());
                        for _ in 0..=num_steps {
                            let last = history.last().unwrap();
                            let del: Pt2 = arrows_store
                                .iter()
                                .map(|arrow| {
                                    let scaling_factor: f64 = last.dist(&arrow.i).sqrt();
                                    (arrow.f - arrow.i) * scaling_factor / MOMENTUM
                                })
                                .fold(Pt2(0.0, 0.0), |acc, x| acc + x);
                            let next: Pt2 = *last + del;
                            history.push(next);
                        }

                        let sg = Multiline(history).expect("multiline");
                        (
                            Obj2::Pg2(sg),
                            Style {
                                color: cluster_color,
                                ..Default::default()
                            },
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
    );

    let frame_pg2: Pg2 = frame.0.clone().try_into().unwrap();
    let objs = Canvas::from_objs(
        dos.into_iter().flat_map(|(obj2, style)| {
            obj2.crop_to(&frame_pg2)
                .into_iter()
                .map(|o| (o, style))
                .collect::<Vec<_>>()
        }),
        /*autobucket=*/ true,
    )
    .with_frame(frame);

    objs.write_to_svg_or_die(
        // yeah, i know
        Size {
            width: 1000,
            height: 800,
        },
        &args.output_path_prefix,
    );
}
