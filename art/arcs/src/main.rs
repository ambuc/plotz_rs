use plotz_geometry::{obj::Obj, shapes::pg::Pg, style::Style};

use argh::FromArgs;
use plotz_color::*;
use plotz_core::{canvas::Canvas, frame::make_frame, svg::Size};
use plotz_geometry::{bounded::Bounded, crop::Croppable, grid::Grid, shapes::curve::CurveArcs};
use rand::Rng;
use std::f64::consts::*;

#[derive(FromArgs)]
#[argh(description = "...")]
struct Args {
    #[argh(option, description = "output path")]
    output_path_prefix: String,
}

fn main() {
    let args: Args = argh::from_env();

    let mut dos = vec![];
    let mgn = 25.0;

    let mut rng = rand::thread_rng();

    let frame = make_frame(
        (1000.0 - 2.0 * mgn, 800.0 - 2.0 * mgn),
        /*offset=*/ (mgn, mgn),
    );
    {
        let frame_polygon: Pg = frame.0.clone().try_into().unwrap();

        let frame_ctr = frame.0.bbox_center();

        for i in 1..200 {
            let i: f64 = i as f64;

            let ctr = frame_ctr;

            let d = (200.0 - i) / 50.0;
            let angle_1 = 0.0 + d * 3.0 + (rng.gen_range(0.0..d));
            let angle_2 = angle_1 + PI;

            let radius = i * 1.6;

            let cas = CurveArcs(ctr, angle_1..=angle_2, radius);

            dos.extend(cas.iter().flat_map(|ca| ca.crop_to(&frame_polygon)).map(
                |ca| -> (Obj, Style) {
                    (
                        Obj::CurveArc(ca),
                        Style {
                            color: &GREEN,
                            thickness: 0.30,
                            ..Default::default()
                        },
                    )
                },
            ));
        }

        dos.extend(
            Grid::builder()
                .width(800)
                .height(1000)
                .build()
                .to_segments()
                .into_iter(),
        );
    }

    let objs = Canvas::from_objs(dos.into_iter(), /*autobucket=*/ false).with_frame(frame);

    objs.write_to_svg_or_die(
        Size {
            width: 800,
            height: 1000,
        },
        &args.output_path_prefix,
    )
}
