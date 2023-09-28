use crate::geom::*;
use indicatif::ProgressBar;
use itertools::Itertools;
use plotz_color::{ColorRGB, RED};
use plotz_geometry::{
    obj2::Obj2,
    shapes::{
        pg2::{multiline::Multiline, Pg2},
        pt2::Pt2,
        sg2::Sg2,
    },
    style::Style,
    styled_obj2::StyledObj2,
};
use rand::seq::SliceRandom;
use rayon::iter::*;
use std::{error::Error, f32::EPSILON, f64::consts::TAU};
use tracing::{error, info, warn};

#[derive(Debug)]
enum StrapsColoring {
    Original,
    Chasing,
}

#[derive(Debug)]
enum Display {
    JustTiles,
    JustStraps(StrapsColoring),
    All,
}

#[derive(Debug)]
struct Settings {
    num_iterations: usize,
    is_deterministic: bool,
    display: Display,
}

impl Settings {
    fn choices(&self) -> Vec<Girih> {
        match self.is_deterministic {
            true => all_girih_tiles(),
            false => {
                let mut weighted_choices = vec![
                    Girih::SormehDan,
                    Girih::Tabl,
                    Girih::Pange,
                    Girih::Torange,
                    Girih::SheshBand,
                ];
                let mut rng = rand::thread_rng();
                weighted_choices.shuffle(&mut rng);
                weighted_choices
            }
        }
    }
}

struct Layout {
    settings: Settings,
    placed_tiles: Vec<PlacedTile>,
}
impl Layout {
    fn new(settings: Settings, pt: PlacedTile) -> Layout {
        Layout {
            settings,
            placed_tiles: vec![pt],
        }
    }

    fn to_styledobjs(&self) -> Vec<StyledObj2> {
        let mut res = vec![];
        for placed_tile in &self.placed_tiles {
            let spt = placed_tile.to_styledobjs();
            match self.settings.display {
                Display::JustTiles => {
                    res.push(spt.outline);
                }
                Display::JustStraps(_) => {
                    res.extend(spt.straps);
                }
                Display::All => {
                    res.push(spt.outline);
                    res.extend(spt.straps);
                }
            }
        }
        res
    }

    fn next_bare_edge(&self) -> Sg2 {
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

        let ctr: Pt2 = Pt2(0, 0);
        bare_edges
            .into_iter()
            .min_by_key(|sg| float_ord::FloatOrd(sg.midpoint().dist(&ctr)))
            .expect("bare_edges should never be empty")
    }

    fn place_tile_on_edge_src(&self, g: Girih, c: Constraint) -> Option<PlacedTile> {
        let cand: PlacedTile = Tile::new(g).clone().place(c);
        match self.evaluate_cand(&cand) {
            true => Some(cand),
            false => None,
        }
    }

    fn evaluate_cand(&self, cand: &PlacedTile) -> bool {
        let test_pts = cand.test_pts();

        if (self.placed_tiles.iter())
            .cartesian_product(test_pts.iter())
            .collect::<Vec<_>>()
            .par_iter()
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
        if cand.pg2.to_segments().iter().any(|cand_sg| -> bool {
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
    fn place_next_tile(&mut self, num_remaining: usize, bar: &mut ProgressBar) -> bool {
        // info!("place_next_tile: {:?}", num_remaining);
        if num_remaining == 0 {
            return true;
        }

        let next_bare_edge: Sg2 = self.next_bare_edge();

        for g in self.settings.choices() {
            let num_pts = Tile::new(g).to_naive_pg2().pts.len();

            let next_tiles: Vec<_> = [next_bare_edge, next_bare_edge.flip()]
                .into_iter()
                .cartesian_product(0..num_pts)
                .collect::<Vec<_>>()
                .into_par_iter()
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
                match self.place_next_tile(num_remaining - 1, bar) {
                    true => {
                        return true;
                    }
                    false => {
                        self.placed_tiles.pop();
                        bar.set_position(bar.position() - 1);
                        // implicit continue
                    }
                }
            }
        }
        // if we made it this far without a placement, something is wrong.
        return false;
    }
    fn postprocess(&self, so2s: Vec<StyledObj2>) -> Vec<StyledObj2> {
        match self.settings.display {
            Display::JustStraps(StrapsColoring::Chasing) => {
                let mut v = vec![];
                // v.extend(so2s.clone());
                v.extend(chase(so2s));
                v
            }
            _ => so2s,
        }
    }
}

fn pts_eq_within(a: Pt2, b: Pt2, epsilon: f64) -> bool {
    a.dist(&b) < epsilon
}
fn vals_eq_within(a: f64, b: f64, epsilon: f64) -> bool {
    (a - b).abs() < epsilon
}

fn chase(inputs: Vec<StyledObj2>) -> Vec<StyledObj2> {
    // first of all, we're guaranteed that every element in so2s is a strap. nothing else.
    let mut inputs: Vec<Sg2> = inputs
        .into_iter()
        .map(|so2| so2.inner.to_sg2().unwrap().clone())
        .collect();

    let mut outputs: Vec<StyledObj2> = vec![];
    let epsilon = 0.001;

    while let Some(first) = inputs.pop() {
        let mut segments: Vec<Sg2> = vec![];

        // collect links in the chain. implicitly going sg.i -> sg.f.
        segments.push(first);

        while let Some(next_idx) = inputs.iter().position(|cand_sg| {
            pts_eq_within(cand_sg.i, segments.last().unwrap().f, epsilon)
                || pts_eq_within(cand_sg.f, segments.last().unwrap().f, epsilon)
        }) {
            // get next sg
            let cand_sg = inputs[next_idx];
            let next_sg = if pts_eq_within(cand_sg.i, segments.last().unwrap().f, epsilon) {
                //
                cand_sg
            } else if pts_eq_within(cand_sg.f, segments.last().unwrap().f, epsilon) {
                //
                cand_sg.flip()
            } else {
                panic!("why did you think there was?");
            };
            // remove sg at next_idx
            inputs.remove(next_idx);
            // use next_sg
            segments.push(next_sg);
        }

        // turn that chain into a list of deduplicated points
        let mut pts: Vec<Pt2> = segments
            .into_iter()
            .flat_map(|sg2| [sg2.i, sg2.f])
            .collect();
        pts.dedup();

        // and then make a multiline, and add it to our final outputs list.
        outputs.push(
            StyledObj2::new(Multiline(pts).unwrap())
                // .with_color(&RED)
                .with_color(plotz_color::take_random_colors(1)[0])
                .with_thickness(3.0),
        );
    }

    outputs
}

pub fn run() -> Vec<StyledObj2> {
    let mut layout = Layout::new(
        Settings {
            num_iterations: 10,
            is_deterministic: false,
            display: Display::JustStraps(StrapsColoring::Chasing),
            // display: Display::JustStraps(StrapsColoring::Original),
            // display: Display::All,
        },
        {
            let tile = Tile::new(Girih::SormehDan);
            let pg2 = tile.to_naive_pg2();
            PlacedTile { pg2, tile }
        },
    );

    let mut bar = ProgressBar::new(layout.settings.num_iterations.try_into().unwrap());
    assert!(layout.place_next_tile(layout.settings.num_iterations, &mut bar));
    bar.finish();

    layout.postprocess(layout.to_styledobjs())
}
