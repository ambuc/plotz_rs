use argh::FromArgs;
use itertools::iproduct;
use plotz_color::*;
use plotz_core::{canvas::Canvas, frame::*};
use plotz_geometry::{shading::shade_config::ShadeConfig, style::Style, *};
use plotz_geometry3d::{
    camera::{Occlusion, Projection},
    obj3::Obj3,
    scene::{debug::SceneDebug, Scene},
    shapes::{cube3d::Cube, pt3::Pt3},
};
use std::iter::zip;
use tracing::*;

#[derive(FromArgs)]
#[argh(description = "...")]
struct Args {
    #[argh(option, description = "output path")]
    output_path_prefix: String,
}

fn cubes() -> Vec<(Obj3, Style)> {
    let mut objects = vec![];
    let e = 0.70;
    let n = 7;

    for ((i, j, k), color) in zip(
        iproduct!(0..n, 0..n, 0..n),
        (vec![&RED, &YELLOW, &GREEN, &BLUE, &PLUM, &ORANGE])
            .iter()
            .cycle(),
    ) {
        let shading = ShadeConfig::builder()
            .gap(0.1)
            .slope(0.05)
            .along_face((i + j + k) % 2 == 0)
            .build();
        let style = Style {
            color,
            shading: Some(shading),
            ..Default::default()
        };
        objects.extend(
            Cube(Pt3(i, j, k), (e, e, e))
                .into_iter_objects()
                .map(|(o, _)| (o, style)),
        );
    }
    objects
}

fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .compact()
        .with_max_level(Level::INFO)
        .without_time()
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let _annotation = AnnotationSettings::builder()
        .font_size(12.0)
        .precision(3)
        .build();
    let _scenedebug = SceneDebug::builder()
        .draw_wireframes(Style {
            color: &GRAY,
            thickness: 0.5,
            ..Default::default()
        })
        .annotate(_annotation)
        .build();

    let args: Args = argh::from_env();
    Canvas::from_objs(
        /*objs=*/
        Scene::builder()
            // .debug(scenedebug)
            .objects(cubes())
            .build()
            .project_with(Projection::default(), Occlusion::True)
            .into_iter(),
        /*autobucket=*/ false,
    )
    .with_frame(make_frame_with_margin(
        (1000.0, 800.0),
        /*margin=*/ 25.0,
    ))
    .scale_to_fit_frame_or_die()
    .write_to_svg_or_die((800, 1000), &args.output_path_prefix);
}
