use {
    argh::FromArgs,
    plotz_color::*,
    plotz_core::{canvas::Canvas, frame::make_frame, svg::Size},
    plotz_geometry::{draw_obj::DrawObj, point::Pt},
    plotz_geometry3d::{
        camera::{Camera, Oblique, Projection},
        object::Object,
        point3d::Pt3d,
        polygon3d::Fill,
        polygon3d::Polygon3d,
        scene::Scene,
        segment3d::Segment3d,
        style::Style,
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

    let frame: DrawObj = make_frame(
        (1000.0 - 2.0 * mgn, 800.0 - 2.0 * mgn),
        /*offset=*/ Pt(mgn, mgn),
    );

    let dos: Vec<DrawObj> = {
        let origin_3d = Pt3d(0.0, 0.0, 0.0);

        let mut objects = vec![];

        let axes: Vec<Object> = vec![
            (Pt3d(1.0, 0.0, 0.0), &RED),
            (Pt3d(0.0, 1.0, 0.0), &BLUE),
            (Pt3d(0.0, 0.0, 1.0), &GREEN),
        ]
        .iter()
        .map(|(diff, color)| {
            Object::new(Segment3d(origin_3d, origin_3d + *diff))
                .with_style(Style::builder().color(color).thickness(2.0).build())
        })
        .collect();

        objects.extend(axes);

        objects.push(
            Object::new(Polygon3d(
                [
                    origin_3d + Pt3d(0.5, 0.0, 0.0),
                    origin_3d + Pt3d(0.0, 0.5, 0.0),
                    origin_3d + Pt3d(0.0, 0.0, 0.5),
                    origin_3d + Pt3d(0.5, 0.0, 0.0),
                ],
                Fill::Opaque,
            ))
            .with_style(Style::builder().color(&POWDERBLUE).thickness(2.0).build()),
        );

        let scene = Scene::from(objects);

        let camera = Camera::builder()
            .at(origin_3d + Pt3d(1.0, 1.0, 1.0))
            .towards(origin_3d)
            .up(Pt3d(0.0, 0.0, 1.0))
            .build();

        scene.project_onto(&camera, Projection::Oblique(Oblique::standard()))
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
