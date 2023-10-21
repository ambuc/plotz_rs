use anyhow::Result;
use argh::FromArgs;
use plotz_color::*;
use plotz_core::{canvas::Canvas, frame::make_frame, svg::Size};
use plotz_geometry::{
    crop::PointLoc,
    grid::grid_layout::{GridLayout, GridLayoutSettings},
    obj::Obj,
    shapes::{curve::CurveArc, pt::Pt},
    style::Style,
};
use rand::{seq::SliceRandom, thread_rng, Rng};
use std::f64::consts::*;
use tracing::*;
use tracing_subscriber::FmtSubscriber;

#[derive(FromArgs)]
#[argh(description = "...")]
struct Args {
    #[argh(option, description = "output path")]
    output_path_prefix: String,
}

fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .compact()
        .with_max_level(Level::TRACE)
        .without_time()
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    let args: Args = argh::from_env();
    trace!("Running.");

    let mut dos = vec![];
    let mgn = 25.0;

    let frame = make_frame(
        (1000.0 - 2.0 * mgn, 800.0 - 2.0 * mgn),
        /*offset=*/ (mgn, mgn),
    );

    {
        let mut grid_layout = GridLayout::new(
            GridLayoutSettings::builder()
                .init((25, 25))
                .dims((750, 950))
                .divisions((4, 5))
                .object_margin((5, 5))
                .build(),
        );

        let mut rng = thread_rng();
        for i in 0..grid_layout.num_cubbys_x() {
            for j in 0..grid_layout.num_cubbys_y() {
                let cubby = (i, j);
                let bounds = grid_layout.get_cubby_bounds(cubby);
                for color in COLORS[0..3].choose_multiple(&mut rng, 3) {
                    let curve_arc_ctr: Pt = || -> Pt {
                        loop {
                            let cand = Pt(rng.gen_range(0.0..800.0), rng.gen_range(0.0..1000.0));
                            if !matches!(bounds.contains_pt(cand), Ok(PointLoc::Inside)) {
                                return cand;
                            }
                        }
                    }();
                    let rstep = rng.gen_range(10..20);
                    for r in (0..2000).step_by(rstep) {
                        grid_layout
                            .insert_and_crop_to_cubby(
                                cubby,
                                (
                                    Obj::CurveArc(CurveArc(curve_arc_ctr, 0.0..=TAU, r as f64)),
                                    Style {
                                        thickness: 1.0,
                                        color,
                                        ..Default::default()
                                    },
                                ),
                            )
                            .expect("ok");
                    }
                }
            }
        }

        dos.extend(grid_layout.to_object2ds());
    }

    let objs = Canvas::from_objs(dos.into_iter(), /*autobucket=*/ true).with_frame(frame);

    objs.write_to_svg(
        Size {
            width: 800,
            height: 1000,
        },
        &args.output_path_prefix,
    )?;
    Ok(())
}
