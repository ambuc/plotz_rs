pub mod geom;
mod strategy1;
mod strategy2;

use {
    argh::FromArgs,
    plotz_core::{canvas::Canvas, frame::make_frame, svg::Size},
    plotz_geometry::{p2, shapes::pt2::Pt2},
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

    let margin = 25.0;

    Canvas::from_objs(
        strategy2::run(&strategy2::Settings {
            num_iterations: 200,
            is_deterministic: false,
        })
        .map(|mut so2| {
            so2 *= 20.0;
            so2 += Pt2(300, 400);
            so2
        }),
        /*autobucket=*/
        true,
    )
    .with_frame(make_frame(
        /*wh=*/ (800.0 - 2.0 * margin, 1000.0 - 2.0 * margin),
        /*offset=*/ p2!(margin, margin),
    ))
    .write_to_svg_or_die(
        Size {
            width: 1000,
            height: 800,
        },
        &args.output_path_prefix,
    );
}
