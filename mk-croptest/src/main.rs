use itertools::iproduct;
use plotz_geometry::{
    crop,
    grid_layout::{GridLayout, GridLayoutSettings},
    group::Group,
    polygon::Polygon,
    traits::Annotatable,
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

    let mut gl = GridLayout::new(
        GridLayoutSettings::builder()
            .init((30, 30))
            .dims((700, 900))
            .divisions((7, 7))
            .object_margin((5, 5))
            .build(),
    );

    let f = 12.5;
    for (idx, offset) in iproduct!(0..=6, 0..=6)
        .map(|(i, j)| ((i, j), Pt((i as f64 - 3.0) * f, (j as f64 - 3.0) * f)))
    //    .filter(|(idx, _)| *idx == (5, 1))
    {
        let r = Rect(Pt(50.0, 50.0), (50.0, 50.0)).unwrap();

        let base_sq = Object2d::new(r.clone())
            .with_color(&BLACK)
            .with_thickness(2.0);
        // let base_sq_annotations = base_sq.annotate();

        let a = Pt(60.0, 60.0);
        let b = Pt(70.0, 60.0);
        let c = Pt(80.0, 60.0);
        let d = Pt(90.0, 60.0);
        let e = Pt(70.0, 75.0);
        let f = Pt(80.0, 75.0);
        let g = Pt(60.0, 90.0);
        let h = Pt(90.0, 90.0);

        let subject_sq = Object2d::new(Polygon([a, b, e, f, c, d, h, g, a]).unwrap())
            .with_color(&RED)
            .with_thickness(1.0)
            + offset;
        // let subject_sq_annotations = subject_sq.annotate();

        let cropped_sqs: Vec<Object2d> = subject_sq
            .crop_to(&r)
            .unwrap()
            .into_iter()
            .map(|o| o.with_color(&GREEN).with_thickness(2.0))
            .collect();

        let mut v: Vec<Object2d> = vec![base_sq, subject_sq];
        // v.extend(base_sq_annotations);
        // v.extend(subject_sq_annotations);
        v.extend(cropped_sqs);

        let g = Group::new(v);

        gl.insert_and_rescale_to_cubby(idx, Object2d::new(g), 1.00);
    }

    dos.extend(gl.to_object2ds());

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
