pub mod geom;

use {
    argh::FromArgs,
    plotz_color::*,
    plotz_core::{canvas::Canvas, frame::make_frame, svg::Size},
    plotz_geometry::{
        crop::PointLoc,
        interpolate::extrapolate_2d,
        p2,
        shapes::{
            pg2::Pg2,
            pt2::{PolarPt, Pt2},
        },
        styled_obj2::StyledObj2,
    },
    std::{collections::HashSet, f64::consts::*},
    tracing::*,
    tracing_subscriber::FmtSubscriber,
};

#[derive(FromArgs)]
#[argh(description = "...")]
struct Args {
    #[argh(option, description = "output path")]
    output_path_prefix: String,
}

// A placed tile is a tile enum type and its corresponding polygon.
#[derive(Clone, Debug)]
struct PlacedTile {
    girih: geom::Girih,
    pg2: Pg2,
}

// places a tile in the grid.
// accepts an argument |num_to_place| in order to stop descending, eventually.
// returns true if a tile was successfully placed.
fn strategy1(tiles: &mut Vec<PlacedTile>, num_to_place: usize) -> bool {
    fn average(vals: &Vec<f64>) -> f64 {
        vals.iter().sum::<f64>() / (vals.len() as f64)
    }

    // find the corner to attack next.
    fn find_next_pt_to_attack(tiles: &Vec<PlacedTile>) -> Pt2 {
        // is there a single space around this point which isn't already occupied by a tile?
        fn does_space_around_pt_have_empty(tiles: &Vec<PlacedTile>, pt: Pt2) -> bool {
            fn is_point_empty(tiles: &Vec<PlacedTile>, pt: Pt2) -> bool {
                tiles
                    .iter()
                    .any(|placed_tile| match placed_tile.pg2.contains_pt(&pt) {
                        PointLoc::Outside => true,
                        PointLoc::Inside | PointLoc::OnPoint(_) | PointLoc::OnSegment(_) => false,
                    })
            }

            (0..=10)
                .map(|n| (n as f64) * PI / 5.0)
                .map(|angle| pt + PolarPt(0.1, angle))
                .any(|neighbor_pt| is_point_empty(tiles, neighbor_pt))
        }

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

    // base-o case-o
    if num_to_place == 0 {
        return true;
    }

    let pt_to_attack: Pt2 = find_next_pt_to_attack(&tiles);
    println!("pt to attack: {:?}", pt_to_attack);

    for new_girih_type in geom::all_girih_tiles_in_random_order() {
        println!("trying type: {:?}", new_girih_type);
        let (pg2, ks) = geom::make_tile(new_girih_type);

        let mut hs_tried_ks: HashSet<geom::K> = HashSet::<geom::K>::new();
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
                if strategy1(tiles, num_to_place - 1) {
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
    let _num_tiles_placed: usize = 1;
    let (tabl_pg2, _tabl_pointtypes) = geom::make_tile(geom::Girih::Tabl);
    let mut tiles: Vec<PlacedTile> = vec![PlacedTile {
        girih: geom::Girih::Tabl,
        pg2: tabl_pg2,
    }];

    let _success = strategy1(&mut tiles, 3);
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
    //     (geom::Girih::Tabl, &RED),
    //     (geom::Girih::Pange, &ORANGE),
    //     (geom::Girih::SheshBand, &GREEN),
    //     (geom::Girih::SormehDan, &BLUE),
    //     (geom::Girih::Torange, &PURPLE_7),
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
