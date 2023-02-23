use plotz_core::frame::make_frame;
use plotz_geometry::bounded::Bounded;

use {
    argh::FromArgs,
    itertools::Itertools,
    plotz_color::{ColorRGB, COLORS},
    plotz_core::{
        draw_obj::DrawObj,
        svg::{write_layer_to_svg, Size},
    },
    plotz_geometry::{
        point::Pt,
        polygon::Polygon,
        shading_02::{shade_polygon, ShadeConfig},
    },
    rand::Rng,
};

static DIM: f64 = 600.0;
static SIZE: Size = Size {
    width: 750,
    height: 750,
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
                *rng.choose(&COLORS[0..20]).expect("color"),
                rand::random(),
            ),
            4 | 5 => Style::Nested(
                vec![rng.gen_range(0.3, 0.8)],
                *rng.choose(&COLORS[0..20]).expect("color"),
            ),
            _ => Style::None,
        }
    }
}

fn main() {
    let args: Args = argh::from_env();

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
            Polygon(cell.iter_vertices().map(|vertex| Pt(vertex.x, vertex.y)))
                .expect("valid polygon")
                * DIM
                + Pt(50.0, 50.0)
        })
        .collect();

    let mut colored_objs: Vec<DrawObj> = polygons
        .iter()
        .flat_map(|p| match Style::rand() {
            Style::Shade(shade_config, color, draw_border) => std::iter::once(if draw_border {
                Some(DrawObj::from_polygon(p.clone()).with_color(color))
            } else {
                None
            })
            .flatten()
            .chain(
                shade_polygon(&shade_config, p)
                    .expect("failed to shade")
                    .iter()
                    .map(|segment| DrawObj::from_segment(*segment).with_color(color)),
            )
            .collect::<Vec<_>>(),
            Style::Nested(fs, color) => fs
                .into_iter()
                .map(|f| {
                    let del = p.bbox_center();
                    DrawObj::from_polygon(((p.clone() - del) * f) + del).with_color(color)
                })
                .collect::<Vec<_>>(),
            Style::None => vec![],
        })
        .collect();

    colored_objs.sort_by(|a, b| a.color.cmp(&b.color));

    let groups = colored_objs.into_iter().group_by(|a| a.color);

    for (i, (_color, group)) in groups.into_iter().enumerate() {
        let colored_objs: Vec<DrawObj> = group.into_iter().collect();
        let num = write_layer_to_svg(
            SIZE,
            format!("{}_{}.svg", args.output_path_prefix, i),
            &colored_objs,
        )
        .expect("failed to write");

        println!("Wrote {:?} lines", num);
    }

    // write frame

    let frame = make_frame((DIM, DIM), Pt(50.0, 50.0));

    let _ = write_layer_to_svg(
        SIZE,
        format!("{}_{}.svg", args.output_path_prefix, "frame"),
        &[frame],
    );
}
