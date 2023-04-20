#![allow(unused_imports)]

use itertools::zip;
use {
    argh::FromArgs,
    itertools::iproduct,
    plotz_color::*,
    plotz_core::{canvas::Canvas, frame::make_frame, svg::Size},
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

    let mgn = 25.0;

    let frame: Object2d = make_frame(
        (1000.0 - 2.0 * mgn, 800.0 - 2.0 * mgn),
        /*offset=*/ Pt(mgn, mgn),
    );

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
            // use rand::Rng;
            // let mut rng = rand::thread_rng();
            let n = 2;
            let colors = vec![
                &RED,
                &YELLOW,
                &GREEN,
                &BLUE,
                &PLUM,
                &ORANGE,
                &DARKRED,
                &BROWN,
                &DARKGREEN,
                &DARKBLUE,
                &DARKORCHID,
                &DARKORANGE,
            ];
            for ((i, j), color) in zip(iproduct!(0..n, 0..n), colors.iter().cycle()) {
                // let zh = rng.gen_range(0.5..=3.0);
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
        }

        // {
        //     // objects.extend(
        //     //     Cube(Pt3d(0.0, 0.0, 0.0), (1.0, 1.0, 1.0))
        //     //         .items
        //     //         .into_iter()
        //     //         .map(|face| {
        //     //             Object3d::new(face).with_style(Style3d::builder().thickness(2.0).build())
        //     //         }),
        //     // );
        //     let style = Style3d::builder().thickness(2.0).build();
        //     // objects.push(
        //     //     Object3d::new(Face::from(Polygon3d([
        //     //         Pt3d(1.0, 0.0, 0.0) + Pt3d(0.0, 0.0, 0.0),
        //     //         Pt3d(1.0, 0.0, 0.0) + Pt3d(0.0, 1.0, 0.0),
        //     //         Pt3d(1.0, 0.0, 0.0) + Pt3d(0.0, 1.0, 1.0),
        //     //         Pt3d(1.0, 0.0, 0.0) + Pt3d(0.0, 0.0, 1.0),
        //     //         Pt3d(1.0, 0.0, 0.0) + Pt3d(0.0, 0.0, 0.0),
        //     //     ])))
        //     //     .with_style(style),
        //     // );
        //     objects.extend(
        //         Cube(Pt3d(0.0, 1.3, 0.0), (1.0, 1.0, 1.0))
        //             .items
        //             .into_iter()
        //             .map(|face| Object3d::new(face).with_style(style)),
        //     );
        //     objects.extend(
        //         Cube(Pt3d(0.0, 0.0, 0.0), (1.0, 1.0, 1.0))
        //             .items
        //             .into_iter()
        //             .map(|face| Object3d::new(face).with_style(style)),
        //     );
        // }

        // {
        //     let o = Pt3d(0.0, 0.0, 0.0);

        //     let f1 = Face::from(Polygon3d([
        //         o,
        //         o + Pt3d(0.0, 1.0, 0.0),
        //         o + Pt3d(0.0, 1.0, 1.0),
        //         o + Pt3d(0.0, 0.0, 1.0),
        //         o,
        //     ]));
        //     objects.push(Object3d::new(f1.clone() + Pt3d(0.0, 0.0, 0.0)).with_color(&RED));
        //     objects.push(Object3d::new(f1.clone() + Pt3d(0.3, 0.0, 0.0)).with_color(&ORANGE));
        //     objects.push(Object3d::new(f1.clone() + Pt3d(0.6, 0.0, 0.0)).with_color(&YELLOW));
        //     objects.push(Object3d::new(f1.clone() + Pt3d(0.9, 0.0, 0.0)).with_color(&GREEN));
        //     objects.push(Object3d::new(f1.clone() + Pt3d(1.2, 0.0, 0.0)).with_color(&BLUE));
        // }

        // Triangle.
        // objects.push(
        //     Object3d::new(Polygon3d([
        //         origin_3d + Pt3d(0.5, 0.0, 0.0),
        //         origin_3d + Pt3d(0.0, 0.5, 0.0),
        //         origin_3d + Pt3d(0.0, 0.0, 0.5),
        //         origin_3d + Pt3d(0.5, 0.0, 0.0),
        //     ]))
        //     .with_style(Style3d::builder().color(&POWDERBLUE).thickness(2.0).build()),
        // );

        let scene = Scene::builder()
            .debug(
                DebugSettings::builder()
                    .draw_wireframes(Style3d::builder().color(&RED).thickness(0.1).build())
                    .should_annotate(true)
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
