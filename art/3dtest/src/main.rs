use argh::FromArgs;
use itertools::iproduct;
use plotz_color::*;
use plotz_core::{canvas::Canvas, frame::*};
use plotz_geometry::{style::Style, *};
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

#[derive(Debug)]
struct CubesConfig {
    width: f64,
    n: usize,
}

fn cubes(cc: CubesConfig) -> Vec<(Obj3, Style)> {
    let mut objects = vec![];

    for ((i, j, k), color) in zip(
        iproduct!(0..cc.n, 0..cc.n, 0..cc.n),
        (vec![&RED, &YELLOW, &GREEN, &BLUE, &PLUM, &ORANGE])
            .iter()
            .cycle(),
    ) {
        let style = Style {
            color,
            thickness: 2.0,
            ..Default::default()
        };
        objects.extend(
            Cube(
                Pt3(i as f64, j as f64, k as f64),
                (cc.width, cc.width, cc.width),
            )
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
            // .debug(_scenedebug)
            .objects(cubes(CubesConfig { n: 3, width: 0.8 }))
            .build()
            .project_with(Projection::default(), Occlusion::True)
            .expect("ok")
            .into_iter(),
        /*autobucket=*/ false,
    )
    .with_frame(make_frame_with_margin(
        (800.0, 600.0),
        /*margin=*/ 25.0,
    ))
    .scale_to_fit_frame_or_die()
    .write_to_svg_or_die((600, 800), &args.output_path_prefix);
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::*;

    #[ignore]
    #[test]
    fn test_cubes_generation_no_crash_reproduce() {
        let _ = Scene::builder()
            .objects(cubes(CubesConfig {
                n: 4,
                width: 19.0 / 20.0,
            }))
            .build()
            .project_with(Projection::default(), Occlusion::True)
            .into_iter()
            .collect::<Vec<_>>();
    }

    #[ignore]
    #[test_matrix(1..=6, 1..=15)]
    fn test_cubes_generation_no_crash(n: usize, w: usize) {
        let width: f64 = (w as f64) / 15.0;
        assert!(0.0 <= width && width <= 1.0);
        let _ = Scene::builder()
            .objects(cubes(CubesConfig { n, width }))
            .build()
            .project_with(Projection::default(), Occlusion::True)
            .into_iter()
            .collect::<Vec<_>>();
    }
}
