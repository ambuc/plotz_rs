use itertools::iproduct;
use plotz_geometry3d::camera::Occlusion;

use {
    argh::FromArgs,
    plotz_color::*,
    plotz_core::{canvas::Canvas, frame::make_frame, svg::Size},
    plotz_geometry::{object2d::Object2d, point::Pt},
    plotz_geometry3d::{
        camera::{Oblique, Projection},
        cube3d::Cube,
        object3d::Object3d,
        point3d::Pt3d,
        polygon3d::Polygon3d,
        scene::Scene,
        segment3d::Segment3d,
        style::Style3d,
    },
};

#[derive(FromArgs)]
#[argh(description = "...")]
struct Args {
    #[argh(option, description = "output path")]
    output_path_prefix: String,
}

fn main() {
    let args: Args = argh::from_env();

    let mgn = 25.0;

    let frame: Object2d = make_frame(
        (1000.0 - 2.0 * mgn, 800.0 - 2.0 * mgn),
        /*offset=*/ Pt(mgn, mgn),
    );

    let dos: Vec<Object2d> = {
        let origin_3d = Pt3d(0.0, 0.0, 0.0);

        let mut objects: Vec<Object3d> = vec![];

        let axes: Vec<Object3d> = vec![
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

        // objects.extend(axes);

        let e = 0.6;
        for (i, j) in iproduct!(0..1, 0..2) {
            objects.extend(
                Cube(Pt3d(i as f64, j as f64, 0.0), (e, e, e))
                    .items
                    .into_iter()
                    .map(|face| {
                        Object3d::new(face)
                            .with_style(Style3d::builder().color(&PINK).thickness(1.0).build())
                    }),
            );
        }

        // // Triangle.
        // objects.push(
        //     Object3d::new(Polygon3d([
        //         origin_3d + Pt3d(0.5, 0.0, 0.0),
        //         origin_3d + Pt3d(0.0, 0.5, 0.0),
        //         origin_3d + Pt3d(0.0, 0.0, 0.5),
        //         origin_3d + Pt3d(0.5, 0.0, 0.0),
        //     ]))
        //     .with_style(Style3d::builder().color(&POWDERBLUE).thickness(2.0).build()),
        // );

        Scene::from(objects).project_with(Projection::Oblique(Oblique::standard()), Occlusion::True)
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