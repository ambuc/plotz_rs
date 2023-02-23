use plotz_geometry::bounded::Bounded;

use {
    argh::FromArgs,
    plotz_color::{ColorRGB, COLORS},
    plotz_core::{
        colored_obj::{ColoredObj, Obj},
        svg::{write_layer_to_svg, Size},
    },
    plotz_geometry::{
        point::Pt,
        polygon::Polygon,
        shading_02::{shade_polygon, ShadeConfig},
    },
    rand::Rng,
};

#[derive(FromArgs)]
#[argh(description = "...")]
struct Args {
    #[argh(option, description = "output path")]
    output_path_prefix: String,
}

enum Style {
    Shade(ShadeConfig, ColorRGB, bool),
    Nested(Vec<f64>, ColorRGB),
    None,
}

impl Style {
    fn rand() -> Style {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0, 6) {
            1 | 2 | 3 => Style::Shade(
                ShadeConfig {
                    gap: 4.0,
                    slope: (rng.gen_range(0.0_f64, 360.0_f64)).tan(),
                    thickness: 1.0,
                },
                *rng.choose(&COLORS).expect("color"),
                rand::random(),
            ),
            4 | 5 => Style::Nested(
                vec![rng.gen_range(0.3, 0.8)],
                *rng.choose(&COLORS).expect("color"),
            ),
            _ => Style::None,
        }
    }
}

fn main() {
    let args: Args = argh::from_env();
    //
    println!("args");

    let mut rng = rand::thread_rng();

    let sites: Vec<voronoice::Point> = (1..200)
        .step_by(1)
        .map(|_| {
            let x: f64 = rng.gen();
            let y: f64 = rng.gen();
            voronoice::Point { x, y }
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
            let mut p = Polygon(cell.iter_vertices().map(|vertex| Pt(vertex.x, vertex.y)))
                .expect("valid polygon");
            p *= 400.0;
            p += Pt(50.0, 50.0);
            p
        })
        .collect();

    let colored_objs: Vec<ColoredObj> = polygons
        .iter()
        .flat_map(|p| match Style::rand() {
            Style::Shade(shade_config, color, draw_border) => std::iter::once(if draw_border {
                Some(ColoredObj {
                    obj: Obj::Polygon(p.clone()),
                    color,
                    thickness: 1.0,
                })
            } else {
                None
            })
            .flatten()
            .chain(
                shade_polygon(&shade_config, p)
                    .expect("failed to shade")
                    .iter()
                    .map(|segment| ColoredObj {
                        obj: Obj::Segment(*segment),
                        color,
                        thickness: 1.0,
                    }),
            )
            .collect::<Vec<_>>(),
            Style::Nested(fs, color) => fs
                .iter()
                .map(|f| {
                    let mut p = p.clone();
                    let del = p.bbox_center();
                    p -= del;
                    p *= *f;
                    p += del;
                    ColoredObj {
                        obj: Obj::Polygon(p.clone()),
                        color,
                        thickness: 1.0,
                    }
                })
                .collect::<Vec<_>>(),
            Style::None => vec![],
        })
        .collect();

    let num = write_layer_to_svg(
        Size {
            width: 500,
            height: 500,
        },
        args.output_path,
        &colored_objs,
    )
    .expect("failed to write");

    println!("Wrote {:?} lines", num);
}
