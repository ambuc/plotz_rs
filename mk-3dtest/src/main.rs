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
        p3,
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
        let origin_3d = p3!(0, 0, 0);

        let mut objects: Vec<Object3d> = vec![];

        if false {
            objects.extend(
                vec![
                    (p3!(1, 0, 0), &RED),
                    (p3!(0, 1, 0), &BLUE),
                    (p3!(0, 0, 1), &GREEN),
                ]
                .iter()
                .map(|(diff, color)| {
                    Object3d::new(Segment3d(origin_3d, origin_3d + *diff))
                        .with_style(Style3d::new(color, 2.0))
                }),
            );
        }

        if false {
            let e = 0.7;
            for ((i, j), color) in zip(
                iproduct!(0..2, 0..1),
                (vec![&RED, &YELLOW, &GREEN, &BLUE, &PLUM, &ORANGE])
                    .iter()
                    .cycle(),
            ) {
                let zh = 1.0;
                objects.extend(
                    Cube(p3!(i, j, 0), (e, e, zh))
                        .items
                        .into_iter()
                        .map(|face| Object3d::new(face).with_style(Style3d::new(color, 3.0))),
                );
            }
        }

        if true {
            // objects.extend(
            //     Cube(p3!(0, 0, 0), (0.7, 0.7, 1.0))
            //         .items
            //         .into_iter()
            //         .map(|face| Object3d::new(face).with_style(Style3d::new(&RED, 3.0))),
            // );

            for pg3d in [
                //Polygon3d([Pt3d(0.00,0.00,0.00), Pt3d(0.70,0.00,0.00), Pt3d(0.70,0.70,0.00), Pt3d(0.00,0.70,0.00), Pt3d(0.00,0.00,0.00)]),
                //Polygon3d([Pt3d(0.00,0.00,0.00), Pt3d(0.70,0.00,0.00), Pt3d(0.70,0.00,1.00), Pt3d(0.00,0.00,1.00), Pt3d(0.00,0.00,0.00)]),
                //Polygon3d([Pt3d(0.00,0.00,0.00), Pt3d(0.00,0.70,0.00), Pt3d(0.00,0.70,1.00), Pt3d(0.00,0.00,1.00), Pt3d(0.00,0.00,0.00)]),
                Polygon3d([Pt3d(0.70,0.00,0.00), Pt3d(0.70,0.70,0.00), Pt3d(0.70,0.70,1.00), Pt3d(0.70,0.00,1.00), Pt3d(0.70,0.00,0.00)]),
                //Polygon3d([Pt3d(0.00,0.70,0.00), Pt3d(0.00,0.70,1.00), Pt3d(0.70,0.70,1.00), Pt3d(0.70,0.70,0.00), Pt3d(0.00,0.70,0.00)]),
                //Polygon3d([Pt3d(0.00,0.00,1.00), Pt3d(0.70,0.00,1.00), Pt3d(0.70,0.70,1.00), Pt3d(0.00,0.70,1.00), Pt3d(0.00,0.00,1.00)]),
            ] {

                objects.push(Object3d::new(pg3d).with_style(Style3d::new(&RED, 3.0)));
            }


            // objects.extend(
            //     Cube(p3!(1, 0, 0), (0.7, 0.7, 1.0))
            //         .items
            //         .into_iter()
            //         .map(|face| Object3d::new(face).with_style(Style3d::new(&YELLOW, 3.0))),
            // );
            for pg3d in [
                // Polygon3d([ Pt3d(1.00, 0.00, 0.00), Pt3d(1.70, 0.00, 0.00), Pt3d(1.70, 0.70, 0.00), Pt3d(1.00, 0.70, 0.00), Pt3d(1.00, 0.00, 0.00), ]),
                // Polygon3d([ Pt3d(1.00, 0.00, 0.00), Pt3d(1.70, 0.00, 0.00), Pt3d(1.70, 0.00, 1.00), Pt3d(1.00, 0.00, 1.00), Pt3d(1.00, 0.00, 0.00), ]),
                // Polygon3d([ Pt3d(1.00, 0.00, 0.00), Pt3d(1.00, 0.70, 0.00), Pt3d(1.00, 0.70, 1.00), Pt3d(1.00, 0.00, 1.00), Pt3d(1.00, 0.00, 0.00), ]),
                // Polygon3d([ Pt3d(1.70, 0.00, 0.00), Pt3d(1.70, 0.70, 0.00), Pt3d(1.70, 0.70, 1.00), Pt3d(1.70, 0.00, 1.00), Pt3d(1.70, 0.00, 0.00), ]),
                 Polygon3d([ Pt3d(1.00, 0.70, 0.00), Pt3d(1.00, 0.70, 1.00), Pt3d(1.70, 0.70, 1.00), Pt3d(1.70, 0.70, 0.00), Pt3d(1.00, 0.70, 0.00), ]),
                 Polygon3d([ Pt3d(1.00, 0.00, 1.00), Pt3d(1.70, 0.00, 1.00), Pt3d(1.70, 0.70, 1.00), Pt3d(1.00, 0.70, 1.00), Pt3d(1.00, 0.00, 1.00), ]),
            ] {
                objects.push(Object3d::new(pg3d).with_style(Style3d::new(&YELLOW, 3.0)));
            }
        }

        let scene = Scene::builder()
            .debug(
                DebugSettings::builder()
                    .draw_wireframes(Style3d::new(&GRAY, 0.5))
                    .annotate(AnnotationSettings::builder().font_size(12.0).build())
                    .build(),
            )
            .objects(objects)
            .build();
        scene.project_with(Projection::Oblique(Oblique::standard()), Occlusion::True)
    };

    let mut canvas = Canvas::from_objs(dos.into_iter(), /*autobucket=*/ false).with_frame(frame);
    canvas.scale_to_fit_frame().unwrap();

    let () = canvas
        .write_to_svg((800, 1000), &args.output_path_prefix)
        .expect("write");
}
