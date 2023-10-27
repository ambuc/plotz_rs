pub mod geom;
mod layout;
mod strategy1;
mod strategy2;
mod strategy3;

use anyhow::Result;
use argh::FromArgs;
use plotz_core::{
    canvas::{self, Canvas},
    frame::make_frame,
    svg::Size,
};
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

    let margin = 25.0;

    Canvas::builder()
        .dos_by_bucket(canvas::to_canvas_map(
            strategy3::run()?
                // strategy2::run()
                .into_iter()
                .map(|(mut obj, style)| {
                    obj *= 40.0;
                    obj += (500, 400);
                    (obj, style)
                })
                .map(|so2| (so2.0, so2.1)),
            /*autobucket=*/
            true,
        ))
        .frame(make_frame(
            /*wh=*/ (800.0 - 2.0 * margin, 1000.0 - 2.0 * margin),
            /*offset=*/ (margin, margin),
        ))
        .build()
        .write_to_svg(
            Size {
                width: 1000,
                height: 800,
            },
            &args.output_path_prefix,
        )?;
    Ok(())
}
