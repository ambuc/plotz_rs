use {
    argh::FromArgs,
    plotz_core::{canvas::Canvas, frame::make_frame, svg::Size},
    plotz_geometry::{
        crop::PointLoc, object2d::Object2d, object2d_inner::Object2dInner, shapes::point::Pt,
        traits::YieldPoints,
    },
};

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

    let frame: Object2d = make_frame((720.0, 720.0 * 1.3), /*offset=*/ Pt(20.0, 20.0));
    let frame_polygon = match frame.inner {
        Object2dInner::Polygon(ref pg) => pg.clone(),
        _ => unimplemented!(),
    };

    // drain things not in frame
    dos.retain(|d_o| {
        if let Some(yp) = d_o.yield_pts() {
            yp.into_iter()
                .all(|pt| matches!(frame_polygon.contains_pt(pt), PointLoc::Inside))
        } else {
            true
        }
    });

    let objs = Canvas::from_objs(dos.into_iter(), /*autobucket=*/ false).with_frame(frame);

    //objs.join_adjacent_segments();

    let () = objs
        .write_to_svg(
            Size {
                width: (750.0 * 1.3) as usize,
                height: 750,
            },
            &args.output_path_prefix,
        )
        .expect("write");
}
