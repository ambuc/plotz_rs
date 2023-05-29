use plotz_geometry::traits::AnnotationSettings;

use {
    argh::FromArgs,
    itertools::{iproduct, zip},
    plotz_color::*,
    plotz_core::{canvas::Canvas, frame::*},
    plotz_geometry::{style::Style, styled_obj2::StyledObj2},
    plotz_geometry3d::{
        camera::{Occlusion, Projection},
        p3,
        scene::{debug::SceneDebug, Scene},
        shapes::{cube3d::Cube, pt3::Pt3},
        styled_obj3::StyledObj3,
    },
    tracing::*,
};

#[derive(FromArgs)]
#[argh(description = "...")]
struct Args {
    #[argh(option, description = "output path")]
    output_path_prefix: String,
}

fn cubes() -> Vec<StyledObj3> {
    let shading = plotz_geometry::shading::shade_config::ShadeConfig::builder()
        .gap(0.1)
        .slope(0.07)
        .build();

    let mut objects = vec![];
    let e = 0.70;
    let n = 7;

    for ((i, j, k), color) in zip(
        iproduct!(0..n, 0..n, 0..n),
        (vec![&RED, &YELLOW, &GREEN, &BLUE, &PLUM, &ORANGE])
            .iter()
            .cycle(),
    ) {
        let style = Style::builder()
            .color(color)
            .thickness(1.0)
            // .shading(shading)
            .build();
        objects.extend(
            Cube(p3!(i, j, k), (e, e, e))
                .iter_objects()
                .cloned()
                .map(|o| StyledObj3::new(o).with_style(style)),
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

    let annotation = AnnotationSettings::builder()
        .font_size(12.0)
        .precision(3)
        .build();
    let scenedebug = SceneDebug::builder()
        .draw_wireframes(Style::new(&GRAY, 0.5))
        .annotate(annotation)
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
