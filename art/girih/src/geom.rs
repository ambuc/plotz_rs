use anyhow::Result;
use plotz_color::{subway::PURPLE_7, ColorRGB, LIGHTBLUE, LIMEGREEN, ORANGERED, YELLOW};
use plotz_geometry::{
    bounded::Bounded,
    crop::PointLocation,
    overlaps::opinion::SegmentOpinion,
    shapes::{
        point::{Point, PolarPt},
        polygon::Polygon,
        ray::Ray,
        segment::Segment,
    },
};
use rand::seq::SliceRandom;
use std::f64::consts::*;

// girih tiles https://en.m.wikipedia.org/wiki/Girih_tiles. The five shapes of
// the tiles, and their Persian names, are:
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum Girih {
    Tabl,
    SheshBand,
    SormehDan,
    Torange,
    Pange,
}

impl Girih {
    pub fn color(&self) -> ColorRGB {
        match self {
            Girih::Tabl => LIGHTBLUE,
            Girih::SheshBand => LIMEGREEN,
            Girih::SormehDan => ORANGERED,
            Girih::Torange => PURPLE_7,
            Girih::Pange => YELLOW,
        }
    }

    pub fn num_pts(&self) -> usize {
        match self {
            Girih::Tabl => 10,
            Girih::SheshBand => 6,
            Girih::SormehDan => 6,
            Girih::Torange => 4,
            Girih::Pange => 5,
        }
    }
}

pub fn all_girih_tiles() -> Vec<Girih> {
    vec![
        Girih::Tabl,
        Girih::SheshBand,
        Girih::SormehDan,
        Girih::Torange,
        Girih::Pange,
    ]
}

pub fn all_girih_tiles_in_random_order() -> Vec<Girih> {
    let mut tiles: Vec<Girih> = all_girih_tiles();
    let mut rng = rand::thread_rng();
    tiles.shuffle(&mut rng);
    tiles
}

#[derive(Clone, Debug)]
pub struct Tile {
    enum_type: Girih,
    angs_rad: Vec<f64>,
}

impl Tile {
    pub fn new(g: Girih) -> Tile {
        // these are all in radians
        const ANG_2: f64 = 2.0 * PI / 5.0;
        const ANG_3: f64 = 3.0 * PI / 5.0;
        const ANG_4: f64 = 4.0 * PI / 5.0;
        const ANG_6: f64 = 6.0 * PI / 5.0;

        Tile {
            enum_type: g,
            angs_rad: match g {
                Girih::Tabl => [ANG_4; 10].to_vec(),
                Girih::Pange => [ANG_3; 5].to_vec(),
                Girih::SheshBand => [ANG_4, ANG_4, ANG_2, ANG_4, ANG_4, ANG_2].to_vec(),
                Girih::SormehDan => [ANG_2, ANG_2, ANG_6, ANG_2, ANG_2, ANG_6].to_vec(),
                Girih::Torange => [ANG_3, ANG_2, ANG_3, ANG_2].to_vec(),
            },
        }
    }

    fn angles_deg(&self) -> Vec<f64> {
        self.angs_rad.iter().map(|i| i * 180.0 / PI).collect()
    }

    // what's naive about this? SO glad you asked bestie. it's the right shape
    // but that's it. you have to place this somewhere sensible upon usage.
    pub fn to_naive_pg(&self) -> Polygon {
        let vertex_turn_angles: &[f64] = &self.angles_deg();
        let mut cursor_position = Point(0, 0);
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
        // very last point, Pg() automatically closes it for us.
        accumulated.pop();
        let mut pg = Polygon(accumulated).unwrap();
        pg.rotate(&Point(0, 0), 0.00001);
        pg
    }

    pub fn color(&self) -> ColorRGB {
        self.enum_type.color()
    }

    pub fn place(self, c: Constraint) -> PlacedTile {
        let naive_pg = self.to_naive_pg();
        let naive_sg = naive_pg.to_segments()[c.src_index];

        let mut modified_pg = naive_pg;

        let t = c.target.midpoint() - naive_sg.midpoint();
        modified_pg += t;

        let modified_sg = modified_pg.to_segments()[c.src_index];

        let rotation: f64 = (c.target.ray_angle()) - naive_sg.ray_angle();
        modified_pg.rotate(&modified_sg.midpoint(), rotation);

        PlacedTile {
            pg: modified_pg,
            tile: self,
        }
    }
}

// place tile sg #{usize} along real segment {Sg}.
// because girih tiles all have the same length, this will involve rotation and
// translation but never scaling.
#[derive(Debug)]
pub struct Constraint<'a> {
    pub src_index: usize,
    pub target: &'a Segment,
}

#[derive(Debug)]
pub struct PlacedTile {
    pub pg: Polygon,
    pub tile: Tile,
}

impl PlacedTile {
    pub fn to_strapwork(&self) -> Result<Vec<Segment>> {
        let g = self.tile.enum_type;
        let mut strapwork = vec![];

        for (edge1, edgeb) in self
            .pg
            .to_segments()
            .iter()
            .zip(self.pg.to_segments().iter().cycle().skip(1))
        {
            let a_ray_angle = {
                let a_angle = edge1.ray_angle();

                let angle_1 = a_angle + (3.0 * PI / 10.0);
                let angle_2 = a_angle + (-7.0 * PI / 10.0);

                let sg_1_f = edge1.midpoint() + PolarPt(0.1, angle_1);
                let sg_2_f = edge1.midpoint() + PolarPt(0.1, angle_2);
                match (self.pg.contains_pt(&sg_1_f)?, self.pg.contains_pt(&sg_2_f)?) {
                    (PointLocation::Inside, _) => angle_1,
                    (_, PointLocation::Inside) => angle_2,
                    _ => panic!("oh"),
                }
            };

            let a_ray: Ray = Ray(edge1.midpoint(), a_ray_angle);

            if let Some(_) = a_ray.intersects_sg(edgeb)? {
                strapwork.push(Segment(edge1.midpoint(), edgeb.midpoint()));
            } else {
                // imagine a bridge from a_mdpt to b_mdpt.
                // out of the center of the bridge rise2 a perpendicular tower.
                let bridge = Segment(edge1.midpoint(), edgeb.midpoint());
                let tower_a = Ray(bridge.midpoint(), bridge.ray_angle() - FRAC_PI_2);
                let tower_b = Ray(bridge.midpoint(), bridge.ray_angle() + FRAC_PI_2);

                // ztex lies at the intersection of a_ray and the tower.
                let ztex = match (tower_a.intersects(&a_ray)?, tower_b.intersects(&a_ray)?) {
                    (Some((op, _)), _) | (_, Some((op, _))) => match op {
                        SegmentOpinion::AtPointAlongSegment { at_point, .. } => at_point,
                        _ => panic!("oh"),
                    },
                    _ => panic!("oh"),
                };

                strapwork.extend(&[
                    Segment(edge1.midpoint(), ztex),
                    Segment(ztex, edgeb.midpoint()),
                ]);
            }
        }

        // columbo voice: one last thing -- some of these strapworks might intersect with each other.
        // if they do, crop them by each other (i.e., if ab intersects cd at x, create ax, xb, cx, xd)
        // and remove the ones with one end outside of the tile.

        let mut s_ver = vec![];

        let tile_contains = |sg: &Segment| {
            self.pg.point_is_inside_or_on_border(&sg.i)
                && self.pg.point_is_inside_or_on_border(&sg.f)
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
                    let (perp_ray_1, perp_ray_2) =
                        self.pg.to_segments()[0].rays_perpendicular_both();

                    let pt_inside = match (
                        self.pg.point_is_inside_or_on_border(&s.i),
                        self.pg.point_is_inside_or_on_border(&s.f),
                    ) {
                        (true, false) => s.i,
                        (false, true) => s.f,
                        _ => panic!("oh"),
                    };

                    match (perp_ray_1.intersects_sg(&s)?, perp_ray_2.intersects_sg(&s)?) {
                        (Some((op, _)), _) | (_, Some((op, _))) => match op {
                            SegmentOpinion::AtPointAlongSegment { at_point, .. } => {
                                s_ver.push(Segment(pt_inside, at_point))
                            }
                            _ => panic!("oh"),
                        },
                        _ => panic!("oh"),
                    }
                }
                (false, _) => {
                    panic!("uh oh")
                }
            }
        }
        Ok(s_ver)
    }

    pub fn to_annotated_placed_tiles(&self) -> Result<AnnotatedPlacedTile> {
        let axis = Point(0, 0);
        let offset = 0.01;

        Ok(AnnotatedPlacedTile {
            girih: self.tile.enum_type,
            outline: self.pg.clone(),
            straps: (PlacedTile {
                pg: {
                    let mut m = self.pg.clone();
                    m.rotate(&axis, offset);
                    m
                },
                tile: self.tile.clone(),
            })
            .to_strapwork()?
            .into_iter()
            .map(|mut s| {
                s.rotate(&axis, -offset);
                s
            })
            .collect(),
        })
    }

    pub fn test_pts(&self) -> Result<Vec<Point>> {
        let cand_ctr = self.pg.bounds()?.center();
        Ok(std::iter::once(cand_ctr)
            .chain(
                self.pg
                    .to_segments()
                    .iter()
                    .map(|sg| -> Point { sg.midpoint() }),
            )
            .chain(self.pg.pts.iter().map(|pt| pt.avg(&cand_ctr)))
            .collect::<Vec<_>>())
    }
}

pub struct AnnotatedPlacedTile {
    pub girih: Girih,
    pub outline: Polygon,
    pub straps: Vec<Segment>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_foo() {
        // assert_eq!(1, 2);
        let _t = Tile::new(Girih::Tabl);
        let _p = Tile::new(Girih::Pange);
    }
}
