use anyhow::Result;
use argh::FromArgs;
use plotz_core::{
    canvas::{self, Canvas},
    frame::make_frame,
    svg::Size,
};
use plotz_geometry::{crop::PointLoc, obj::Obj, shapes::pg::Pg, style::Style, Object};

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

fn main() -> Result<()> {
    let args: Args = argh::from_env();

    let mut dos: Vec<(Obj, Style)> = match args.pattern.as_ref() {
        "ab_rhomb" => ab_rhomb::make(),
        "cromwell" => cromwell::make(),
        "danzers" => danzers::make(),
        _ => vec![],
    };

    let frame = make_frame((720.0, 720.0 * 1.3), /*offset=*/ (20, 20))?;
    let frame_polygon: Pg = frame.0.clone().try_into().unwrap();

    // drain things not in frame
    dos.retain(|(obj, _style)| {
        obj.iter()
            .all(|pt| matches!(frame_polygon.contains_pt(pt), Ok(PointLoc::Inside)))
    });

    Canvas::builder()
        .dos_by_bucket(canvas::to_canvas_map(dos, /*autobucket=*/ false))
        .frame(frame)
        .build()
        .write_to_svg(
            Size {
                width: (750.0 * 1.3) as usize,
                height: 750,
            },
            &args.output_path_prefix,
        )?;
    Ok(())
}
