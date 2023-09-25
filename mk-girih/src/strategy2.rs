use crate::geom::{all_girih_tiles, Constraint, Girih, PlacedTile, Tile};
use itertools::{all, Itertools};
use plotz_geometry::{
    bounded::Bounded,
    isxn::IsxnResult,
    shapes::{pg2::Pg2, pt2::Pt2, sg2::Sg2},
    styled_obj2::StyledObj2,
};
use rand::seq::SliceRandom;
use tracing::info;

#[derive(Debug)]
pub struct Settings {
    pub num_iterations: usize,
    //
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
                        return segment;
                    }
                }
            }
        }
        panic!("this should never happen -- how could we have a set of tiles with no border?")
    }

    // returns true if success.
    fn place_next_tile(&self) -> Option<PlacedTile> {
        info!("Layout::place_next_tile");
        let next_bare_edge: Sg2 = self.next_bare_edge();

        for g in all_girih_tiles() {
            for cand_edge in [next_bare_edge, Sg2(next_bare_edge.f, next_bare_edge.i)] {
                if let Some(placed_tile) = self.place_tile_on_edge(g, &cand_edge) {
                    return Some(placed_tile);
                }
            }
        }

        None
    }

    // we know the tile and the target edge, but not the source edge.
    fn place_tile_on_edge(&self, g: Girih, target: &Sg2) -> Option<PlacedTile> {
        info!("Layout::place_tile_on_edge: g {:?}, target {:?}", g, target);

        let naive_tile: Tile = Tile::new(g);
        let naive_pg2: Pg2 = naive_tile.to_naive_pg2();

        for src_index in 0..naive_pg2.to_segments().len() {
            let constraint = Constraint {
                src_index,
                target: *target,
            };
            info!("Layout::evaluate_cand w/ constraint {:?}", constraint);
            let cand: PlacedTile = naive_tile.clone().place(constraint);
            info!("Layout::evaluate_cand w/ cand {:?}", cand);
            if self.evaluate_cand(&cand) {
                info!("success");
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
            .cartesian_product((test_pts.iter()))
            .all(|(extant_tile, test_pt)| !extant_tile.pg2.point_is_inside(&test_pt))
    }

    fn add(&mut self, pt: PlacedTile) {
        self.placed_tiles.push(pt)
    }
}

pub fn run(settings: &Settings) -> impl Iterator<Item = StyledObj2> {
    info!("settings: {:?}", settings);

    let all_tiles = all_girih_tiles();

    let mut layout = Layout::new(Tile::new(all_tiles[0]).place(Constraint {
        src_index: 0,
        target: Sg2(Pt2(0, 0), Pt2(1, 0)),
    }));

    for _ in 0..=settings.num_iterations {
        let next_tile = layout.place_next_tile().expect("top-level failure");
        layout.add(next_tile);
    }

    info!("layout.placed_tiles: {:?}", layout.placed_tiles.len());

    layout.to_styledobjs()
}
