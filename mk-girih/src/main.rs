use crate::geom::{Constraint, Girih, Tile};

pub mod geom;
mod strategy1;
mod strategy2;

use {
    argh::FromArgs,
    plotz_color::*,
    plotz_core::{canvas::Canvas, frame::make_frame, svg::Size},
    plotz_geometry::{
        p2,
        shading::{shade_config::ShadeConfig, shade_polygon},
        shapes::{pt2::Pt2, sg2::Sg2},
        style::Style,
        styled_obj2::StyledObj2,
    },
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

    // let mut so2s: Vec<StyledObj2> = geom::all_girih_tiles_in_random_order()
    //     .iter()
    //     .map(|g| (*g, g.color()))
    //     .into_iter()
    //     .flat_map(|(girih_enum, color)| {
    //         let t = geom::Tile::new(girih_enum);
    //         let tile = t.to_pg2();
    //         let strapwork = t.to_strapwork();

    //         let stripes =
    //             shade_polygon(&ShadeConfig::builder().gap(0.05).slope(0.05).build(), &tile)
    //                 .unwrap()
    //                 .into_iter()
    //                 .map(|stripe| {
    //                     StyledObj2::new(stripe)
    //                         .with_thickness(0.1)
    //                         .with_color(color)
    //                 });
    //         let outline = StyledObj2::new(tile).with_style(Style::new(&color, 2.0));
    //         let straps = strapwork
    //             .into_iter()
    //             .map(|strap| StyledObj2::new(strap).with_thickness(2.0).with_color(color));
    //         stripes.chain(std::iter::once(outline)).chain(straps)
    //     })
    //     .collect::<Vec<_>>();

    let s2settings = strategy2::Settings { num_iterations: 200 };
    let mut so2s = strategy2::run(&s2settings)
        .map(|mut so2| {
            so2 *= 20.0;
            so2 += Pt2(200, 200);
            so2
        })
        .collect::<Vec<_>>();

    Canvas::from_objs(so2s, /*autobucket=*/ true)
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
