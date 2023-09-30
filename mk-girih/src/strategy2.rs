// #![allow(unused)]

use crate::geom::*;
use indicatif::ProgressBar;
use itertools::Itertools;
use plotz_color::BLACK;
use plotz_geometry::{
    shading::{
        shade_config::{self, ShadeConfig},
        shade_polygon,
    },
    shapes::{
        curve::CurveArc,
        pg2::{abp, multiline::Multiline, Pg2},
        pt2::{PolarPt, Pt2},
        sg2::Sg2,
    },
    styled_obj2::StyledObj2,
};
use rand::seq::SliceRandom;
use std::f64::consts::{FRAC_PI_2, TAU};

#[derive(Debug)]
enum Instr {
    StrapsOriginal(/*thickness */ f64),
    StrapsChasing,
    TilesOutline(/*thickness*/ f64),
    TileShaded(ShadeConfig),
}

#[derive(Debug)]
struct Display(Vec<Instr>);

#[derive(Debug)]
struct Settings {
    num_iterations: usize,
    is_deterministic: bool,
    display: Display,
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
struct StyledPlacedTiles {
    outlines: Vec<StyledObj2>,
    straps: Vec<StyledObj2>,
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

    fn to_styledobjs(&self) -> StyledPlacedTiles {
        let mut spts = StyledPlacedTiles {
            outlines: vec![],
            straps: vec![],
        };
        for placed_tile in &self.placed_tiles {
            let spt = placed_tile.to_styledobjs();
            spts.outlines.push(spt.outline);
            spts.straps.extend(spt.straps);
        }
        spts
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
            .iter()
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
        if num_remaining == 0 {
            return true;
        }

        let next_bare_edge: Sg2 = self.next_bare_edge();

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
                if self.place_next_tile(num_remaining - 1, bar) {
                    return true;
                }
                self.placed_tiles.pop();
                bar.set_position(bar.position() - 1);
            }
        }
        // if we made it this far without a placement, something is wrong.
        return false;
    }

    fn postprocess(&self, spts: StyledPlacedTiles) -> Vec<StyledObj2> {
        let mut v = vec![];

        self.settings.display.0.iter().for_each(|inst| match inst {
            Instr::StrapsOriginal(thickness) => {
                v.extend(spts.clone().straps.into_iter().map(|mut so2| {
                    so2.style.thickness = *thickness;
                    so2
                }))
            }
            Instr::StrapsChasing => v.extend(chase(&spts)),
            Instr::TilesOutline(thickness) => {
                v.extend(spts.clone().outlines.into_iter().map(|so2| {
                    StyledObj2::new(so2.inner)
                        .with_color(&BLACK)
                        .with_thickness(*thickness)
                }))
            }
            Instr::TileShaded(shade_config) => {
                v.extend(spts.clone().outlines.iter().flat_map(|outline| {
                    shade_polygon(&shade_config, outline.inner.to_pg2().unwrap())
                        .unwrap()
                        .into_iter()
                        .map(|shade| {
                            StyledObj2::new(shade)
                                .with_color(outline.style.color)
                                .with_thickness(1.0)
                        })
                }))
            }
        });

        v
    }
}

fn pts_eq_within(a: Pt2, b: Pt2, epsilon: f64) -> bool {
    a.dist(&b) < epsilon
}
fn vals_eq_within(a: f64, b: f64, epsilon: f64) -> bool {
    (a - b).abs() < epsilon
}

fn chase(styled_placed_tiles: &StyledPlacedTiles) -> Vec<StyledObj2> {
    // first of all, we're guaranteed that every element in so2s is a strap. nothing else.
    let mut inputs: Vec<Sg2> = styled_placed_tiles
        .straps
        .iter()
        .map(|so2| so2.inner.to_sg2().unwrap().clone())
        .collect();

    let mut outputs: Vec<StyledObj2> = vec![];
    let epsilon = 0.001;

    // collect links in the chain. implicitly going sg.i -> sg.f.
    while let Some(first) = inputs.pop() {
        let mut segments: Vec<Sg2> = vec![first];

        'l: loop {
            let last = segments.last().unwrap();

            let positions = inputs
                .iter()
                .positions(|cand_sg| {
                    pts_eq_within(cand_sg.i, last.f, epsilon)
                        || pts_eq_within(cand_sg.f, last.f, epsilon)
                })
                .collect::<Vec<usize>>();

            let next_idx: usize = match positions[..] {
                [] => break 'l,
                [next_idx] => next_idx,
                _ => *positions
                    .iter()
                    .find(|cand_idx| {
                        vals_eq_within(inputs[**cand_idx].ray_angle(), last.ray_angle(), epsilon)
                    })
                    .unwrap(),
            };

            let cand_sg: Sg2 = inputs.remove(next_idx); // get next sg

            let next_sg = if pts_eq_within(cand_sg.i, last.f, epsilon) {
                cand_sg
            } else {
                cand_sg.flip()
            };

            segments.push(next_sg); // use next_sg
        }

        let mut pts = segments.iter().map(|sg2| sg2.i).collect::<Vec<_>>();
        pts.push(segments.first().unwrap().i);

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

/*
fn scallop(styled_placed_tiles: &StyledPlacedTiles) -> Vec<StyledObj2> {
    let mut v = vec![];
    //

    let d = 0.1;

    for o in styled_placed_tiles.outlines.iter() {
        let pg2 = o.inner.to_pg2().unwrap();
        for sg2 in pg2.to_segments() {
            let m: Pt2 = sg2.midpoint();
            let o: Pt2 = m + PolarPt(d, sg2.ray_angle() + FRAC_PI_2);
            let r: f64 = o.dist(&a);
            let ang_a: f64 = abp(&o, &(o + Pt2(1.0, 0.0)), a);
            let ang_b: f64 = abp(&o, &(o + Pt2(1.0, 0.0)), b);

            let s = o.style;

            v.push(StyledObj2::new(CurveArc(o, ang_a..ang_b, r)));
        }
    }

    v
}
*/

pub fn run() -> Vec<StyledObj2> {
    let shade_config = ShadeConfig::builder()
        .gap(0.05)
        .slope(0.5)
        .switchback(false)
        .build();
    let mut layout = Layout::new(
        Settings {
            num_iterations: 50,
            is_deterministic: false,
            display: Display(vec![
                // Instr::StrapsOriginal(2.0),
                Instr::TilesOutline(1.0),
                Instr::TileShaded(shade_config),
                Instr::StrapsChasing,
            ]),
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
