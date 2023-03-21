use std::f64::consts::PI;

use {
    argh::FromArgs,
    plotz_color::*,
    plotz_core::{
        draw_obj::{DrawObj, DrawObjs},
        frame::make_frame,
        svg::Size,
    },
    plotz_geometry::{curve::CurveArc, point::Pt},
};

#[derive(FromArgs)]
#[argh(description = "...")]
struct Args {
    #[argh(option, description = "output path")]
    output_path_prefix: String,
}

fn main() {
    let args: Args = argh::from_env();

    let mgn = 20.0;
    let ell = 720.0;
    let asp = 1.3;

    let frame: DrawObj = make_frame((ell, ell * asp), /*offset=*/ Pt(mgn, mgn));

    let mut dos = vec![];
    for i in 1..130 {
        dos.push(
            DrawObj::from_curve_arc(CurveArc::new(
                Pt(
                    //c x
                    mgn + (ell * asp) / 2.0 
                    // c x off
                    + 1.3 * (i as f64) - 160.0,
                    //c y
                    (ell) / 2.0 + mgn 
                ),
                /*angle_1=*/
                0.0, // -1.0 * PI / 2.0,
                /*angle_2=*/
                3.0 * (i as f64) * 1.15 * PI / 100.0,
                /*radius=*/
                0.0 + (2.0 * i as f64),
            ))
            .with_color(&BLUE)
            .with_thickness(0.3),
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
