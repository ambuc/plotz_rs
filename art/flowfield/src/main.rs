use anyhow::Result;
use argh::FromArgs;
use indicatif::ParallelProgressIterator;
use plotz_color::*;
use plotz_core::{
    canvas::{self, Canvas},
    frame::make_frame,
    svg::Size,
};
use plotz_geometry::{
    crop::Croppable,
    obj::Obj,
    shapes::{curve::CurveArc, multiline::Ml, point::Point, polygon::Pg, segment::Segment},
    style::Style,
};
use rand::{thread_rng, Rng};
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

fn main() -> Result<()> {
    let uniform_shift = Point(0, 0);

    let args: Args = argh::from_env();

    let mut dos: Vec<(Obj, Style)> = vec![];
    let mgn = 25.0;

    let frame = make_frame(
        (800.0 - 2.0 * mgn, 1000.0 - 2.0 * mgn),
        /*offset=*/ (mgn, mgn),
    )?;

    let mut arrows_store: Vec<Segment> = vec![];

    let arrow_style = Style {
        thickness: 2.0,
        ..Default::default()
    };
    let arrow_start_style = Style {
        thickness: 1.0,
        color: GREEN,
        ..Default::default()
    };
    let arrow_end_style = Style {
        thickness: 1.0,
        color: RED,
        ..Default::default()
    };

    for i in (0..=(900 / GRID_GRANULARITY)).map(|n| n * GRID_GRANULARITY) {
        for j in (0..=(700 / GRID_GRANULARITY)).map(|n| n * GRID_GRANULARITY) {
            let dx = thread_rng().gen_range(ARROW_RANGE.clone());
            let dy = thread_rng().gen_range(ARROW_RANGE.clone());
            let arrow_i = Point(i as f64, j as f64);
            let arrow_f = arrow_i + (dx, dy) + uniform_shift;
            let arrow = Segment(arrow_i, arrow_f);
            arrows_store.push(arrow);
            if PRINT_ARROWS {
                dos.extend([
                    (arrow.into(), arrow_style),
                    (
                        CurveArc(arrow_f, 0.0..=TAU, /*radius=*/ 2.0).into(),
                        arrow_end_style,
                    ),
                    (
                        CurveArc(arrow_f, 0.0..=TAU, /*radius=*/ 2.0).into(),
                        arrow_start_style,
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
                let cluster_center = Point(rx, ry);

                (0..NUM_PTS_PER_CLUSTER)
                    .into_par_iter()
                    .progress()
                    .map(|_| {
                        let rx = thread_rng().gen_range(CLUSTER_RANGE.clone());
                        let ry = thread_rng().gen_range(CLUSTER_RANGE.clone());
                        let pt = cluster_center + (rx, ry);

                        let mut history = vec![pt];
                        let num_steps = thread_rng().gen_range(NUM_STEPS_RANGE.clone());
                        for _ in 0..=num_steps {
                            let last = history.last().unwrap();
                            let del: Point = arrows_store
                                .iter()
                                .map(|arrow| {
                                    let scaling_factor: f64 = last.dist(&arrow.i).sqrt();
                                    (arrow.f - arrow.i) * scaling_factor / MOMENTUM
                                })
                                .fold(Point(0, 0), |acc, x| acc + x);
                            let next: Point = *last + del;
                            history.push(next);
                        }

                        (
                            Ml(history).into(),
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

    let frame_pg: Pg = frame.0.clone().try_into().unwrap();
    Canvas::builder()
        .dos_by_bucket(canvas::to_canvas_map(
            dos.into_iter().flat_map(|(obj, style)| {
                obj.crop_to(&frame_pg)
                    .unwrap()
                    .into_iter()
                    .map(|o| (o, style))
                    .collect::<Vec<_>>()
            }),
            /*autobucket=*/
            true,
        ))
        .frame(frame)
        .build()
        .write_to_svg(
            // yeah, i know
            Size {
                width: 1000,
                height: 800,
            },
            &args.output_path_prefix,
        )?;
    Ok(())
}
