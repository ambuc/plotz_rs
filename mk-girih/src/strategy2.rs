use crate::geom::*;
use average::Mean;
use plotz_geometry::{
    bounded::Bounded,
    shapes::{pt2::Pt2, sg2::Sg2},
    styled_obj2::StyledObj2,
};
use rand::{seq::SliceRandom, Rng};
use std::f64::consts::{PI, TAU};
use tracing::info;

#[derive(Debug)]
pub struct Settings {
    pub num_iterations: usize,
    pub is_deterministic: bool,
}

impl Settings {
    pub fn choices(&self) -> Vec<Girih> {
        match self.is_deterministic {
            true => all_girih_tiles(),
            false => {
                let mut weighted_choices = vec![
                    (Girih::SormehDan, 1),
                    (Girih::Tabl, 1),
                    (Girih::Pange, 1),
                    (Girih::Torange, 1),
                    (Girih::SheshBand, 1),
                ];
                let mut dest = vec![];
                let mut rng = rand::thread_rng();
                while !weighted_choices.is_empty() {
                    let marble = weighted_choices
                        .choose_weighted(&mut rng, |(item, weight)| *weight)
                        .unwrap()
                        .0;
                    weighted_choices.remove(
                        weighted_choices
                            .iter()
                            .position(|(item, weight)| *item == marble)
                            .unwrap(),
                    );
                    dest.push(marble);
                }
                dest
            }
        }
    }
}

struct Layout {
    placed_tiles: Vec<PlacedTile>,
}
impl Layout {
    fn new(pt: PlacedTile) -> Layout {
        Layout {
            placed_tiles: vec![pt],
        }
    }

    fn to_styledobjs(self) -> impl Iterator<Item = StyledObj2> {
        self.placed_tiles
            .into_iter()
            .flat_map(|pt| pt.to_styledobjs())
    }

    fn next_bare_edge(&self) -> Sg2 {
        // what if we WEIGHTED these by proximity to average center?
        // let ctrs: Vec<Pt2> = self
        //     .placed_tiles
        //     .iter()
        //     .map(|placed_tile| placed_tile.pg2.bbox_center())
        //     .collect::<Vec<_>>();

        // let mean_x: Mean = ctrs.iter().map(|pt2| pt2.x.0).collect();
        // let mean_y: Mean = ctrs.iter().map(|pt2| pt2.y.0).collect();
        // let ctr: Pt2 = Pt2(mean_x.mean(), mean_y.mean());
        let ctr: Pt2 = Pt2(0, 0);

        let mut bare_edges = vec![];
        for placed_tile in &self.placed_tiles {
            for segment in placed_tile.pg2.to_segments() {
                // both rays which emit from the midpoint.
                let (ray_a, ray_b) = segment.rays_perpendicular_both();
                let offset = segment.abs() * 0.1;
                // if there is any point adjacent to the segment (a tiny offset away)
                for pt in [ray_a.to_sg2(offset).f, ray_b.to_sg2(offset).f] {
                    // for which it is outside of _ALL_ known placed tiles
                    if self
                        .placed_tiles
                        .iter()
                        .all(|t| t.pg2.point_is_outside(&pt))
                    {
                        bare_edges.push(segment);
                    }
                }
            }
        }

        bare_edges
            .into_iter()
            .min_by_key(|sg| float_ord::FloatOrd(sg.midpoint().dist(&ctr)))
            .expect("bare_edges should never be empty")
    }

    fn place_tile_on_edge_src(
        &self,
        g: Girih,
        target: &Sg2,
        src_index: usize,
    ) -> Option<PlacedTile> {
        let cand: PlacedTile = Tile::new(g).clone().place(Constraint {
            src_index,
            target: *target,
        });
        if self.evaluate_cand(&cand) {
            return Some(cand);
        }
        None
    }

    fn evaluate_cand(&self, cand: &PlacedTile) -> bool {
        let cand_ctr = cand.pg2.bbox_center();
        let test_pts: Vec<Pt2> = std::iter::once(cand.pg2.bbox_center())
            .chain(
                cand.pg2
                    .to_segments()
                    .iter()
                    .map(|sg2| -> Pt2 { sg2.midpoint() }),
            )
            .chain(cand.pg2.pts.iter().map(|pt| pt.avg(&cand_ctr)))
            .collect::<Vec<_>>();

        use itertools::Itertools;
        use rayon::prelude::*;
        if (self.placed_tiles.iter())
            .cartesian_product(test_pts.iter())
            .any(|(extant_tile, test_pt)| extant_tile.pg2.point_is_inside(&test_pt))
        {
            return false;
        }

        // not having collisions is very important. but there is another
        // important characteristic as well. we want to make sure that, around
        // each corner of the newly placed tile, we haven't created a tight
        // corner -- an acute angle of size pi/5 (for example) which no tile
        // could fill.

        // if there's _any_ collision, return false;
        if cand.pg2.to_segments().par_iter().any(|cand_sg| -> bool {
            // returns true if there's a collision
            let mut results: Vec<bool> = vec![];
            let mut rotor = Sg2(cand_sg.i, cand_sg.midpoint());
            rotor.rotate(&cand_sg.i, 0.001 * TAU); // offset
            for _ in 0..=10 {
                // ten times, rotate the rotor by TAU/10 (or, (2PI)/10)
                let axis = rotor.i;
                rotor.rotate(&axis, 1.0 / 11.0 * TAU);

                let trial_pt = rotor.f;
                results.push(
                    cand.pg2.point_is_inside(&trial_pt)
                        || self
                            .placed_tiles
                            .iter()
                            .any(|extant_tile| extant_tile.pg2.point_is_inside(&trial_pt)),
                );
            }
            if results
                .iter()
                .cycle()
                .take(11)
                .collect::<Vec<_>>()
                .windows(3)
                .any(|window| matches!(window, [true, false, true]))
            {
                return true;
            }
            return false;
        }) {
            // if there's any collision, return false.
            return false;
        }

        return true;
    }

    // returns true if successfully placed tile (or if no tile needed to be placed.)
    fn place_next_tile(&mut self, settings: &Settings, num_remaining: usize) -> bool {
        info!("place_next_tile: {:?}", num_remaining);
        if num_remaining == 0 {
            return true;
        }

        let next_bare_edge: Sg2 = self.next_bare_edge();

        for g in settings.choices() {
            for target in [next_bare_edge.flip(), next_bare_edge] {
                for src_index in 0..Tile::new(g).to_naive_pg2().pts.len() {
                    if let Some(placed_tile) = self.place_tile_on_edge_src(g, &target, src_index) {
                        self.placed_tiles.push(placed_tile);
                        match self.place_next_tile(settings, num_remaining - 1) {
                            true => {
                                return true;
                            }
                            false => {
                                self.placed_tiles.pop();
                                // implicit continue
                            }
                        }
                    }
                }
            }
        }
        // if we made it this far without a placement, something is wrong.
        return false;
    }
}

pub fn run(settings: &Settings) -> impl Iterator<Item = StyledObj2> {
    let all_tiles = all_girih_tiles();

    let mut layout = Layout::new({
        let tile = Tile::new(Girih::SormehDan);
        let mut pg2 = tile.to_naive_pg2();
        PlacedTile { pg2, tile }
    });

    assert!(layout.place_next_tile(settings, settings.num_iterations));

    layout.to_styledobjs()
}
