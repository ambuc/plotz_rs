use plotz_geometry::bounded::Bounded;

use {
    argh::FromArgs,
    plotz_color::*,
    plotz_core::{
        draw_obj::{DrawObj, DrawObjs},
        frame::make_frame,
        svg::Size,
    },
    plotz_geometry::{curve::CurveArc, point::Pt},
    std::f64::consts::PI,
};

#[derive(FromArgs)]
#[argh(description = "...")]
struct Args {
    #[argh(option, description = "output path")]
    output_path_prefix: String,
}

fn main() {
    let args: Args = argh::from_env();

    let mgn = 100.0; // 20.0;
    let ell = 500.0; // 720.0;
    let asp = 1.3;

    let frame: DrawObj = make_frame((ell, ell * asp), /*offset=*/ Pt(mgn, mgn));

    let frame_ctr = frame.obj.bbox_center();

    let mut dos = vec![];

    for i in 1..100 {
        let i: f64 = i as f64;

        let ctr = frame_ctr;

        let angle_1 = 0.0 + 0.3 * i;
        let angle_2 = 0.5 * PI + 0.4 * i;

        let radius = 1.0 + 10.0 * i;

        let ca = CurveArc::new(ctr, angle_1, angle_2, radius);

        dos.push(
            DrawObj::new(ca)
                .with_color(&BROWN)
                .with_thickness(1.0),
        );
    }

    let draw_objs = DrawObjs::from_objs(dos).with_frame(frame);

    let () = draw_objs
        .write_to_svg(
            Size {
                width: (750.0 * 1.3) as usize,
                height: 750,
            },
            &args.output_path_prefix,
        )
        .expect("write");
}
