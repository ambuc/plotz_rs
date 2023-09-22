use plotz_geometry::{grid, group::Group, shapes::pg2::Rect};

use crate::strategy1::strategy1;

pub mod geom;
mod strategy1;

use {
    argh::FromArgs,
    plotz_color::{subway::*, *},
    plotz_core::{canvas::Canvas, frame::make_frame, svg::Size},
    plotz_geometry::{
        p2,
        shading::{shade_config::ShadeConfig, shade_polygon},
        shapes::{pg2::Pg2, pt2::Pt2},
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

    let mut so2s: Vec<StyledObj2> = [
        (geom::Girih::Tabl, &RED),
        (geom::Girih::Pange, &ORANGE),
        (geom::Girih::SheshBand, &GREEN),
        (geom::Girih::SormehDan, &BLUE),
        (geom::Girih::Torange, &PURPLE_7),
    ]
    .into_iter()
    .flat_map(|(girih_enum, color)| {
        let (mut tile, mut strapwork) = geom::make_girih_tile_and_strapwork(girih_enum);

        let shade = ShadeConfig::builder().gap(0.05).slope(0.05).build();
        shade_polygon(&shade, &tile)
            .unwrap()
            .into_iter()
            .map(|stripe| {
                StyledObj2::new(stripe)
                    .with_thickness(0.1)
                    .with_color(color)
            })
            .chain(std::iter::once(
                StyledObj2::new(tile).with_style(Style::new(&color, 2.0)),
            ))
            .chain(
                strapwork
                    .into_iter()
                    .map(|strap| StyledObj2::new(strap).with_thickness(1.0).with_color(color)),
            )
    })
    .collect::<Vec<_>>();

    so2s.iter_mut().for_each(|so2| {
        *so2 *= 100.0;
        *so2 += Pt2(500.0, 100.0);
    });

    Canvas::from_objs(so2s.into_iter(), /*autobucket=*/ true)
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
