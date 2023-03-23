use {
    argh::FromArgs,
    plotz_core::{
        draw_obj::DrawObj, draw_obj_inner::DrawObjInner, draw_objs::DrawObjs, frame::make_frame,
        svg::Size,
    },
    plotz_geometry::{point::Pt, polygon::PointLoc, traits::YieldPoints},
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

    let frame: DrawObj = make_frame((720.0, 720.0 * 1.3), /*offset=*/ Pt(20.0, 20.0));
    let frame_polygon = match frame.obj {
        DrawObjInner::Polygon(ref pg) => pg.clone(),
        _ => unimplemented!(),
    };

    // drain things not in frame
    dos.retain(|d_o| {
        if let Some(yp) = d_o.yield_pts() {
            yp.into_iter()
                .all(|pt| matches!(frame_polygon.contains_pt(pt), Ok(PointLoc::Inside)))
        } else {
            true
        }
    });

    let draw_objs = DrawObjs::from_objs(dos).with_frame(frame);

    //draw_objs.join_adjacent_segments();

    let () = draw_objs
        .write_to_svg(
            Size {
                width: (750.0 * 1.3) as usize,
                height: 750,
            },
            &args.output_path_prefix,
        )
        .expect("write");
}
