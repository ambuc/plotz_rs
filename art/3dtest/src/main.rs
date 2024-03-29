use anyhow::{Context, Result};
use argh::FromArgs;
use itertools::iproduct;
use lazy_static::lazy_static;
use plotz_color::*;
use plotz_core::{
    canvas::{self, Canvas},
    frame::*,
};
use plotz_geometry::{style::Style, *};
use plotz_geometry3d::{
    group3::Group3,
    obj3::Obj3,
    scene::{
        debug::SceneDebug,
        occluder::{Occluder, OccluderConfig},
        Scene,
    },
    shapes::{cube3d::Cube, cuboid3d::Cuboid, point3::Point3},
    RotatableBounds,
};
use std::{f64::consts::*, iter::zip};
use tracing::*;

lazy_static! {
    static ref GRADIENT: colorgrad::Gradient = colorgrad::rainbow();
}

#[derive(FromArgs)]
#[argh(description = "...")]
struct Args {
    #[argh(option, description = "output path")]
    output_path_prefix: String,
}

#[derive(Debug)]
struct CubesConfig {
    width: f64,
    i: usize,
    j: usize,
    k: usize,
}

#[allow(unused)]
fn cubes(cc: CubesConfig) -> Vec<(Obj3, Style)> {
    let mut objects = vec![];

    for ((i, j, k), color) in zip(
        iproduct!(0..cc.i, 0..cc.j, 0..cc.k),
        (vec![RED, YELLOW, GREEN, BLUE, PLUM, ORANGE])
            .iter()
            .cycle(),
    ) {
        let style = Style {
            color: *color,
            thickness: 2.0,
            ..Default::default()
        };
        objects.extend(
            Cube(Point3(i as f64, j as f64, k as f64), cc.width)
                .into_iter_objects()
                .map(|(o, _)| (o, style)),
        );
    }
    objects
}

#[allow(unused)]
fn scene1() -> Result<Vec<(Obj3, Style)>> {
    Ok(cubes(CubesConfig {
        i: 3,
        j: 3,
        k: 3,
        width: 0.70,
    }))
}

fn scene2() -> Result<impl Iterator<Item = (Obj3, Style)>> {
    // jengas
    let (x_len, y_len, z_len) = (17.0, 5.0, 3.0);
    let (_, y_space, z_space) = ((), 1.0, 3.0);
    let (_, y_num, z_num) = ((), 1, 1);
    let jenga: Group3<()> = Cuboid((0, 0, 0), (x_len, y_len, z_len));

    let layer: Group3<_> = Group3::new(
        (0..=y_num)
            .map(|n| (jenga.clone() + (0, (n as f64) * (y_len + y_space), 0)))
            .flat_map(|o: Group3<_>| o.into_iter_objects()),
    );

    Ok((0..=z_num)
        .map(move |n| {
            (layer.clone() + (0, 0, (n as f64) * (z_len + z_space)))
                .rotate_about_center_z_axis(if n % 2 == 0 { 0.0 } else { FRAC_PI_2 })
                .expect("..")
        })
        .flat_map(|x: Group3<_>| x.into_iter_objects())
        .map(|(o, _): (Obj3, _)| {
            (
                o,
                Style {
                    thickness: 5.0,
                    ..Default::default()
                },
            )
        }))
}

fn main() -> Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .compact()
        .with_max_level(Level::INFO)
        .with_target(true)
        .without_time()
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let _scenedebug = SceneDebug::builder()
        .draw_wireframes(Style {
            color: GRAY,
            thickness: 0.5,
            ..Default::default()
        })
        .annotate(
            AnnotationSettings::builder()
                .font_size(12.0)
                .precision(3)
                .build(),
        )
        .build();

    let args: Args = argh::from_env();
    Canvas::builder()
        .dos_by_bucket(canvas::to_canvas_map(
            Scene::builder()
                // .debug(_scenedebug)
                // .objects(scene1()?)
                .objects(scene2()?.collect())
                .occluder(
                    Occluder::builder()
                        .config(
                            OccluderConfig::builder()
                                .color_according_to_depth(Some(&GRADIENT))
                                .build(),
                        )
                        .build(),
                )
                .build()
                .project()
                .context("default projection with occlusion")?,
            /*autobucket=*/ false,
        ))
        .frame(make_frame_with_margin(
            (800.0, 1150.0),
            /*margin=*/ 25.0,
        )?)
        .build()
        .scale_to_fit_frame()?
        .write_to_svg((1150, 800), &args.output_path_prefix)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use plotz_geometry::obj2::Obj2;
    use test_case::*;

    #[test_matrix(1..=3, 1..=3, 1..=3, 1..=10)]
    fn test_cubes_generation_no_crash(i: usize, j: usize, k: usize, w: usize) {
        if !(i <= j && j <= k) {
            return;
        }
        let width: f64 = (w as f64) / 10.0;
        assert!(0.0 <= width && width <= 1.0);
        let _: Vec<Vec<(Obj2, Style)>> = Scene::builder()
            .objects(cubes(CubesConfig { i, j, k, width }))
            .build()
            .project()
            .into_iter()
            .collect::<Vec<_>>();
    }
}
