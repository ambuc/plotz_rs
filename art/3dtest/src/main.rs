use anyhow::{Context, Result};
use argh::FromArgs;
use itertools::iproduct;
use plotz_color::*;
use plotz_core::{canvas::Canvas, frame::*};
use plotz_geometry::{style::Style, *};
use plotz_geometry3d::{
    camera::{Occlusion, Projection},
    group3::Group3,
    obj3::Obj3,
    scene::{debug::SceneDebug, Scene},
    shapes::{cube3d::Cube, cuboid3d::Cuboid, pt3::Pt3},
    RotatableBounds,
};
use std::{f64::consts::FRAC_PI_2, iter::zip};
use tracing::*;

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
        (vec![&RED, &YELLOW, &GREEN, &BLUE, &PLUM, &ORANGE])
            .iter()
            .cycle(),
    ) {
        let style = Style {
            color,
            thickness: 2.0,
            ..Default::default()
        };
        objects.extend(
            Cube(Pt3(i as f64, j as f64, k as f64), cc.width)
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

fn scene2() -> Result<Vec<(Obj3, Style)>> {
    // jengas
    let jenga: Group3<()> = Cuboid((0, 0, 0), (15, 5, 3));

    let mut i: Vec<_> = vec![];
    i.extend((jenga.clone() + Pt3(0, 0, 0)).into_iter_objects());
    i.extend((jenga.clone() + Pt3(0, 6, 0)).into_iter_objects());
    i.extend((jenga.clone() + Pt3(0, 12, 0)).into_iter_objects());
    let layer: Group3<()> = Group3::<()>::new(i);

    let mut g: Vec<Group3<()>> = vec![];
    g.push(layer.clone());
    g.push((layer.clone() + Pt3(0, 0, 4)).rotate_about_center_z_axis(FRAC_PI_2)?);

    Ok(g.into_iter()
        .flat_map(|x| x.into_iter_objects().map(|(o, _)| (o, Style::default())))
        .collect())
}

fn main() -> Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .compact()
        .with_max_level(Level::INFO)
        .without_time()
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let _annotation = AnnotationSettings::builder()
        .font_size(12.0)
        .precision(3)
        .build();
    let _scenedebug = SceneDebug::builder()
        .draw_wireframes(Style {
            color: &GRAY,
            thickness: 0.5,
            ..Default::default()
        })
        .annotate(_annotation)
        .build();

    let args: Args = argh::from_env();
    Canvas::from_objs(
        /*objs=*/
        Scene::builder()
            // .debug(_scenedebug)
            // .objects(scene1()?)
            .objects(scene2()?)
            .build()
            .project_with(Projection::default(), Occlusion::True)
            .context("default projection with occlusion")?
            .into_iter(),
        /*autobucket=*/ false,
    )
    .with_frame(make_frame_with_margin(
        (800.0, 600.0),
        /*margin=*/ 25.0,
    ))
    .scale_to_fit_frame()?
    .write_to_svg((600, 800), &args.output_path_prefix)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use plotz_geometry::obj::Obj;
    use test_case::*;

    #[test_matrix(1..=3, 1..=3, 1..=3, 1..=10)]
    fn test_cubes_generation_no_crash(i: usize, j: usize, k: usize, w: usize) {
        if !(i <= j && j <= k) {
            return;
        }
        let width: f64 = (w as f64) / 10.0;
        assert!(0.0 <= width && width <= 1.0);
        let _: Vec<Vec<(Obj, Style)>> = Scene::builder()
            .objects(cubes(CubesConfig { i, j, k, width }))
            .build()
            .project_with(Projection::default(), Occlusion::True)
            .into_iter()
            .collect::<Vec<_>>();
    }
}
