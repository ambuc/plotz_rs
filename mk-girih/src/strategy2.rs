use crate::geom::*;
use average::Mean;
use itertools::Itertools;
use plotz_geometry::{
    bounded::Bounded,
    shapes::{pg2::Pg2, pt2::Pt2, sg2::Sg2},
    styled_obj2::StyledObj2,
};
use tracing::info;

#[derive(Debug)]
pub struct Settings {
    pub num_iterations: usize,
    pub is_deterministic: bool,
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
        // step (1) find average center of whole board.
        let ctrs: Vec<Pt2> = self
            .placed_tiles
            .iter()
            .map(|placed_tile| placed_tile.pg2.bbox_center())
            .collect::<Vec<_>>();

        let mean_x: Mean = ctrs.iter().map(|pt2| pt2.x.0).collect();
        let mean_y: Mean = ctrs.iter().map(|pt2| pt2.y.0).collect();
        let ctr: Pt2 = Pt2(mean_x.mean(), mean_y.mean());

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

    // we know the tile and the target edge, but not the source edge.
    // returns the placed tile if this was successfully placed _without_ a collision.
    // otherwise, returns none.
    fn place_tile_on_edge(&self, g: Girih, target: &Sg2) -> Option<PlacedTile> {
        let naive_tile: Tile = Tile::new(g);
        let naive_pg2: Pg2 = naive_tile.to_naive_pg2();

        for src_index in 0..naive_pg2.to_segments().len() {
            let constraint = Constraint {
                src_index,
                target: *target,
            };
            let cand: PlacedTile = naive_tile.clone().place(constraint);
            if self.evaluate_cand(&cand) {
                return Some(cand);
            }
        }

        None
    }

    fn evaluate_cand(&self, cand: &PlacedTile) -> bool {
        let test_pts: Vec<Pt2> = std::iter::once(cand.pg2.bbox_center())
            .chain(
                cand.pg2
                    .to_segments()
                    .iter()
                    .map(|sg2| -> Pt2 { sg2.midpoint() }),
            )
            .collect::<Vec<_>>();

        (self.placed_tiles.iter())
            .cartesian_product(test_pts.iter())
            .all(|(extant_tile, test_pt)| !extant_tile.pg2.point_is_inside(&test_pt))
    }

    // returns true if successfully placed tile (or if no tile needed to be placed.)
    fn place_next_tile(&mut self, settings: &Settings, num_remaining: usize) -> bool {
        info!("place_next_tile: {:?}", num_remaining);
        if num_remaining == 0 {
            return true;
        }

        let next_bare_edge: Sg2 = self.next_bare_edge();

        for g in match settings.is_deterministic {
            true => all_girih_tiles(),
            false => all_girih_tiles_in_random_order(),
        } {
            for cand_edge in [next_bare_edge, next_bare_edge.flip()] {
                if let Some(placed_tile) = self.place_tile_on_edge(g, &cand_edge) {
                    self.placed_tiles.push(placed_tile);
                    match self.place_next_tile(settings, num_remaining - 1) {
                        true => {
                            return true;
                        }
                        false => {
                            self.placed_tiles.pop();
                            continue;
                        }
                    }
                }
            }
        }
        // if we made it this far without a placement, something is wrong.
        return false;
    }

    fn add(&mut self, pt: PlacedTile) {
        self.placed_tiles.push(pt)
    }
}

pub fn run(settings: &Settings) -> impl Iterator<Item = StyledObj2> {
    let all_tiles = all_girih_tiles();

    let mut layout = Layout::new(Tile::new(all_tiles[0]).place(Constraint {
        src_index: 0,
        target: Sg2(Pt2(0, 0), Pt2(1, 0)),
    }));

    assert!(layout.place_next_tile(settings, settings.num_iterations));

    layout.to_styledobjs()
}
