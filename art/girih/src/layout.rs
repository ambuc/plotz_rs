use crate::geom::*;
use anyhow::{anyhow, Result};
use indicatif::ProgressBar;
use itertools::Itertools;
use plotz_geometry::shapes::{point::Point, polygon::Polygon, segment::Segment};
use rand::seq::SliceRandom;
use std::f64::consts::TAU;

#[derive(Debug)]
pub struct Settings {
    pub num_iterations: usize,
    pub is_deterministic: bool,
}

impl Settings {
    fn choices(&self) -> Vec<Girih> {
        let mut c = all_girih_tiles();
        if !self.is_deterministic {
            let mut rng = rand::thread_rng();
            c.shuffle(&mut rng);
        }
        c
    }
}

#[derive(Debug, Clone)]
pub struct AnnotatedPlacedTiles {
    pub outlines: Vec<(Girih, Polygon)>,
    pub straps: Vec<(Girih, Segment)>,
}

pub struct Layout {
    settings: Settings,
    placed_tiles: Vec<PlacedTile>,
}
impl Layout {
    pub fn new(settings: Settings, pt: PlacedTile) -> Layout {
        Layout {
            settings,
            placed_tiles: vec![pt],
        }
    }

    pub fn to_annotated_placed_tiles(&self) -> Result<AnnotatedPlacedTiles> {
        let mut spts = AnnotatedPlacedTiles {
            outlines: vec![],
            straps: vec![],
        };
        for placed_tile in &self.placed_tiles {
            let spt = placed_tile.to_annotated_placed_tiles()?;
            spts.outlines.push((spt.girih, spt.outline));
            spts.straps
                .extend(spt.straps.into_iter().map(|strap| (spt.girih, strap)));
        }
        Ok(spts)
    }

    fn next_bare_edge(&self) -> Result<Segment> {
        let mut bare_edges = vec![];
        for placed_tile in &self.placed_tiles {
            for segment in placed_tile.pg.to_segments() {
                // both rays which emit from the midpoint.
                let (ray_a, ray_b) = segment.rays_perpendicular_both();
                let offset = segment.length() * 0.1;
                // if there is any point adjacent to the segment (a tiny offset away)
                for pt in [ray_a.to_sg(offset).f, ray_b.to_sg(offset).f] {
                    // for which it is outside of _ALL_ known placed tiles
                    if self.placed_tiles.iter().all(|t| t.pg.point_is_outside(&pt)) {
                        bare_edges.push(segment);
                    }
                }
            }
        }

        let ctr: Point = Point(0, 0);
        bare_edges
            .into_iter()
            .min_by_key(|sg| float_ord::FloatOrd(sg.midpoint().dist(&ctr)))
            .ok_or(anyhow!("?"))
    }

    fn place_tile_on_edge_src(&self, g: Girih, c: Constraint) -> Option<PlacedTile> {
        let cand: PlacedTile = Tile::new(g).place(c);
        match self.evaluate_cand(&cand) {
            true => Some(cand),
            false => None,
        }
    }

    fn evaluate_cand(&self, cand: &PlacedTile) -> bool {
        let test_pts = cand.test_pts().expect("?");

        if (self.placed_tiles.iter())
            .cartesian_product(test_pts.iter())
            .collect::<Vec<_>>()
            .iter()
            .any(|(extant_tile, test_pt)| extant_tile.pg.point_is_inside(test_pt))
        {
            return false;
        }

        // not having collisions is very important. but there is another
        // important characteristic as well. we want to make sure that, around
        // each corner of the newly placed tile, we haven't created a tight
        // corner -- an acute angle of size pi/5 (for example) which no tile
        // could fill.

        // if there's _any_ collision, return false;
        if cand.pg.to_segments().iter().any(|cand_sg| -> bool {
            // returns true if there's a collision
            let mut results: Vec<bool> = vec![];
            let mut rotor = Segment(cand_sg.i, cand_sg.midpoint());
            rotor.rotate(&cand_sg.i, 0.001 * TAU); // offset
            for _ in 0..=10 {
                // ten times, rotate the rotor by TAU/10 (or, (2PI)/10)
                let axis = rotor.i;
                rotor.rotate(&axis, 1.0 / 11.0 * TAU);

                let trial_pt = rotor.f;
                results.push(
                    cand.pg.point_is_inside(&trial_pt)
                        || self
                            .placed_tiles
                            .iter()
                            .any(|extant_tile| extant_tile.pg.point_is_inside(&trial_pt)),
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
            false
        }) {
            // if there's any collision, return false.
            return false;
        }

        true
    }

    // returns true if successfully placed tile (or if no tile needed to be placed.)
    fn place_next_tile(&mut self, num_remaining: usize, bar: &mut ProgressBar) -> Result<bool> {
        if num_remaining == 0 {
            return Ok(true);
        }

        let next_bare_edge: Segment = self.next_bare_edge()?;

        for g in self.settings.choices() {
            let next_tiles: Vec<_> = [next_bare_edge, next_bare_edge.flip()]
                .into_iter()
                .cartesian_product(0..g.num_pts())
                .collect::<Vec<_>>()
                .into_iter()
                .flat_map(|(target, src_index)| {
                    self.place_tile_on_edge_src(
                        g,
                        Constraint {
                            src_index,
                            target: &target,
                        },
                    )
                })
                .collect();

            for placed_tile in next_tiles {
                self.placed_tiles.push(placed_tile);
                bar.inc(1);
                if self.place_next_tile(num_remaining - 1, bar)? {
                    return Ok(true);
                }
                self.placed_tiles.pop();
                bar.set_position(bar.position() - 1);
            }
        }
        // if we made it this far without a placement, something is wrong.
        Ok(false)
    }

    pub fn run(&mut self) -> Result<()> {
        let mut bar = ProgressBar::new(self.settings.num_iterations.try_into().unwrap());
        assert!(self.place_next_tile(self.settings.num_iterations, &mut bar)?);
        bar.finish();
        Ok(())
    }
}
