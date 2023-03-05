use {
    crate::danzers::make_danzers,
    argh::FromArgs,
    plotz_core::{draw_obj::DrawObjs, frame::make_frame, svg::Size},
    plotz_geometry::point::Pt,
};

mod danzers;

static DIM: f64 = 600.0;

#[derive(FromArgs)]
#[argh(description = "...")]
struct Args {
    #[argh(option, description = "output path")]
    output_path_prefix: String,
}

fn main() {
    let args: Args = argh::from_env();

    let dos = make_danzers();

    let mut draw_objs = DrawObjs::from_objs(dos).with_frame(make_frame(
        (DIM, DIM * 1.4),
        /*offset=*/ Pt(10.0, 10.0),
    ));

    draw_objs.join_adjacent_segments();

    let () = draw_objs
        .write_to_svg(
            Size {
                width: (750.0 * 1.4) as usize,
                height: 750,
            },
            &args.output_path_prefix,
        )
        .expect("write");
}
