//! The point of entry for plotz. Call this executable to parse geojson to svg.

#![deny(missing_docs)]

use anyhow::Result;
use argh::FromArgs;
use glob::glob;
use plotz_core::{
    map::{Map, MapConfig},
    svg::Size,
};
use plotz_geometry::shapes::point::Point;

#[derive(FromArgs, Debug)]
#[argh(description = "...")]
struct Args {
    #[argh(option, description = "all geojson")]
    input_glob: String,
    #[argh(option, description = "output file prefix")]
    output_directory: std::path::PathBuf,
    #[argh(option, description = "width")]
    width: usize,
    #[argh(option, description = "height")]
    height: usize,
    #[argh(switch, description = "draw frame")]
    draw_frame: bool,
    #[argh(option, description = "scale factor", default = "0.9")]
    scale_factor: f64,

    #[argh(option, description = "center lat")]
    center_lat: Option<f64>,
    #[argh(option, description = "center lng")]
    center_lng: Option<f64>,
}

fn main() -> Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .compact()
        .with_max_level(tracing::Level::TRACE)
        .without_time()
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let args: Args = argh::from_env();
    main_inner(args)?;
    Ok(())
}

fn main_inner(args: Args) -> Result<()> {
    let map_config = MapConfig::builder()
        .input_files(glob(&args.input_glob)?.collect::<Result<Vec<_>, _>>()?)
        .output_directory(args.output_directory)
        .size(Size {
            width: args.width,
            height: args.height,
        })
        .draw_frame(args.draw_frame)
        .scale_factor(args.scale_factor)
        .build();

    let map = Map::new(
        &map_config,
        match (args.center_lat, args.center_lng) {
            (Some(x), Some(y)) => Some(Point(y, x)),
            _ => None,
        },
    )?;

    map.render(&map_config)?;

    Ok(())
}
