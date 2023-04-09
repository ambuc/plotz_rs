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

    let frame: DrawObj = make_frame(
        (1000.0 - 2.0 * mgn, 800.0 - 2.0 * mgn),
        /*offset=*/ Pt(mgn, mgn),
    );

    {
        let mut grid_layout = GridLayout::new(
            GridLayoutSettings::builder()
                .init((25,25))
                .dims((750, 950))
                .divisions((4, 6))
                .object_margin((5, 5))
                .build(),
        );

        let mut rng = thread_rng();
        for i in 0..grid_layout.num_cubbys_x() {
            for j in 0..grid_layout.num_cubbys_y() {
                for color in GREYSCALE.choose_multiple(&mut rng, 2) {
                    let cubby = (i, j);
                    let bounds = grid_layout.get_cubby_bounds(cubby);
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
                        let ca = CurveArc(curve_arc_ctr, 0.0..=TAU, r as f64);
                        let d_o = DrawObj::new(ca).with_thickness(1.0).with_color(color);
                        grid_layout.insert_and_crop_to_cubby(cubby, d_o);
                    }
                }
            }
        }

        dos.extend(grid_layout.to_draw_obj());
    }

    let draw_objs = Canvas::from_objs(dos, /*autobucket=*/ false).with_frame(frame);

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
