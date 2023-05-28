#![allow(unused_imports)]

use itertools::zip;
use plotz_geometry::{object2d_inner::Object2dInner, polygon::Polygon, traits::AnnotationSettings};
use plotz_geometry3d::occluder::{self, Occluder};
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

        if true {
            let e = 0.65;
            for ((i, j, k), color) in zip(
                iproduct!(0..3, 0..3, 0..3),
                (vec![&RED, &YELLOW, &GREEN, &BLUE, &PLUM, &ORANGE])
                    .iter()
                    .cycle(),
            ) {
                let zh = 1.0;
                objects.extend(
                    Cube(p3!(i, j, k), (e, e, e))
                        .items
                        .into_iter()
                        .map(|face| Object3d::new(face).with_style(Style3d::new(color, 3.0))),
                );
            }
        }

        if false {
            objects.extend(
                Cube(p3!(0, 0, 0), (0.7, 0.7, 1.0))
                    .items
                    .into_iter()
                    .map(|face| Object3d::new(face).with_style(Style3d::new(&RED, 3.0))),
            );
            objects.extend(
                Cube(p3!(1, 0, 0), (0.7, 0.7, 1.0))
                    .items
                    .into_iter()
                    .map(|face| Object3d::new(face).with_style(Style3d::new(&YELLOW, 3.0))),
            );
        }
        //
        if false {
            for pg3d in [
                Polygon3d([
                    p3!(0.00, 0.00, 0.00),
                    p3!(0.70, 0.00, 0.00),
                    p3!(0.70, 0.70, 0.00),
                    p3!(0.00, 0.70, 0.00),
                    p3!(0.00, 0.00, 0.00),
                ]),
                Polygon3d([
                    p3!(0.00, 0.00, 0.00),
                    p3!(0.70, 0.00, 0.00),
                    p3!(0.70, 0.00, 1.00),
                    p3!(0.00, 0.00, 1.00),
                    p3!(0.00, 0.00, 0.00),
                ]),
                Polygon3d([
                    p3!(0.00, 0.00, 0.00),
                    p3!(0.00, 0.70, 0.00),
                    p3!(0.00, 0.70, 1.00),
                    p3!(0.00, 0.00, 1.00),
                    p3!(0.00, 0.00, 0.00),
                ]),
                Polygon3d([
                    p3!(0.70, 0.00, 0.00),
                    p3!(0.70, 0.70, 0.00),
                    p3!(0.70, 0.70, 1.00),
                    p3!(0.70, 0.00, 1.00),
                    p3!(0.70, 0.00, 0.00),
                ]),
                Polygon3d([
                    p3!(0.00, 0.70, 0.00),
                    p3!(0.00, 0.70, 1.00),
                    p3!(0.70, 0.70, 1.00),
                    p3!(0.70, 0.70, 0.00),
                    p3!(0.00, 0.70, 0.00),
                ]),
                Polygon3d([
                    p3!(0.00, 0.00, 1.00),
                    p3!(0.70, 0.00, 1.00),
                    p3!(0.70, 0.70, 1.00),
                    p3!(0.00, 0.70, 1.00),
                    p3!(0.00, 0.00, 1.00),
                ]),
            ] {
                objects.push(Object3d::new(pg3d).with_style(Style3d::new(&RED, 6.0)));
            }
            for pg3d in [
                Polygon3d([
                    p3!(1.00, 0.00, 0.00),
                    p3!(1.70, 0.00, 0.00),
                    p3!(1.70, 0.70, 0.00),
                    p3!(1.00, 0.70, 0.00),
                    p3!(1.00, 0.00, 0.00),
                ]),
                Polygon3d([
                    p3!(1.00, 0.00, 0.00),
                    p3!(1.70, 0.00, 0.00),
                    p3!(1.70, 0.00, 1.00),
                    p3!(1.00, 0.00, 1.00),
                    p3!(1.00, 0.00, 0.00),
                ]),
                Polygon3d([
                    p3!(1.00, 0.00, 0.00),
                    p3!(1.00, 0.70, 0.00),
                    p3!(1.00, 0.70, 1.00),
                    p3!(1.00, 0.00, 1.00),
                    p3!(1.00, 0.00, 0.00),
                ]),
                Polygon3d([
                    p3!(1.70, 0.00, 0.00),
                    p3!(1.70, 0.70, 0.00),
                    p3!(1.70, 0.70, 1.00),
                    p3!(1.70, 0.00, 1.00),
                    p3!(1.70, 0.00, 0.00),
                ]),
                Polygon3d([
                    p3!(1.00, 0.00, 1.00),
                    p3!(1.70, 0.00, 1.00),
                    p3!(1.70, 0.70, 1.00),
                    p3!(1.00, 0.70, 1.00),
                    p3!(1.00, 0.00, 1.00),
                ]),
                Polygon3d([
                    p3!(1.00, 0.70, 0.00),
                    p3!(1.00, 0.70, 1.00),
                    p3!(1.70, 0.70, 1.00),
                    p3!(1.70, 0.70, 0.00),
                    p3!(1.00, 0.70, 0.00),
                ]),
            ] {
                objects.push(Object3d::new(pg3d).with_style(Style3d::new(&YELLOW, 3.0)));
            }
        }

        let scene = Scene::builder()
            .debug(
                DebugSettings::builder()
                    // .draw_wireframes(Style3d::new(&GRAY, 0.5))
                    // .annotate( AnnotationSettings::builder() .font_size(12.0) .precision(3) .build(),)
                    .build(),
            )
            .objects(objects)
            .build();
        scene.project_with(Projection::Oblique(Oblique::standard()), Occlusion::True)
    };

    // let dos = {
    //     let p1 = Polygon([ Pt(-1.0000000000, 1.6800000000), Pt(-1.0000000000, 0.6800000000), Pt(-0.3000000000, 0.1900000000), Pt(-0.3000000000, 1.1900000000), ]);
    //     let p2 = Polygon([ Pt(-0.7000000000, -0.5100000000), Pt(0.0000000000, -0.0200000000), Pt(0.0000000000, 0.9800000000), Pt(-0.7000000000, 0.4900000000), Pt(-0.7000000000, 0.4700000000), Pt(-0.3000000000, 0.1900000000), Pt(-0.7000000000, -0.0900000000), ]);

    //     let mut o = Occluder::new();
    //     o.add(p1.clone().into(), Some(Style3d::new(&YELLOW, 3.0)));
    //     o.add(p2.clone().into(), Some(Style3d::new(&RED, 6.0)));

    //     let mut dos = vec![];
    //     dos.push(Object2d::new(p1).with_color(&ORANGE).with_thickness(9.0));
    //     dos.push(Object2d::new(p2).with_color(&BROWN).with_thickness(12.0));
    //     dos.extend(o.export());
    //     dos

    // };

    let mut canvas = Canvas::from_objs(dos.into_iter(), /*autobucket=*/ false).with_frame(frame);
    canvas.scale_to_fit_frame().unwrap();

    let () = canvas
        .write_to_svg((800, 1000), &args.output_path_prefix)
        .expect("write");
}
