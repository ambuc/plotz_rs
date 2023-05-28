use {
    argh::FromArgs,
    itertools::{iproduct, zip},
    plotz_color::*,
    plotz_core::{canvas::Canvas, frame::*},
    plotz_geometry::styled_obj2::StyledObj2,
    plotz_geometry3d::{
        camera::{Occlusion, Projection},
        p3,
        scene::{debug::SceneDebug, Scene},
        shapes::{cube3d::Cube, pg3::Pg3, pt3::Pt3, sg3::Sg3},
        style::Style3d,
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

        let objects: Vec<StyledObj3> =
            {
                let mut objects = vec![];
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
                    let e = 0.85;
                    let n = 5;
                    for ((i, j, k), color) in zip(
                        iproduct!(0..n, 0..n, 0..n),
                        (vec![&RED, &YELLOW, &GREEN, &BLUE, &PLUM, &ORANGE])
                            .iter()
                            .cycle(),
                    ) {
                        objects.extend(Cube(p3!(i, j, k), (e, e, e)).items.into_iter().map(
                            |face| StyledObj3::new(face).with_style(Style3d::new(color, 3.0)),
                        ));
                    }
                }

                if false {
                    let red = Style3d::new(&RED, 3.0);
                    let yellow = Style3d::new(&YELLOW, 3.0);

                    objects.extend(
                        Cube(p3!(0, 0, 0), (0.7, 0.7, 1.0))
                            .items
                            .into_iter()
                            .map(|face| StyledObj3::new(face).with_style(red)),
                    );
                    objects.extend(
                        Cube(p3!(1, 0, 0), (0.7, 0.7, 1.0))
                            .items
                            .into_iter()
                            .map(|face| StyledObj3::new(face).with_style(yellow)),
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
                objects
            };

        Scene::builder()
            .debug(
                SceneDebug::builder()
                    // .draw_wireframes(Style3d::new(&GRAY, 0.5))
                    // .annotate( AnnotationSettings::builder() .font_size(12.0) .precision(3) .build(),)
                    .build(),
            )
            .objects(objects)
            .build()
            .project_with(Projection::default(), Occlusion::True)
    };

    Canvas::from_objs(dos.into_iter(), /*autobucket=*/ false)
        .with_frame(frame)
        .scale_to_fit_frame_or_die()
        .write_to_svg_or_die((800, 1000), &args.output_path_prefix);
}
