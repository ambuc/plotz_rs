use anyhow::{anyhow, Result};
use argh::FromArgs;
use plotz_color::{ColorRGB, *};
use plotz_core::{
    canvas::{self, Canvas},
    frame::make_frame,
    svg::Size,
};
use plotz_geometry::{
    obj2::Obj2,
    shading::{shade_config::ShadeConfig, shade_polygon},
    shapes::{point::Point, polygon::Polygon},
    style::Style,
};
use rand::{prelude::SliceRandom, Rng};
use std::f64::consts::*;

static DIM: f64 = 750.0;

#[derive(FromArgs)]
#[argh(description = "...")]
struct Args {
    #[argh(option, description = "output path")]
    output_path_prefix: String,
}

#[derive(Debug, Clone)]
struct Shade {
    config: ShadeConfig,
    color: ColorRGB,
}

impl Shade {
    fn rand(palette: &Vec<ColorRGB>) -> Result<Shade> {
        let mut rng = rand::thread_rng();

        Ok(Shade {
            config: ShadeConfig::builder()
                .gap(3.0)
                .switchback(true)
                .slope((rng.gen_range(0.0_f64..360.0_f64)).tan())
                .build(),
            color: *palette.choose(&mut rng).ok_or(anyhow!("?"))?,
        })
    }
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();

    let mut rng = rand::thread_rng();

    let palette: Vec<ColorRGB> = vec![
        RED,
        YELLOW,
        BLUE,
        GREEN,
        ORANGE,
        ORANGERED,
        YELLOWGREEN,
        BLUEVIOLET,
        // &VIOLET,
        // &PINK,
    ];

    let sites: Vec<voronoice::Point> = (1..150)
        .step_by(1)
        .map(|_| {
            let r: f64 = rng.gen_range(0.0..0.5);
            let theta: f64 = rng.gen_range(0.0..TAU);

            voronoice::Point {
                x: r * theta.cos() + 0.5,
                y: r * theta.sin() + 0.5,
            }
        })
        .collect();

    let vornoi = voronoice::VoronoiBuilder::default()
        .set_sites(sites)
        .set_bounding_box(voronoice::BoundingBox::new(
            voronoice::Point { x: 0.5, y: 0.5 },
            1.0,
            1.0,
        ))
        .set_lloyd_relaxation_iterations(10)
        .build()
        .ok_or(anyhow!("build voronoi"))?;

    let polygons: Vec<Polygon> = vornoi
        .iter_cells()
        .map(|cell| {
            Polygon(cell.iter_vertices().map(|vertex| (vertex.x, vertex.y))).unwrap() * DIM
                + (20, 20)
        })
        .collect();

    let mut dos = vec![];

    dos.extend(polygons.iter().flat_map(|p| {
        (0..=1).flat_map(|_| {
            let shade = Shade::rand(&palette).expect("?");
            shade_polygon(&shade.config, p)
                .expect("failed to shade")
                .iter()
                .map(|sg| {
                    (
                        Obj2::Segment(*sg),
                        Style {
                            color: shade.color,
                            ..Default::default()
                        },
                    )
                })
                .collect::<Vec<_>>()
        })
    }));

    Canvas::builder()
        .dos_by_bucket(canvas::to_canvas_map(dos, /*autobucket=*/ true))
        .frame(make_frame((DIM, DIM), Point(20, 20))?)
        .build()
        .write_to_svg(
            Size {
                width: 800,
                height: 1000,
            },
            &args.output_path_prefix,
        )?;
    Ok(())
}
