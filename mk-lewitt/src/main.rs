use {
    argh::FromArgs,
    plotz_color::*,
    plotz_core::{canvas::Canvas, frame::make_frame, svg::Size},
    plotz_geometry::{
        crop::PointLoc,
        curve::CurveArc,
        draw_obj::DrawObj,
        grid_layout::{GridLayout, GridLayoutSettings},
        point::Pt,
    },
    rand::{seq::SliceRandom, thread_rng, Rng},
    std::f64::consts::*,
    tracing::*,
    tracing_subscriber::FmtSubscriber,
};

#[derive(FromArgs)]
#[argh(description = "...")]
struct Args {
    #[argh(option, description = "output path")]
    output_path_prefix: String,
}

fn main() {
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

    let frame: DrawObj = make_frame(
        (1000.0 - 2.0 * mgn, 800.0 - 2.0 * mgn),
        /*offset=*/ Pt(mgn, mgn),
    );

    {
        let mut grid_layout = GridLayout::new(
            GridLayoutSettings::builder()
                .init((25, 25))
                .dims((750, 950))
                .divisions((10, 12))
                .object_margin((1, 1))
                .build(),
        );

        let mut rng = thread_rng();
        for i in 0..grid_layout.num_cubbys_x() {
            for j in 0..grid_layout.num_cubbys_y() {
                let cubby = (i, j);
                let bounds = grid_layout.get_cubby_bounds(cubby);
                for color in COLORS.choose_multiple(&mut rng, 5) {
                    let curve_arc_ctr: Pt = || -> Pt {
                        loop {
                            let cand = Pt(rng.gen_range(0.0..800.0), rng.gen_range(0.0..1000.0));
                            if !matches!(bounds.contains_pt(cand), Ok(PointLoc::Inside)) {
                                return cand;
                            }
                        }
                    }();
                    let rstep = rng.gen_range(5..10);
                    for r in (0..2000).step_by(rstep) {
                        let ca = CurveArc(curve_arc_ctr, 0.0..=TAU, r as f64);
                        let d_o = DrawObj::new(ca).with_thickness(1.0).with_color(color);
                        grid_layout.insert_and_crop_to_cubby(cubby, d_o);
                    }
                }
            }
        }

        dos.extend(grid_layout.to_draw_obj());
    }

    let draw_objs = Canvas::from_objs(dos.into_iter(), /*autobucket=*/ false).with_frame(frame);

    let () = draw_objs
        .write_to_svg(
            Size {
                width: 800,
                height: 1000,
            },
            &args.output_path_prefix,
        )
        .expect("write");
}
