#![allow(unused)]

// use {
//     crate::geom,
//     plotz_color::*,
//     plotz_geometry::{
//         crop::PointLoc,
//         interpolate::extrapolate_2d,
//         shapes::{
//             pg2::Pg2,
//             pt2::{PolarPt, Pt2},
//         },
//         styled_obj2::StyledObj2,
//     },
//     std::{collections::HashSet, f64::consts::*},
// };
// 
// // A placed tile is a tile enum type and its corresponding polygon.
// #[derive(Clone, Debug)]
// struct PlacedTile {
//     girih: geom::Girih,
//     pg2: Pg2,
// }
// 
// // places a tile in the grid.
// // accepts an argument |num_to_place| in order to stop descending, eventually.
// // returns true if a tile was successfully placed.
// fn strategy1_inner(tiles: &mut Vec<PlacedTile>, num_to_place: usize) -> bool {
//     fn average(vals: &Vec<f64>) -> f64 {
//         vals.iter().sum::<f64>() / (vals.len() as f64)
//     }
// 
//     // find the corner to attack next.
//     fn find_next_pt_to_attack(tiles: &Vec<PlacedTile>) -> Pt2 {
//         // is there a single space around this point which isn't already occupied by a tile?
//         fn does_space_around_pt_have_empty(tiles: &Vec<PlacedTile>, pt: Pt2) -> bool {
//             fn is_point_empty(tiles: &Vec<PlacedTile>, pt: Pt2) -> bool {
//                 tiles
//                     .iter()
//                     .any(|placed_tile| match placed_tile.pg2.contains_pt(&pt) {
//                         PointLoc::Outside => true,
//                         PointLoc::Inside | PointLoc::OnPoint(_) | PointLoc::OnSegment(_) => false,
//                     })
//             }
// 
//             (0..=10)
//                 .map(|n| (n as f64) * PI / 5.0)
//                 .map(|angle| pt + PolarPt(0.1, angle))
//                 .any(|neighbor_pt| is_point_empty(tiles, neighbor_pt))
//         }
// 
//         let last_placed_tile = tiles.last().unwrap();
//         let pt_to_attack: Pt2 = last_placed_tile
//             .pg2
//             .pts
//             .iter()
//             .find(|pt| does_space_around_pt_have_empty(&tiles, **pt))
//             .copied()
//             .expect("this was the last placed tile, there should be a point to attack.");
//         pt_to_attack
//     }
// 
//     // base-o case-o
//     if num_to_place == 0 {
//         return true;
//     }
// 
//     let pt_to_attack: Pt2 = find_next_pt_to_attack(&tiles);
// 
//     for new_girih_type in geom::all_girih_tiles_in_random_order() {
//         let (pg2, ks) = geom::make_tile(new_girih_type);
// 
//         let mut hs_tried_ks: HashSet<geom::K> = HashSet::<geom::K>::new();
//         for (pg_pt, k) in pg2.pts.iter().zip(ks.iter()) {
//             // first, check for k type
//             if hs_tried_ks.contains(k) {
//                 continue;
//             }
//             hs_tried_ks.insert(*k);
// 
//             // shift
//             let delta: Pt2 = *pg_pt - pt_to_attack;
//             let pg_copy: Pg2 = pg2.clone() + delta;
//             // for each rotation
//             for rot_n in 0..10 {
//                 // rotate
//                 let rot = (rot_n as f64) * PI / 10.0;
//                 let mut cand = pg_copy.clone();
//                 cand.rotate(pg_pt, rot);
//                 // detect collisions
// 
//                 let cand_ctr = Pt2(
//                     average(&cand.pts.iter().map(|p| p.x.0).collect::<Vec<_>>()),
//                     average(&cand.pts.iter().map(|p| p.y.0).collect::<Vec<_>>()),
//                 );
// 
//                 if !tiles.iter().all(|extant_tile| {
//                     matches!(extant_tile.pg2.contains_pt(&cand_ctr), PointLoc::Outside)
//                 }) {
//                     // rejecting candidate because its ctr was inside a tile.
//                     continue;
//                 }
// 
//                 // no points of the candidate must be fully inside any tile.
//                 if cand.pts.iter().any(|cand_pt| {
//                     tiles
//                         .iter()
//                         .any(|extant_tile| extant_tile.pg2.point_is_inside(cand_pt))
//                 }) {
//                     // rejecting candidate because one of its pts was inside a tile.
//                     continue;
//                 }
// 
//                 if cand.to_segments().iter().any(|cand_sg| {
//                     (0..10).any(|f| {
//                         let p: f64 = (f as f64) / 10.0;
//                         let pt = extrapolate_2d(cand_sg.i, cand_sg.f, p);
//                         tiles
//                             .iter()
//                             .any(|extant_tile| extant_tile.pg2.point_is_inside(&pt))
//                     })
//                 }) {
//                     // rejecting candidate because a point partway along it was inside another tile.
//                     continue;
//                 }
// 
//                 // if there is a single extant tile for which all points are fully inside the candidate,
//                 if tiles.iter().any(|extant_tile| {
//                     extant_tile
//                         .pg2
//                         .pts
//                         .iter()
//                         .all(|extant_pt| cand.point_is_inside_or_on_border(extant_pt))
//                 }) {
//                     // reject
//                     continue;
//                 }
// 
//                 // if all of the candidate points are within any tile
//                 if cand.pts.iter().all(|cand_pt| {
//                     tiles
//                         .iter()
//                         .any(|extant_tile| extant_tile.pg2.point_is_inside_or_on_border(cand_pt))
//                 }) {
//                     continue;
//                 }
// 
//                 // else seems ok.
// 
//                 tiles.push(PlacedTile {
//                     girih: new_girih_type,
//                     pg2: cand.clone(),
//                 });
//                 if strategy1_inner(tiles, num_to_place - 1) {
//                     return true;
//                 } else {
//                     tiles.pop();
//                     continue;
//                 }
//             }
//         }
//     }
//     return false;
// }
// 
// pub fn strategy1() -> Vec<StyledObj2> {
//     let mut so2s = vec![];
//     // first, lay down one tile.
//     let _num_tiles_placed: usize = 1;
//     let (tabl_pg2, _tabl_pointtypes) = geom::make_tile(geom::Girih::Tabl);
//     let mut tiles: Vec<PlacedTile> = vec![PlacedTile {
//         girih: geom::Girih::Tabl,
//         pg2: tabl_pg2,
//     }];
// 
//     let _success = strategy1_inner(&mut tiles, 3);
//     // assert!(success);
// 
//     // tiles -> objs
//     tiles.into_iter().for_each(|placed_tile| {
//         so2s.push(
//             StyledObj2::new(placed_tile.pg2)
//                 .with_thickness(1.0)
//                 .with_color(&RED),
//         );
//     });
// 
//     so2s
// }
