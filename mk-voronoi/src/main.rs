use {
    argh::FromArgs,
    plotz_color::{ColorRGB, *},
    plotz_core::{canvas::Canvas, frame::make_frame, svg::Size},
    plotz_geometry::{
        object2d::Object2d,
        point::Pt,
        polygon::Polygon,
        shading::{shade_polygon, ShadeConfig},
    },
    rand::{prelude::SliceRandom, Rng},
    std::f64::consts::*,
};

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
    color: &'static ColorRGB,
}

impl Shade {
    fn rand(palette: &Vec<&'static ColorRGB>) -> Shade {
        let mut rng = rand::thread_rng();

        Shade {
            config: ShadeConfig::builder()
                .gap(3.0)
                .switchback(true)
                .slope((rng.gen_range(0.0_f64..360.0_f64)).tan())
                .build(),
            color: palette.choose(&mut rng).expect("color"),
        }
    }
}

fn main() {
    let args: Args = argh::from_env();

    let mut rng = rand::thread_rng();

    let palette: Vec<&ColorRGB> = vec![
        &RED,
        &YELLOW,
        &BLUE,
        &GREEN,
        &ORANGE,
        &ORANGERED,
        &YELLOWGREEN,
        &BLUEVIOLET,
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
        .expect("build vornoi");

    let polygons: Vec<Polygon> = vornoi
        .iter_cells()
        .map(|cell| {
            Polygon(cell.iter_vertices().map(|vertex| Pt(vertex.x, vertex.y))) * DIM
                + Pt(20.0, 20.0)
        })
        .collect();

    let mut dos = vec![];

    dos.extend(polygons.iter().flat_map(|p| {
        (0..=1).flat_map(|_| {
            let shade = Shade::rand(&palette);
            shade_polygon(&shade.config, p)
                .expect("failed to shade")
                .iter()
                .map(|sg| {
                    Object2d::new(*sg)
                        .with_color(shade.color)
                        .with_thickness(1.0)
                })
                .collect::<Vec<_>>()
        })
    }));

    // TODO(ambuc): split by group color before printing
    // TODO(ambuc): split by group color before printing
    // TODO(ambuc): split by group color before printing
    // TODO(ambuc): split by group color before printing
    // TODO(ambuc): split by group color before printing
    // TODO(ambuc): split by group color before printing

    let canvas = Canvas::from_objs(dos.into_iter(), /*autobucket=*/ true)
        .with_frame(make_frame((DIM, DIM), Pt(20.0, 20.0)));

    let () = canvas
        .write_to_svg(
            Size {
                width: 800,
                height: 1000,
            },
            &args.output_path_prefix,
        )
        .expect("write");
}
