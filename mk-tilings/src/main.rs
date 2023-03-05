use {
    crate::{ab_rhomb::make_ab_rhomb, danzers::make_danzers},
    argh::FromArgs,
    plotz_core::{draw_obj::DrawObjs, frame::make_frame, svg::Size},
    plotz_geometry::point::Pt,
};

mod ab_rhomb;
mod danzers;

#[derive(FromArgs)]
#[argh(description = "...")]
struct Args {
    #[argh(option, description = "output path")]
    output_path_prefix: String,

    #[argh(option, description = "danzers, etc ")]
    pattern: String,
}

fn main() {
    let args: Args = argh::from_env();

    let dos = match args.pattern.as_ref() {
        "danzers" => make_danzers(),
        "ab_rhomb" => make_ab_rhomb(),
        _ => vec![],
    };

    let mut draw_objs = DrawObjs::from_objs(dos).with_frame(make_frame(
        (600.0, 600.0 * 1.4),
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
