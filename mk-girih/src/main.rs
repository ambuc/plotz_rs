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
        shapes::pt2::Pt2,
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

    let mut so2s: Vec<StyledObj2> = strategy1();

    // objs -> mutate
    so2s.iter_mut().for_each(|o| {
        *o *= 100.0;
        *o += Pt2(550.0, 250.0)
    });

    let transformation_pg2 = |x| x * 100.0 + Pt2(500, 300);
    let transformation_sg2 = |x| x * 100.0 + Pt2(500, 300);

    [
        (geom::Girih::Tabl, &RED),
        (geom::Girih::Pange, &ORANGE),
        (geom::Girih::SheshBand, &GREEN),
        (geom::Girih::SormehDan, &BLUE),
        (geom::Girih::Torange, &PURPLE_7),
    ]
    .into_iter()
    .for_each(|(girih_enum, color)| {
        let (mut girih_tile, mut strapwork) = geom::make_girih_tile_and_strapwork(girih_enum);

        // transform tile and strapwork.
        girih_tile = transformation_pg2(girih_tile);
        strapwork
            .iter_mut()
            .for_each(|sg| *sg = transformation_sg2(*sg));

        // shade the tile and write its stripes to |objs|.
        shade_polygon(
            &ShadeConfig::builder().gap(2.0).slope(0.05).build(),
            &girih_tile,
        )
        .unwrap()
        .into_iter()
        .for_each(|sg| {
            so2s.push(StyledObj2::new(sg).with_thickness(0.1).with_color(color));
        });

        // write |tile| itself to |objs|.
        so2s.push(StyledObj2::new(girih_tile).with_style(Style::new(&color, 2.0)));

        // finally, write the strapwork to |objs|.
        strapwork.into_iter().for_each(|sg| {
            so2s.push(StyledObj2::new(sg).with_thickness(1.0).with_color(color));
        });
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
