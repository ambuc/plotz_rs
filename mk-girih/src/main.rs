use float_ord::FloatOrd;
use plotz_color::subway::PURPLE_7;
use plotz_geometry::{
    isxn::{Intersection, IsxnResult},
    shapes::{pg2::Pg2, pt2::PolarPt, sg2::Sg2},
    traits::{Annotatable, AnnotationSettings},
};

use {
    argh::FromArgs,
    plotz_color::*,
    plotz_core::{canvas::Canvas, frame::make_frame, svg::Size},
    plotz_geometry::{
        crop::PointLoc,
        grid::grid_layout::{GridLayout, GridLayoutSettings},
        p2,
        shapes::{curve::CurveArc, pt2::Pt2},
        styled_obj2::StyledObj2,
    },
    rand::{seq::SliceRandom, thread_rng, Rng},
    std::f64::consts::*,
    tracing::*,
    tracing_subscriber::FmtSubscriber,
};

#[derive(FromArgs)]
#[argh(description = "...")]
struct Args {
    #[argh(option, description = "output path")]
    output_path_prefix: String,
}

// girih tiles

// https://en.m.wikipedia.org/wiki/Girih_tiles
//
// The five shapes of the tiles, and their Persian names, are:
//
//
// * All sides of these figures have the same length, and all their angles are
//   multiples of 36° (π/5 radians). All of them except the pentagon have
//   bilateral (reflection) symmetry through two perpendicular lines. Some have
//   additional symmetries. Specifically, the decagon has tenfold rotational
//   symmetry (rotation by 36°); and the pentagon has fivefold rotational
//   symmetry (rotation by 72°).

enum Girih {
    Tabl,
    SheshBand,
    SormehDan,
    Torange,
    Pange,
}

fn make_girih_tile_and_strapwork(g: Girih) -> (Pg2, Vec<Sg2>) {
    let mut tile = make_girih_polygon_from_vertex_turn_angles(match g {
        Girih::Tabl => &[144.0; 10],
        Girih::SheshBand => &[72.0, 144.0, 144.0, 72.0, 144.0, 144.0],
        Girih::SormehDan => &[72.0, 72.0, 216.0, 72.0, 72.0, 216.0],
        Girih::Torange => &[72.0, 108.0, 72.0, 108.0],
        Girih::Pange => &[108.0; 5],
    });
    // NB must offset or vertical line tangents don't work, lmfao
    tile.rotate(&Pt2(0, 0), 0.00001);

    let mut strapwork = vec![];

    let girih_segments = tile.to_segments();
    for (sg_a, sg_b) in girih_segments
        .iter()
        .zip(girih_segments.iter().cycle().skip(1))
    {
        let a_ray_angle = {
            let a_angle = sg_a.i.angle_to(&sg_a.f);

            let angle_1 = (a_angle + (3.0 * PI / 10.0)) % TAU;
            let angle_2 = (a_angle + (-7.0 * PI / 10.0)) % TAU;

            let sg_1_f = sg_a.midpoint() + PolarPt(0.1, angle_1);
            if matches!(tile.contains_pt(&sg_1_f), PointLoc::Inside) {
                angle_1
            } else {
                angle_2
            }
        };

        let a_ray = Sg2(
            sg_a.midpoint(),
            sg_a.midpoint() + PolarPt(10.0, a_ray_angle),
        );

        if let Some(IsxnResult::OneIntersection(_)) = a_ray.intersects(sg_b) {
            strapwork.push(Sg2(sg_a.midpoint(), sg_b.midpoint()));
        } else {
            // imagine a bridge from a_mdpt to b_mdpt.
            // out of the center of the bridge rises a perpendicular tower.
            let tower = {
                let bridge = Sg2(sg_a.midpoint(), sg_b.midpoint());
                let tower_slope = ((bridge.i.angle_to(&bridge.f)) + FRAC_PI_2) % TAU;
                let d = PolarPt(10.0, tower_slope);
                Sg2(bridge.midpoint() - d, bridge.midpoint() + d)
            };
            let bridge_extended = {
                let d = PolarPt(10.0, a_ray_angle);
                Sg2(sg_a.midpoint() - d, sg_a.midpoint() + d)
            };

            // ztex lies at the intersection of a_ray and the tower.
            let ztex = match tower.intersects(&bridge_extended).unwrap() {
                IsxnResult::MultipleIntersections(_) => panic!("multiple intersections?"),
                IsxnResult::OneIntersection(Intersection { pt, .. }) => pt,
            };

            strapwork.push(Sg2(sg_a.midpoint(), ztex));
            strapwork.push(Sg2(ztex, sg_b.midpoint()));
        }
    }

    // columbo voice: one last thing -- some of these strapworks might intersect with each other.
    // if they do, crop them by each other (i.e., if ab intersects cd at x, create ax, xb, cx, xd)
    // and remove the ones with one end outside of the tile.

    let strapwork_verified = {
        let mut s_ver = vec![];

        let tile_contains = |sg: &Sg2| {
            tile.point_is_inside_or_on_border(&sg.i) && tile.point_is_inside_or_on_border(&sg.f)
        };

        for s in strapwork {
            if tile_contains(&s) {
                s_ver.push(s);
            } else {
                let n = 3.0 / 10.0 - 1.0 / (5.0 * PI);
                let a = Sg2(s.i, s.i + (s.f - s.i) * n);
                if tile_contains(&a) {
                    s_ver.push(a);
                }
                let b = Sg2(s.f, s.f + (s.i - s.f) * n);
                if tile_contains(&b) {
                    s_ver.push(b);
                }
            }
        }
        s_ver
    };

    (tile, strapwork_verified)
}

// accepts a list of interior angles, in degrees.
fn make_girih_polygon_from_vertex_turn_angles(vertex_turn_angles: &[f64]) -> Pg2 {
    let mut cursor_position = Pt2(0, 0);
    let mut cursor_angle_rad = 0.0;
    let mut accumulated = vec![cursor_position];
    for vertex_turn_angle in vertex_turn_angles
        .iter()
        .map(|x| (180.0 - x) * PI / 180.0)
        .collect::<Vec<f64>>()
    {
        cursor_angle_rad += vertex_turn_angle;
        cursor_position += PolarPt(1.0, cursor_angle_rad);
        accumulated.push(cursor_position)
    }
    // we are constructing a closed polygon -- so we techincally don't need that
    // very last point, Pg2() automatically closes it for us.
    accumulated.pop();
    Pg2(accumulated)
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

    let mut draw_objects = vec![];
    let margin = 25.0;

    let frame: StyledObj2 = make_frame(
        (1000.0 - 2.0 * margin, 800.0 - 2.0 * margin),
        /*offset=*/ p2!(margin, margin),
    );

    let transformation_pg2 = |x| x * 100.0 + Pt2(500, 300);
    let transformation_sg2 = |x| x * 100.0 + Pt2(500, 300);

    for (girih_enum, color) in [
        (Girih::Tabl, &RED),
        (Girih::Pange, &ORANGE),
        (Girih::SheshBand, &GREEN),
        (Girih::SormehDan, &BLUE),
        (Girih::Torange, &PURPLE_7),
    ] {
        let (girih_tile, strapwork) = make_girih_tile_and_strapwork(girih_enum);
        draw_objects.push(
            StyledObj2::new(transformation_pg2(girih_tile))
                .with_thickness(0.5)
                .with_color(color),
        );
        for sg2 in strapwork {
            draw_objects.push(
                StyledObj2::new(transformation_sg2(sg2))
                    .with_thickness(1.0)
                    .with_color(&BLACK),
            );
        }
    }

    let objs = Canvas::from_objs(draw_objects.into_iter(), /*autobucket=*/ true).with_frame(frame);

    objs.write_to_svg_or_die(
        Size {
            width: 800,
            height: 1000,
        },
        &args.output_path_prefix,
    );
}
