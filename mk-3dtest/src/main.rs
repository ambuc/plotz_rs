#![allow(unused_imports)]

use {
    argh::FromArgs,
    itertools::{iproduct, zip},
    plotz_color::*,
    plotz_core::{canvas::Canvas, frame::*, svg::Size},
    plotz_geometry::{
        obj2::Obj2,
        shapes::{pg2::Pg2, point::Pt},
        styled_obj2::StyledObj2,
        traits::AnnotationSettings,
    },
    plotz_geometry3d::{
        camera::{Oblique, Occlusion, Projection},
        occluder::{self, Occluder},
        p3,
        scene::{DebugSettings, Scene},
        shapes::{cube3d::Cube, point3d::Pt3, polygon3d::Pg3, segment3d::Sg3},
        style::Style3d,
        styled_obj3::StyledObj3,
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
    let frame: StyledObj2 = make_frame_with_margin((1000.0, 800.0), margin);

    let dos: Vec<StyledObj2> = {
        let origin_3d = p3!(0, 0, 0);

        let mut objects: Vec<StyledObj3> = vec![];

        if false {
            objects.extend(
                vec![
                    (p3!(1, 0, 0), &RED),
                    (p3!(0, 1, 0), &BLUE),
                    (p3!(0, 0, 1), &GREEN),
                ]
                .iter()
                .map(|(diff, color)| {
                    StyledObj3::new(Sg3(origin_3d, origin_3d + *diff))
                        .with_style(Style3d::new(color, 2.0))
                }),
            );
        }

        if true {
            let e = 0.65;
            let n = 5;
            for ((i, j, k), color) in zip(
                iproduct!(0..n, 0..n, 0..n),
                (vec![&RED, &YELLOW, &GREEN, &BLUE, &PLUM, &ORANGE])
                    .iter()
                    .cycle(),
            ) {
                objects.extend(
                    Cube(p3!(i, j, k), (e, e, e))
                        .items
                        .into_iter()
                        .map(|face| StyledObj3::new(face).with_style(Style3d::new(color, 3.0))),
                );
            }
        }

        if false {
            objects.extend(
                Cube(p3!(0, 0, 0), (0.7, 0.7, 1.0))
                    .items
                    .into_iter()
                    .map(|face| StyledObj3::new(face).with_style(Style3d::new(&RED, 3.0))),
            );
            objects.extend(
                Cube(p3!(1, 0, 0), (0.7, 0.7, 1.0))
                    .items
                    .into_iter()
                    .map(|face| StyledObj3::new(face).with_style(Style3d::new(&YELLOW, 3.0))),
            );
        }
        //
        if false {
            for pg3d in [
                Pg3([
                    p3!(0.00, 0.00, 0.00),
                    p3!(0.70, 0.00, 0.00),
                    p3!(0.70, 0.70, 0.00),
                    p3!(0.00, 0.70, 0.00),
                    p3!(0.00, 0.00, 0.00),
                ]),
                Pg3([
                    p3!(0.00, 0.00, 0.00),
                    p3!(0.70, 0.00, 0.00),
                    p3!(0.70, 0.00, 1.00),
                    p3!(0.00, 0.00, 1.00),
                    p3!(0.00, 0.00, 0.00),
                ]),
                Pg3([
                    p3!(0.00, 0.00, 0.00),
                    p3!(0.00, 0.70, 0.00),
                    p3!(0.00, 0.70, 1.00),
                    p3!(0.00, 0.00, 1.00),
                    p3!(0.00, 0.00, 0.00),
                ]),
                Pg3([
                    p3!(0.70, 0.00, 0.00),
                    p3!(0.70, 0.70, 0.00),
                    p3!(0.70, 0.70, 1.00),
                    p3!(0.70, 0.00, 1.00),
                    p3!(0.70, 0.00, 0.00),
                ]),
                Pg3([
                    p3!(0.00, 0.70, 0.00),
                    p3!(0.00, 0.70, 1.00),
                    p3!(0.70, 0.70, 1.00),
                    p3!(0.70, 0.70, 0.00),
                    p3!(0.00, 0.70, 0.00),
                ]),
                Pg3([
                    p3!(0.00, 0.00, 1.00),
                    p3!(0.70, 0.00, 1.00),
                    p3!(0.70, 0.70, 1.00),
                    p3!(0.00, 0.70, 1.00),
                    p3!(0.00, 0.00, 1.00),
                ]),
            ] {
                objects.push(StyledObj3::new(pg3d).with_style(Style3d::new(&RED, 6.0)));
            }
            for pg3d in [
                Pg3([
                    p3!(1.00, 0.00, 0.00),
                    p3!(1.70, 0.00, 0.00),
                    p3!(1.70, 0.70, 0.00),
                    p3!(1.00, 0.70, 0.00),
                    p3!(1.00, 0.00, 0.00),
                ]),
                Pg3([
                    p3!(1.00, 0.00, 0.00),
                    p3!(1.70, 0.00, 0.00),
                    p3!(1.70, 0.00, 1.00),
                    p3!(1.00, 0.00, 1.00),
                    p3!(1.00, 0.00, 0.00),
                ]),
                Pg3([
                    p3!(1.00, 0.00, 0.00),
                    p3!(1.00, 0.70, 0.00),
                    p3!(1.00, 0.70, 1.00),
                    p3!(1.00, 0.00, 1.00),
                    p3!(1.00, 0.00, 0.00),
                ]),
                Pg3([
                    p3!(1.70, 0.00, 0.00),
                    p3!(1.70, 0.70, 0.00),
                    p3!(1.70, 0.70, 1.00),
                    p3!(1.70, 0.00, 1.00),
                    p3!(1.70, 0.00, 0.00),
                ]),
                Pg3([
                    p3!(1.00, 0.00, 1.00),
                    p3!(1.70, 0.00, 1.00),
                    p3!(1.70, 0.70, 1.00),
                    p3!(1.00, 0.70, 1.00),
                    p3!(1.00, 0.00, 1.00),
                ]),
                Pg3([
                    p3!(1.00, 0.70, 0.00),
                    p3!(1.00, 0.70, 1.00),
                    p3!(1.70, 0.70, 1.00),
                    p3!(1.70, 0.70, 0.00),
                    p3!(1.00, 0.70, 0.00),
                ]),
            ] {
                objects.push(StyledObj3::new(pg3d).with_style(Style3d::new(&YELLOW, 3.0)));
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

    let mut canvas = Canvas::from_objs(dos.into_iter(), /*autobucket=*/ false).with_frame(frame);
    canvas.scale_to_fit_frame().unwrap();

    let () = canvas
        .write_to_svg((800, 1000), &args.output_path_prefix)
        .expect("write");
}
