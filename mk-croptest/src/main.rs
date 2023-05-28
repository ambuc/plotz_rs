use {
    argh::FromArgs,
    itertools::iproduct,
    plotz_color::*,
    plotz_core::{canvas::Canvas, frame::*, svg::Size},
    plotz_geometry::{
        crop::Croppable,
        grid::grid_layout::{GridLayout, GridLayoutSettings},
        group::Group,
        p2,
        shapes::{
            pg2::{Pg2, Rect},
            pt2::Pt2,
        },
        styled_obj2::StyledObj2,
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

    let frame: StyledObj2 = make_frame_with_margin((1000.0, 800.0), mgn);

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
        .map(|(i, j)| ((i, j), p2!((i as f64 - 3.0) * f, (j as f64 - 3.0) * f)))
    // .filter(|(idx, _)| *idx == (1, 2))
    {
        let mut v: Vec<StyledObj2> = vec![];

        let r = Rect(p2!(50.0, 50.0), (50.0, 50.0)).unwrap();

        let base_sq = StyledObj2::new(r.clone())
            .with_color(&BLACK)
            .with_thickness(2.0);
        v.push(base_sq.clone());
        // v.extend(base_sq.annotate(&AnnotationSettings::default()));

        let pts = if false {
            let a = p2!(60.0, 60.0);
            let b = p2!(70.0, 60.0);
            let c = p2!(80.0, 60.0);
            let d = p2!(90.0, 60.0);
            let e = p2!(70.0, 75.0);
            let f = p2!(80.0, 75.0);
            let g = p2!(60.0, 90.0);
            let h = p2!(90.0, 90.0);
            vec![a, b, e, f, c, d, h, g, a]
        } else {
            let a = p2!(60.0, 40.0);
            let b = p2!(70.0, 40.0);
            let c = p2!(70.0, 70.0);
            let d = p2!(80.0, 70.0);
            let e = p2!(80.0, 40.0);
            let f = p2!(90.0, 40.0);
            let g = p2!(90.0, 110.0);
            let h = p2!(80.0, 110.0);
            let i = p2!(80.0, 80.0);
            let j = p2!(70.0, 80.0);
            let k = p2!(70.0, 110.0);
            let l = p2!(60.0, 110.0);
            vec![a, b, c, d, e, f, g, h, i, j, k, l, a]
        };

        let subject_sq = StyledObj2::new(Pg2(pts))
            .with_color(&RED)
            .with_thickness(1.0)
            + offset;
        // v.push(subject_sq.clone());
        // v.extend(subject_sq.annotate(&AnnotationSettings::default()));

        v.extend(
            subject_sq
                .crop_to(&r)
                .into_iter()
                .map(|o| o.with_color(&GREEN).with_thickness(2.0)),
        );

        v.extend(
            subject_sq
                .crop_excluding(&r.clone())
                .into_iter()
                .map(|o| o.with_color(&BLUE).with_thickness(2.0)),
        );

        let g = Group::new(v);

        gl.insert_and_rescale_to_cubby(idx, StyledObj2::new(g), 1.00);
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
