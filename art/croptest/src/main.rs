use anyhow::Result;
use argh::FromArgs;
use itertools::iproduct;
use plotz_color::*;
use plotz_core::{
    canvas::{self, Canvas},
    frame::*,
    svg::Size,
};
use plotz_geometry::{
    crop::Croppable,
    grid::grid_layout::{GridLayout, GridLayoutSettings},
    group::Group,
    obj::Obj,
    shapes::{
        point::Point,
        polygon::{Pg, Rect},
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

    let frame = make_frame_with_margin((1000.0, 800.0), mgn)?;

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
        .map(|(i, j)| ((i, j), Point((i as f64 - 3.0) * f, (j as f64 - 3.0) * f)))
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
            let a = Point(60, 60);
            let b = Point(70, 60);
            let c = Point(80, 60);
            let d = Point(90, 60);
            let e = Point(70, 75);
            let f = Point(80, 75);
            let g = Point(60, 90);
            let h = Point(90, 90);
            vec![a, b, e, f, c, d, h, g, a]
        } else {
            let a = Point(60, 40);
            let b = Point(70, 40);
            let c = Point(70, 70);
            let d = Point(80, 70);
            let e = Point(80, 40);
            let f = Point(90, 40);
            let g = Point(90, 110);
            let h = Point(80, 110);
            let i = Point(80, 80);
            let j = Point(70, 80);
            let k = Point(70, 110);
            let l = Point(60, 110);
            vec![a, b, c, d, e, f, g, h, i, j, k, l, a]
        };

        let subject_sq = (
            Obj::Pg(Pg(pts)?) + offset,
            Style {
                color: RED,
                ..Default::default()
            },
        );
        // v.push(subject_sq.clone());
        // v.extend(subject_sq.annotate(&AnnotationSettings::default()));

        let r = r.clone();
        v.extend(subject_sq.0.crop_to(&r).unwrap().into_iter().map(|x| {
            (
                x,
                Style {
                    color: GREEN,
                    thickness: 2.0,
                    ..Default::default()
                },
            )
        }));

        v.extend(
            subject_sq
                .0
                .crop_excluding(&r.clone())
                .unwrap()
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

    Canvas::builder()
        .dos_by_bucket(canvas::to_canvas_map(dos, /*autobucket=*/ false))
        .frame(frame)
        .build()
        .write_to_svg(
            Size {
                width: 800,
                height: 1000,
            },
            &args.output_path_prefix,
        )?;
    Ok(())
}
