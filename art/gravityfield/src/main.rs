use anyhow::Result;
use argh::FromArgs;
use indicatif::ProgressIterator;
use plotz_color::*;
use plotz_core::{bar::make_bar, canvas::Canvas, frame::make_frame, svg::Size};
use plotz_geometry::{
    crop::Croppable,
    obj::Obj,
    shapes::{
        curve::CurveArc,
        pg::{multiline::Multiline, Pg},
        pt::Pt,
    },
    style::Style,
};
use plotz_physics::{framework, particle::*};
use rand::{thread_rng, Rng};
use std::{f64::consts::TAU, ops::Range};

const CHARGE_MAX: f64 = 2.0;
const CHARGE_RANGE: Range<f64> = -1.0 * CHARGE_MAX..CHARGE_MAX;
const CLUSTER_RANGE: Range<f64> = (-1.0 * CLUSTER_DISTANCE)..CLUSTER_DISTANCE;
const GRID_GRANULARITY: usize = 50;

const NUM_CLUSTERS: usize = 10;
const CLUSTER_DISTANCE: f64 = 100.0;
const NUM_PARTICLES_PER_CLUSTER: usize = 100;

const NUM_STEPS: usize = 100;

#[derive(FromArgs)]
#[argh(description = "...")]
struct Args {
    #[argh(option, description = "output path")]
    output_path_prefix: String,
}

#[derive(Copy, Clone, Debug)]
struct Metadata {
    color: ColorRGB,
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();
    let mut os: Vec<(Obj, Style)> = vec![];
    let margin = 25.0;

    let size = Size {
        width: 800,
        height: 650,
    };
    let frame = make_frame(
        (
            size.height as f64 - 2.0 * margin,
            size.width as f64 - 2.0 * margin,
        ),
        // (800.0 - 2.0 * margin, 1000.0 - 2.0 * margin),
        /*offset=*/
        (margin, margin),
    );

    let mut framework: framework::Framework<Metadata> = framework::Framework {
        config: framework::Config { pow: 1.2 },
        ..Default::default()
    };

    // init configuration
    let particles = {
        let mut ret = vec![];

        for i in (0..=(1000 / GRID_GRANULARITY)).map(|n| n * GRID_GRANULARITY) {
            for j in (0..=(800 / GRID_GRANULARITY)).map(|n| n * GRID_GRANULARITY) {
                // Insert a fixed, invisible high charge particle.
                let charge = thread_rng().gen_range(CHARGE_RANGE.clone());
                ret.push(
                    Particle::builder()
                        .position((i as f64, j as f64))
                        .mobility(Mobility::Fixed)
                        .charge(charge)
                        .visibility(Visibility::Invisible)
                        .metadata(Metadata {
                            color: if charge < 0.0 { RED } else { GREEN },
                        })
                        .build(),
                );
            }
        }

        for _ in 0..NUM_CLUSTERS {
            let cluster_color = random_color();
            let cluster_center = Pt(
                thread_rng().gen_range(0..=900),
                thread_rng().gen_range(0..=700),
            );
            for _ in 0..NUM_PARTICLES_PER_CLUSTER {
                ret.push(
                    Particle::builder()
                        .position(
                            cluster_center
                                + (
                                    thread_rng().gen_range(CLUSTER_RANGE.clone()),
                                    thread_rng().gen_range(CLUSTER_RANGE.clone()),
                                ),
                        )
                        .mobility(Mobility::Mobile)
                        .visibility(Visibility::Visible)
                        .metadata(Metadata {
                            color: cluster_color,
                        })
                        .build(),
                );
            }
        }
        ret
    };

    for p in particles {
        framework.add_particle(p);
    }

    for _ in (0..=NUM_STEPS).progress_with(make_bar(NUM_STEPS, "running simulation...")) {
        framework.advance();
    }

    for (_uuid, p) in framework.into_particles_visible() {
        os.push((
            match p.history.len() {
                // If the object has no history, it is a static particle -- a circle.
                0 => CurveArc(p.position, 0.0..=TAU, 2.0).into(),
                // Otherwise, chart its course.
                _ => Multiline(p.history).unwrap().into(),
            },
            Style {
                color: p.metadata.unwrap().color,
                thickness: 0.1,
                ..Default::default()
            },
        ));
    }

    let frame_pg: Pg = frame.0.clone().try_into().unwrap();
    Canvas::from_objs(
        os.into_iter().flat_map(|(obj, style)| {
            obj.crop_to(&frame_pg)
                .expect("todo")
                .into_iter()
                .map(|o| (o, style))
                .collect::<Vec<_>>()
        }),
        /*autobucket=*/
        true,
    )
    .with_frame(frame)
    .write_to_svg(size, &args.output_path_prefix)?;
    Ok(())
}
