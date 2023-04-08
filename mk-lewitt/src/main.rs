use rand::{thread_rng, Rng};

use {
    argh::FromArgs,
    plotz_core::{canvas::Canvas, frame::make_frame, svg::Size},
    plotz_geometry::{
        curve::CurveArc,
        draw_obj::DrawObj,
        grid_layout::{GridLayout, GridLayoutSettings},
        point::Pt,
    },
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
                .x_init(25)
                .y_init(25)
                .total_width(750)
                .total_height(950)
                .x_divisions(4)
                .y_divisions(6)
                .object_margin_x(2)
                .object_margin_y(2)
                .build(),
        );

        let mut rng = thread_rng();
        for i in 0..grid_layout.num_cubbys_x() {
            for j in 0..grid_layout.num_cubbys_y() {
                for _ in 0..2 {
                    let cubby = (i, j);
                    let ctr = Pt(rng.gen_range(0.0..800.0), rng.gen_range(0.0..1000.0));
                    let rstep = rng.gen_range(6..20);
                    for r in (0..2000).step_by(rstep) {
                        let ca = CurveArc(ctr, 0.0..=TAU, r as f64);
                        let d_o = DrawObj::new(ca).with_thickness(1.0);
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
