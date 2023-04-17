use plotz_geometry::{
    crop,
    grid_layout::{GridLayout, GridLayoutSettings},
    group::Group,
};

use {
    argh::FromArgs,
    plotz_color::*,
    plotz_core::{canvas::Canvas, frame::make_frame, svg::Size},
    plotz_geometry::{
        bounded::Bounded, crop::Croppable, curve::CurveArcs, grid::Grid, object2d::Object2d,
        object2d_inner::Object2dInner, point::Pt, polygon::Rect,
    },
    rand::Rng,
    std::f64::consts::*,
};

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

    let frame: Object2d = make_frame(
        (1000.0 - 2.0 * mgn, 800.0 - 2.0 * mgn),
        /*offset=*/ Pt(mgn, mgn),
    );

    {
        let offset = Pt(30.0, 30.0);
        let r = Rect(Pt(30.0, 30.0), (50.0, 50.0)).unwrap();
        let base_sq = Object2d::new(r.clone())
            .with_color(&BLACK)
            .with_thickness(2.0);
        let subject_sq = Object2d::new(r.clone())
            .with_color(&RED)
            .with_thickness(1.0)
            + offset;
        let cropped_sqs: Vec<Object2d> = subject_sq
            .crop_to(&r)
            .unwrap()
            .into_iter()
            .map(|o| o.with_color(&GREEN).with_thickness(2.0))
            .collect();

        dos.push(base_sq);
        dos.push(subject_sq);
        dos.extend(cropped_sqs);
    }

    let objs = Canvas::from_objs(dos.into_iter(), /*autobucket=*/ false).with_frame(frame);

    let () = objs
        .write_to_svg(
            Size {
                width: 800,
                height: 1000,
            },
            &args.output_path_prefix,
        )
        .expect("write");
}
