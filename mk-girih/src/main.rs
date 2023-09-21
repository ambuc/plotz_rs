use std::collections::HashSet;

use plotz_geometry::interpolate::{self, extrapolate_2d};

use {
    argh::FromArgs,
    plotz_color::subway::*,
    plotz_color::*,
    plotz_core::{canvas::Canvas, frame::make_frame, svg::Size},
    plotz_geometry::{
        crop::PointLoc,
        isxn::{Intersection, IsxnResult},
        p2,
        shading::{shade_config::ShadeConfig, shade_polygon},
        shapes::{
            pg2::Pg2,
            pt2::{PolarPt, Pt2},
            ry2::Ry2,
            sg2::Sg2,
        },
        style::Style,
        styled_obj2::StyledObj2,
    },
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

// girih tiles https://en.m.wikipedia.org/wiki/Girih_tiles. The five shapes of
// the tiles, and their Persian names, are:
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Girih {
    Tabl,
    SheshBand,
    SormehDan,
    Torange,
    Pange,
}

fn all_girih_tiles() -> Vec<Girih> {
    vec![
        Girih::Tabl,
        Girih::SheshBand,
        Girih::SormehDan,
        Girih::Torange,
        Girih::Pange,
    ]
}

fn all_girih_tiles_in_random_order() -> Vec<Girih> {
    use rand::seq::SliceRandom;
    let mut tiles: Vec<Girih> = all_girih_tiles();
    let mut rng = rand::thread_rng();
    tiles.shuffle(&mut rng);
    tiles
}

fn make_strapwork(g: Girih, tile: &Pg2) -> Vec<Sg2> {
    let mut strapwork = vec![];

    for (edge1, edgeb) in tile
        .to_segments()
        .iter()
        .zip(tile.to_segments().iter().cycle().skip(1))
    {
        let a_ray_angle = {
            let a_angle = edge1.ray_angle();

            let angle_1 = a_angle + (3.0 * PI / 10.0);
            let angle_2 = a_angle + (-7.0 * PI / 10.0);

            let sg_1_f = edge1.midpoint() + PolarPt(0.1, angle_1);
            let sg_2_f = edge1.midpoint() + PolarPt(0.1, angle_2);
            match (tile.contains_pt(&sg_1_f), tile.contains_pt(&sg_2_f)) {
                (PointLoc::Inside, _) => angle_1,
                (_, PointLoc::Inside) => angle_2,
                _ => panic!("oh"),
            }
        };

        let a_ray: Ry2 = Ry2(edge1.midpoint(), a_ray_angle);

        if let Some(IsxnResult::OneIntersection(_)) = a_ray.intersects_sg(edgeb) {
            strapwork.push(Sg2(edge1.midpoint(), edgeb.midpoint()));
        } else {
            // imagine a bridge from a_mdpt to b_mdpt.
            // out of the center of the bridge rise2 a perpendicular tower.
            let bridge = Sg2(edge1.midpoint(), edgeb.midpoint());
            let tower_a = Ry2(bridge.midpoint(), bridge.ray_angle() - FRAC_PI_2);
            let tower_b = Ry2(bridge.midpoint(), bridge.ray_angle() + FRAC_PI_2);

            // ztex lies at the intersection of a_ray and the tower.
            let ztex = match (tower_a.intersects(&a_ray), tower_b.intersects(&a_ray)) {
                (Some(IsxnResult::OneIntersection(Intersection { pt, .. })), _) => pt,
                (_, Some(IsxnResult::OneIntersection(Intersection { pt, .. }))) => pt,
                _ => panic!("oh"),
            };

            strapwork.extend(&[Sg2(edge1.midpoint(), ztex), Sg2(ztex, edgeb.midpoint())]);
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
            match (tile_contains(&s), g) {
                (true, _) => {
                    s_ver.push(s);
                }
                (false, Girih::SormehDan) => {
                    // I just so happen to know that the first segment here runs
                    // perpendicular to a line of symmetry. Don't ask me how I
                    // know it. And don't ask me to generalize it.
                    let (perp_ray_1, perp_ray_2) = tile.to_segments()[0].rays_perpendicular_both();

                    let pt_inside = match (
                        tile.point_is_inside_or_on_border(&s.i),
                        tile.point_is_inside_or_on_border(&s.f),
                    ) {
                        (true, false) => s.i,
                        (false, true) => s.f,
                        _ => panic!("oh"),
                    };

                    match (perp_ray_1.intersects_sg(&s), perp_ray_2.intersects_sg(&s)) {
                        (Some(IsxnResult::OneIntersection(Intersection { pt, .. })), _) => {
                            s_ver.push(Sg2(pt_inside, pt));
                        }
                        (_, Some(IsxnResult::OneIntersection(Intersection { pt, .. }))) => {
                            s_ver.push(Sg2(pt_inside, pt));
                        }
                        _ => panic!("OH"),
                    }
                }
                (false, _) => {
                    panic!("uh oh")
                }
            }
        }
        s_ver
    };

    strapwork_verified
}

// Kind
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
enum K {
    A,
    B,
    C,
}

fn make_tile(g: Girih) -> (Pg2, Vec<K>) {
    let (vertexes, pointtypes) = match g {
        Girih::Tabl => (vec![144.0; 10], vec![K::A; 10]),
        Girih::SheshBand => (
            vec![72.0, 144.0, 144.0, 72.0, 144.0, 144.0],
            vec![K::A, K::B, K::C, K::A, K::B, K::C],
        ),
        Girih::SormehDan => (
            vec![72.0, 72.0, 216.0, 72.0, 72.0, 216.0],
            vec![K::A, K::B, K::C, K::A, K::B, K::C],
        ),
        Girih::Torange => (vec![72.0, 108.0, 72.0, 108.0], vec![K::A, K::B, K::A, K::B]),
        Girih::Pange => (vec![108.0; 5], vec![K::A; 5]),
    };

    let pg2 = make_girih_polygon_from_vertex_turn_angles(&vertexes);
    (pg2, pointtypes)
}

fn make_girih_tile_and_strapwork(g: Girih) -> (Pg2, Vec<Sg2>) {
    let (mut tile, _) = make_tile(g);
    // NB must offset or vertical line tangents don't work, lmfao
    tile.rotate(&Pt2(0, 0), 0.00001);
    let strapwork = make_strapwork(g, &tile);
    (tile, strapwork)
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

// A placed tile is a tile enum type and its corresponding polygon.
#[derive(Clone, Debug)]
struct PlacedTile {
    girih: Girih,
    pg2: Pg2,
}

fn is_point_empty(tiles: &Vec<PlacedTile>, pt: Pt2) -> bool {
    tiles
        .iter()
        .any(|placed_tile| match placed_tile.pg2.contains_pt(&pt) {
            PointLoc::Outside => true,
            PointLoc::Inside | PointLoc::OnPoint(_) | PointLoc::OnSegment(_) => false,
        })
}

// is there a single space around this point which isn't already occupied by a tile?
fn does_space_around_pt_have_empty(tiles: &Vec<PlacedTile>, pt: Pt2) -> bool {
    (0..=10)
        .map(|n| (n as f64) * PI / 5.0)
        .map(|angle| pt + PolarPt(0.1, angle))
        .any(|neighbor_pt| is_point_empty(tiles, neighbor_pt))
}

// find the corner to attack next.
fn find_next_pt_to_attack(tiles: &Vec<PlacedTile>) -> Pt2 {
    let last_placed_tile = tiles.last().unwrap();
    dbg!(last_placed_tile);
    let pt_to_attack: Pt2 = last_placed_tile
        .pg2
        .pts
        .iter()
        .find(|pt| does_space_around_pt_have_empty(&tiles, **pt))
        .copied()
        .expect("this was the last placed tile, there should be a point to attack.");
    pt_to_attack
}

fn average(vals: &Vec<f64>) -> f64 {
    vals.iter().sum::<f64>() / (vals.len() as f64)
}

// places a tile in the grid.
// accepts an argument |num_to_place| in order to stop descending, eventually.
// returns true if a tile was successfully placed.
fn place_tile(tiles: &mut Vec<PlacedTile>, num_to_place: usize) -> bool {
    // base-o case-o
    if num_to_place == 0 {
        return true;
    }

    let pt_to_attack: Pt2 = find_next_pt_to_attack(&tiles);
    println!("pt to attack: {:?}", pt_to_attack);

    for new_girih_type in all_girih_tiles_in_random_order() {
        println!("trying type: {:?}", new_girih_type);
        let (pg2, ks) = make_tile(new_girih_type);

        let mut hs_tried_ks: HashSet<K> = HashSet::<K>::new();
        for (pg_pt, k) in pg2.pts.iter().zip(ks.iter()) {
            // println!("trying pg_pt: {:?} @ {:?}", pg_pt, k);
            // first, check for k type
            if hs_tried_ks.contains(k) {
                continue;
            }
            hs_tried_ks.insert(*k);

            // shift
            let delta: Pt2 = *pg_pt - pt_to_attack;
            let pg_copy: Pg2 = pg2.clone() + delta;
            // for each rotation
            for rot_n in 0..10 {
                // println!("trying rot {:?}", rot_n);
                // rotate
                let rot = (rot_n as f64) * PI / 10.0;
                let mut cand = pg_copy.clone();
                cand.rotate(pg_pt, rot);
                // detect collisions

                let cand_ctr = Pt2(
                    average(&cand.pts.iter().map(|p| p.x.0).collect::<Vec<_>>()),
                    average(&cand.pts.iter().map(|p| p.y.0).collect::<Vec<_>>()),
                );

                if !tiles.iter().all(|extant_tile| {
                    matches!(extant_tile.pg2.contains_pt(&cand_ctr), PointLoc::Outside)
                }) {
                    // rejecting candidate because its ctr was inside a tile.
                    continue;
                }

                // no points of the candidate must be fully inside any tile.
                if cand.pts.iter().any(|cand_pt| {
                    tiles
                        .iter()
                        .any(|extant_tile| extant_tile.pg2.point_is_inside(cand_pt))
                }) {
                    // rejecting candidate because one of its pts was inside a tile.
                    continue;
                }

                if cand.to_segments().iter().any(|cand_sg| {
                    (0..10).any(|f| {
                        let p: f64 = (f as f64) / 10.0;
                        let pt = extrapolate_2d(cand_sg.i, cand_sg.f, p);
                        tiles
                            .iter()
                            .any(|extant_tile| extant_tile.pg2.point_is_inside(&pt))
                    })
                }) {
                    // rejecting candidate because a point partway along it was inside another tile.
                    continue;
                }

                // if there is a single extant tile for which all points are fully inside the candidate,
                if tiles.iter().any(|extant_tile| {
                    extant_tile
                        .pg2
                        .pts
                        .iter()
                        .all(|extant_pt| cand.point_is_inside_or_on_border(extant_pt))
                }) {
                    // reject
                    continue;
                }

                // if all of the candidate points are within any tile
                if cand.pts.iter().all(|cand_pt| {
                    tiles
                        .iter()
                        .any(|extant_tile| extant_tile.pg2.point_is_inside_or_on_border(cand_pt))
                }) {
                    continue;
                }

                // else seems ok.

                tiles.push(PlacedTile {
                    girih: new_girih_type,
                    pg2: cand.clone(),
                });
                if place_tile(tiles, num_to_place - 1) {
                    return true;
                } else {
                    tiles.pop();
                    continue;
                }
            }
        }
    }
    return false;
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

    let mut objs = vec![];
    let margin = 25.0;

    let frame: StyledObj2 = make_frame(
        (800.0 - 2.0 * margin, 1000.0 - 2.0 * margin),
        /*offset=*/ p2!(margin, margin),
    );

    // loop:
    //    -lay down one tile.
    //    -go to each corner of the tile:
    //    -for each corner -- while (the area around the corner isn't 100% full):
    //         -pick a random tile (if any remain untried)
    //             -take a random corner of the tile and align it
    //             -rotate the tile about that corner in increments of pi/5 until it doesn't
    //              intersect with any already-placed tiles
    //             -if this works, place it and descend
    //    - if nothing works, backtrack

    // first, lay down one tile.
    let mut num_tiles_placed: usize = 1;
    let (tabl_pg2, tabl_pointtypes) = make_tile(Girih::Tabl);
    let mut tiles: Vec<PlacedTile> = vec![PlacedTile {
        girih: Girih::Tabl,
        pg2: tabl_pg2,
    }];

    let success = place_tile(&mut tiles, 2);

    // assert!(success);

    // tiles -> objs
    tiles.into_iter().for_each(|placed_tile| {
        objs.push(
            StyledObj2::new(placed_tile.pg2)
                .with_thickness(1.0)
                .with_color(&RED),
        );
    });

    // objs -> mutate
    objs.iter_mut().for_each(|o| {
        *o *= 100.0;
        *o += Pt2(550.0, 250.0)
    });

    // let transformation_pg2 = |x| x * 100.0 + Pt2(500, 300);
    // let transformation_sg2 = |x| x * 100.0 + Pt2(500, 300);

    // for (girih_enum, color) in [
    //     (Girih::Tabl, &RED),
    //     (Girih::Pange, &ORANGE),
    //     (Girih::SheshBand, &GREEN),
    //     (Girih::SormehDan, &BLUE),
    //     (Girih::Torange, &PURPLE_7),
    // ] {
    //     let (mut girih_tile, mut strapwork) = make_girih_tile_and_strapwork(girih_enum);

    //     // transform tile and strapwork.
    //     girih_tile = transformation_pg2(girih_tile);
    //     strapwork
    //         .iter_mut()
    //         .for_each(|sg| *sg = transformation_sg2(*sg));

    //     // shade the tile and write its stripes to |objs|.
    //     shade_polygon(
    //         &ShadeConfig::builder().gap(2.0).slope(0.05).build(),
    //         &girih_tile,
    //     )
    //     .unwrap()
    //     .into_iter()
    //     .for_each(|sg| {
    //         objs.push(StyledObj2::new(sg).with_thickness(0.1).with_color(color));
    //     });

    //     // write |tile| itself to |objs|.
    //     objs.push(StyledObj2::new(girih_tile).with_style(Style::new(&color, 2.0)));

    //     // finally, write the strapwork to |objs|.
    //     strapwork.into_iter().for_each(|sg| {
    //         objs.push(StyledObj2::new(sg).with_thickness(1.0).with_color(color));
    //     });
    // }

    let objs = Canvas::from_objs(objs.into_iter(), /*autobucket=*/ true).with_frame(frame);

    objs.write_to_svg_or_die(
        Size {
            width: 1000,
            height: 800,
        },
        &args.output_path_prefix,
    );
}
