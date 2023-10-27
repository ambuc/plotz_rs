use anyhow::Result;
use argh::FromArgs;
use itertools::iproduct;
use plotz_color::*;
use plotz_core::{canvas::Canvas, frame::*, svg::Size};
use plotz_geometry::{
    crop::Croppable,
    grid::grid_layout::{GridLayout, GridLayoutSettings},
    group::Group,
    obj::Obj,
    shapes::{
        pg::{Pg, Rect},
        pt::Pt,
    },
    style::Style,
};

#[derive(FromArgs)]
#[argh(description = "...")]
struct Args {
    #[argh(option, description = "output path")]
    output_path_prefix: String,
}

fn main() -> Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .compact()
            .with_max_level(tracing::Level::INFO)
            .without_time()
            .finish(),
    )?;

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
        .map(|(i, j)| ((i, j), Pt((i as f64 - 3.0) * f, (j as f64 - 3.0) * f)))
    // .filter(|(idx, _)| *idx == (1, 2))
    {
        let mut v: Vec<(Obj, Style)> = vec![];

        let r = Rect((50, 50), (50, 50)).unwrap();

        let base_sq = (
            Obj::Pg(r.clone()),
            Style {
                thickness: 2.0,
                ..Default::default()
            },
        );
        v.push(base_sq.clone());
        // v.extend(base_sq.annotate(&AnnotationSettings::default()));

        let pts = if false {
            let a = Pt(60, 60);
            let b = Pt(70, 60);
            let c = Pt(80, 60);
            let d = Pt(90, 60);
            let e = Pt(70, 75);
            let f = Pt(80, 75);
            let g = Pt(60, 90);
            let h = Pt(90, 90);
            vec![a, b, e, f, c, d, h, g, a]
        } else {
            let a = Pt(60, 40);
            let b = Pt(70, 40);
            let c = Pt(70, 70);
            let d = Pt(80, 70);
            let e = Pt(80, 40);
            let f = Pt(90, 40);
            let g = Pt(90, 110);
            let h = Pt(80, 110);
            let i = Pt(80, 80);
            let j = Pt(70, 80);
            let k = Pt(70, 110);
            let l = Pt(60, 110);
            vec![a, b, c, d, e, f, g, h, i, j, k, l, a]
        };

        let subject_sq = (
            Obj::Pg(Pg(pts)) + offset,
            Style {
                color: RED,
                ..Default::default()
            },
        );
        // v.push(subject_sq.clone());
        // v.extend(subject_sq.annotate(&AnnotationSettings::default()));

        let r = r.clone();
        v.extend(
            subject_sq
                .0
                .crop_to(&r)
                .expect("todo")
                .into_iter()
                .map(|x| {
                    (
                        x,
                        Style {
                            color: GREEN,
                            thickness: 2.0,
                            ..Default::default()
                        },
                    )
                }),
        );

        v.extend(
            subject_sq
                .0
                .crop_excluding(&r.clone())
                .expect("todo")
                .into_iter()
                .map(|x| {
                    (
                        x,
                        Style {
                            color: BLUE,
                            ..Default::default()
                        },
                    )
                }),
        );

        gl.insert_and_rescale_to_cubby(
            idx,
            (Obj::Group(Group::new(v.into_iter())), Style::default()),
            1.00,
        )?;
    }

    dos.extend(gl.to_object2ds());

    let objs = Canvas::from_objs(dos.into_iter(), /*autobucket=*/ false).with_frame(frame);

    objs.write_to_svg(
        Size {
            width: 800,
            height: 1000,
        },
        &args.output_path_prefix,
    )?;
    Ok(())
}
