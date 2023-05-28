use plotz_geometry::traits::AnnotationSettings;

use {
    argh::FromArgs,
    itertools::iproduct,
    plotz_color::*,
    plotz_core::{canvas::Canvas, frame::*, svg::Size},
    plotz_geometry::{
        crop::Croppable,
        grid_layout::{GridLayout, GridLayoutSettings},
        group::Group,
        object2d::Object2d,
        point::Pt,
        polygon::{Polygon, Rect},
        traits::Annotatable,
    },
};

#[derive(FromArgs)]
#[argh(description = "...")]
struct Args {
    #[argh(option, description = "output path")]
    output_path_prefix: String,
}

fn main() {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .compact()
            .with_max_level(tracing::Level::INFO)
            .without_time()
            .finish(),
    )
    .expect("setting default subscriber failed");

    let args: Args = argh::from_env();

    let mut dos = vec![];
    let mgn = 25.0;

    let frame: Object2d = make_frame_with_margin((1000.0, 800.0), mgn);

    let mut gl = GridLayout::new(
        GridLayoutSettings::builder()
            .init((30, 30))
            .dims((700, 900))
            .divisions((7, 7))
            .object_margin((5, 5))
            .build(),
    );

    let f = 10.0;
    for (idx, offset) in iproduct!(0..=5, 0..=4)
        .map(|(i, j)| ((i, j), Pt((i as f64 - 3.0) * f, (j as f64 - 3.0) * f)))
    // .filter(|(idx, _)| *idx == (2, 2))
    {
        let r = Rect(Pt(50.0, 50.0), (50.0, 50.0)).unwrap();

        let base_sq = Object2d::new(r.clone())
            .with_color(&BLACK)
            .with_thickness(2.0);
        let base_sq_annotations = base_sq.annotate(&AnnotationSettings::default());

        let pts = if false {
            let a = Pt(60.0, 60.0);
            let b = Pt(70.0, 60.0);
            let c = Pt(80.0, 60.0);
            let d = Pt(90.0, 60.0);
            let e = Pt(70.0, 75.0);
            let f = Pt(80.0, 75.0);
            let g = Pt(60.0, 90.0);
            let h = Pt(90.0, 90.0);
            vec![a, b, e, f, c, d, h, g, a]
        } else {
            let a = Pt(60.0, 40.0);
            let b = Pt(70.0, 40.0);
            let c = Pt(70.0, 70.0);
            let d = Pt(80.0, 70.0);
            let e = Pt(80.0, 40.0);
            let f = Pt(90.0, 40.0);
            let g = Pt(90.0, 110.0);
            let h = Pt(80.0, 110.0);
            let i = Pt(80.0, 80.0);
            let j = Pt(70.0, 80.0);
            let k = Pt(70.0, 110.0);
            let l = Pt(60.0, 110.0);
            vec![a, b, c, d, e, f, g, h, i, j, k, l, a]
        };

        let subject_sq = Object2d::new(Polygon(pts))
            .with_color(&RED)
            .with_thickness(1.0)
            + offset;
        // let subject_sq_annotations = subject_sq.annotate();

        let mut v: Vec<Object2d> = vec![];
        // v.push(base_sq);
        // v.push(subject_sq.clone());
        v.extend(base_sq_annotations);
        // v.extend(subject_sq_annotations);

        // v.extend( subject_sq .crop_to(&r) .into_iter() .map(|o| o.with_color(&GREEN).with_thickness(2.0)),);
        v.extend(
            subject_sq
                .crop_excluding(&r.clone())
                .into_iter()
                .map(|o| o.with_color(&BLUE).with_thickness(2.0)),
        );

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
