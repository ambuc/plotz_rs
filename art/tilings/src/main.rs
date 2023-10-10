use plotz_geometry::shapes::pg2::Pg2;

use argh::FromArgs;
use plotz_core::{canvas::Canvas, frame::make_frame, svg::Size};
use plotz_geometry::{crop::PointLoc, shapes::pt2::Pt2};

mod ab_rhomb;
mod cromwell;
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

    let mut dos = match args.pattern.as_ref() {
        "ab_rhomb" => ab_rhomb::make(),
        "cromwell" => cromwell::make(),
        "danzers" => danzers::make(),
        _ => vec![],
    };

    let frame = make_frame((720.0, 720.0 * 1.3), /*offset=*/ Pt2(20, 20));
    let frame_polygon: Pg2 = frame.0.clone().try_into().unwrap();

    // drain things not in frame
    dos.retain(|(obj2, _style)| {
        obj2.iter()
            .all(|pt| matches!(frame_polygon.contains_pt(pt), PointLoc::Inside))
    });

    let objs = Canvas::from_objs(dos.into_iter(), /*autobucket=*/ false).with_frame(frame);

    //objs.join_adjacent_segments();

    objs.write_to_svg_or_die(
        Size {
            width: (750.0 * 1.3) as usize,
            height: 750,
        },
        &args.output_path_prefix,
    );
}
