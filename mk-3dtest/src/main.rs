#![allow(unused_imports)]

use itertools::zip;
use plotz_geometry::{object2d_inner::Object2dInner, polygon::Polygon, traits::AnnotationSettings};
use {
    argh::FromArgs,
    itertools::iproduct,
    plotz_color::*,
    plotz_core::{canvas::Canvas, frame::*, svg::Size},
    plotz_geometry::{object2d::Object2d, point::Pt},
    plotz_geometry3d::{
        camera::{Oblique, Occlusion, Projection},
        cube3d::Cube,
        face::Face,
        object3d::Object3d,
        point3d::Pt3d,
        polygon3d::Polygon3d,
        scene::{DebugSettings, Scene},
        segment3d::Segment3d,
        style::Style3d,
    },
    tracing::*,
    tracing_subscriber::FmtSubscriber,
};

#[derive(FromArgs)]
#[argh(description = "...")]
struct Args {
    #[argh(option, description = "output path")]
    output_path_prefix: String,
}

fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .compact()
        .with_max_level(Level::INFO)
        .without_time()
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let args: Args = argh::from_env();

    let margin = 25.0;

    let frame: Object2d = make_frame_with_margin((1000.0, 800.0), margin);

    let dos: Vec<Object2d> = {
        let origin_3d = Pt3d(0.0, 0.0, 0.0);

        let mut objects: Vec<Object3d> = vec![];

        let _axes: Vec<Object3d> = vec![
            (Pt3d(1.0, 0.0, 0.0), &RED),
            (Pt3d(0.0, 1.0, 0.0), &BLUE),
            (Pt3d(0.0, 0.0, 1.0), &GREEN),
        ]
        .iter()
        .map(|(diff, color)| {
            Object3d::new(Segment3d(origin_3d, origin_3d + *diff))
                .with_style(Style3d::builder().color(color).thickness(2.0).build())
        })
        .collect();

        // objects.extend(_axes);

        {
            let e = 0.7;
            // //use rand::Rng;
            // //let mut rng = rand::thread_rng();
            let colors = vec![&RED, &YELLOW, &GREEN, &BLUE, &PLUM, &ORANGE];
            for ((i, j), color) in zip(iproduct!(0..2, 0..2), colors.iter().cycle()) {
                //let zh = rng.gen_range(0.5..=3.0);
                let zh = 1.0;
                objects.extend(
                    Cube(Pt3d(i as f64, j as f64, 0.0), (e, e, zh))
                        .items
                        .into_iter()
                        .map(|face| {
                            Object3d::new(face)
                                .with_style(Style3d::builder().color(color).thickness(3.0).build())
                        }),
                );
            }

            // objects.extend(
            //     Cube(Pt3d(0.0, 0.0, 0.0), (0.7, 0.7, 1.0))
            //         .items
            //         .into_iter()
            //         .map(|face| {
            //             Object3d::new(face)
            //                 .with_style(Style3d::builder().color(&RED).thickness(1.0).build())
            //         }),
            // );
            // objects.extend(
            //     Cube(Pt3d(1.0, 0.0, 0.0), (0.7, 0.7, 1.0))
            //         .items
            //         .into_iter()
            //         .map(|face| {
            //             Object3d::new(face)
            //                 .with_style(Style3d::builder().color(&YELLOW).thickness(1.0).build())
            //         }),
            // );
        }

        let scene = Scene::builder()
            .debug(
                DebugSettings::builder()
                    // .draw_wireframes(Style3d::builder().color(&GRAY).thickness(0.1).build())
                    // .annotate(AnnotationSettings::builder().font_size(12.0).build())
                    .build(),
            )
            .objects(objects)
            .build();
        scene.project_with(Projection::Oblique(Oblique::standard()), Occlusion::True)
    };

    let mut canvas = Canvas::from_objs(dos.into_iter(), /*autobucket=*/ false).with_frame(frame);
    canvas.scale_to_fit_frame().unwrap();

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
