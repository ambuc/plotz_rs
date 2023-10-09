use plotz_geometry::{obj2::Obj2, style::Style};

use {
    argh::FromArgs,
    itertools::iproduct,
    plotz_color::*,
    plotz_core::{canvas::Canvas, frame::*, svg::Size},
    plotz_geometry::{
        crop::Croppable,
        grid::grid_layout::{GridLayout, GridLayoutSettings},
        group::Group,
        shapes::{
            pg2::{Pg2, Rect},
            pt2::Pt2,
        },
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

    let frame = make_frame_with_margin((1000.0, 800.0), mgn);

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
        .map(|(i, j)| ((i, j), Pt2((i as f64 - 3.0) * f, (j as f64 - 3.0) * f)))
    // .filter(|(idx, _)| *idx == (1, 2))
    {
        let mut v: Vec<(Obj2, Style)> = vec![];

        let r = Rect(Pt2(50.0, 50.0), (50.0, 50.0)).unwrap();

        let base_sq = (
            Obj2::Pg2(r.clone()),
            Style::builder().thickness(2.0).build(),
        );
        v.push(base_sq.clone());
        // v.extend(base_sq.annotate(&AnnotationSettings::default()));

        let pts = if false {
            let a = Pt2(60.0, 60.0);
            let b = Pt2(70.0, 60.0);
            let c = Pt2(80.0, 60.0);
            let d = Pt2(90.0, 60.0);
            let e = Pt2(70.0, 75.0);
            let f = Pt2(80.0, 75.0);
            let g = Pt2(60.0, 90.0);
            let h = Pt2(90.0, 90.0);
            vec![a, b, e, f, c, d, h, g, a]
        } else {
            let a = Pt2(60.0, 40.0);
            let b = Pt2(70.0, 40.0);
            let c = Pt2(70.0, 70.0);
            let d = Pt2(80.0, 70.0);
            let e = Pt2(80.0, 40.0);
            let f = Pt2(90.0, 40.0);
            let g = Pt2(90.0, 110.0);
            let h = Pt2(80.0, 110.0);
            let i = Pt2(80.0, 80.0);
            let j = Pt2(70.0, 80.0);
            let k = Pt2(70.0, 110.0);
            let l = Pt2(60.0, 110.0);
            vec![a, b, c, d, e, f, g, h, i, j, k, l, a]
        };

        let subject_sq = (
            Obj2::Pg2(Pg2(pts)) + offset,
            Style::builder().color(&RED).build(),
        );
        // v.push(subject_sq.clone());
        // v.extend(subject_sq.annotate(&AnnotationSettings::default()));

        let r = r.clone();
        v.extend(
            subject_sq
                .0
                .crop_to(&r)
                .into_iter()
                .map(|x| (x, Style::builder().color(&GREEN).thickness(2.0).build())),
        );

        v.extend(
            subject_sq
                .0
                .crop_excluding(&r.clone())
                .into_iter()
                .map(|x| (x, Style::builder().color(&BLUE).thickness(2.0).build())),
        );

        gl.insert_and_rescale_to_cubby(
            idx,
            (Obj2::Group(Group::new(v.into_iter())), Style::default()),
            1.00,
        );
    }

    dos.extend(gl.to_object2ds());

    let objs = Canvas::from_objs(dos.into_iter(), /*autobucket=*/ false).with_frame(frame);

    objs.write_to_svg_or_die(
        Size {
            width: 800,
            height: 1000,
        },
        &args.output_path_prefix,
    );
}
