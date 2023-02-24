use {
    argh::FromArgs,
    plotz_color::{take_random_colors, ColorRGB},
    plotz_core::{
        draw_obj::{DrawObj, DrawObjs},
        frame::make_frame,
        svg::Size,
    },
    plotz_geometry::{
        bounded::Bounded,
        point::Pt,
        polygon::Polygon,
        shading_02::{shade_polygon, ShadeConfig},
    },
    rand::Rng,
};

static DIM: f64 = 600.0;

#[derive(FromArgs)]
#[argh(description = "...")]
struct Args {
    #[argh(option, description = "output path")]
    output_path_prefix: String,
}

enum Style {
    Shade(ShadeConfig, &'static ColorRGB, bool),
    Nested(Vec<f64>, &'static ColorRGB),
    None,
}

impl Style {
    fn rand(palette: &Vec<&'static ColorRGB>) -> Style {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0, 6) {
            1 | 2 | 3 => Style::Shade(
                ShadeConfig {
                    gap: 4.0,
                    slope: (rng.gen_range(0.0_f64, 360.0_f64)).tan(),
                    thickness: 1.0,
                },
                rng.choose(palette).expect("color"),
                rand::random(),
            ),
            4 | 5 => Style::Nested(
                vec![rng.gen_range(0.3, 0.8)],
                *rng.choose(palette).expect("color"),
            ),
            _ => Style::None,
        }
    }
}

fn main() {
    let args: Args = argh::from_env();

    let mut rng = rand::thread_rng();
    let palette: Vec<&ColorRGB> = take_random_colors(20);

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

    let draw_objs = DrawObjs::from_objs(polygons.iter().flat_map(|p| {
        match Style::rand(&palette) {
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
        }
    }))
    .with_frame(make_frame((DIM, DIM), Pt(50.0, 50.0)));

    let () = draw_objs
        .write_to_svg(
            Size {
                width: 750,
                height: 750,
            },
            &args.output_path_prefix,
        )
        .expect("write");
}
