use plotz_color::{subway::PURPLE_7, ColorRGB, LIGHTBLUE, LIMEGREEN, ORANGERED, YELLOW};
use plotz_geometry::styled_obj2::StyledObj2;

use {
    plotz_geometry::{
        crop::PointLoc,
        isxn::{Intersection, IsxnResult},
        shapes::{
            pg2::Pg2,
            pt2::{PolarPt, Pt2},
            ry2::Ry2,
            sg2::Sg2,
        },
    },
    rand::seq::SliceRandom,
    std::f64::consts::*,
};

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
    pub fn color(&self) -> &'static ColorRGB {
        match self {
            Girih::Tabl => &LIGHTBLUE,
            Girih::SheshBand => &LIMEGREEN,
            Girih::SormehDan => &ORANGERED,
            Girih::Torange => &PURPLE_7,
            Girih::Pange => &YELLOW,
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

// Kind
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum K {
    A,
    B,
    C,
}

#[derive(Clone)]
pub struct Tile {
    enum_type: Girih,
    angs_rad: Vec<f64>,
    placed_pg2: Option<Pg2>,
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
            placed_pg2: None,
        }
    }

    fn angles_deg(&self) -> Vec<f64> {
        self.angs_rad.iter().map(|i| i * 180.0 / PI).collect()
    }

    // what's naive about this? SO glad you asked bestie. it's the right shape
    // but that's it. you have to place this somewhere sensible upon usage.
    pub fn to_naive_pg2(&self) -> Pg2 {
        let vertex_turn_angles: &[f64] = &self.angles_deg();
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
        let mut pg2 = Pg2(accumulated);
        pg2.rotate(&Pt2(0, 0), 0.00001);
        pg2
    }

    fn to_pointtypes(&self) -> Vec<K> {
        match self.enum_type {
            Girih::Tabl => vec![K::A; 10],
            Girih::SheshBand => vec![K::A, K::B, K::C, K::A, K::B, K::C],
            Girih::SormehDan => vec![K::A, K::B, K::C, K::A, K::B, K::C],
            Girih::Torange => vec![K::A, K::B, K::A, K::B],
            Girih::Pange => vec![K::A; 5],
        }
    }

    pub fn color(&self) -> &'static ColorRGB {
        self.enum_type.color()
    }

    pub fn place(self, c: Constraint) -> PlacedTile {
        let mut naive_pg = self.to_naive_pg2();
        let naive_sg: Sg2 = naive_pg.to_segments()[c.src_index];
        let target_sg: Sg2 = c.target;

        let translation = target_sg.i - naive_sg.i;
        let rotation: f64 = target_sg.ray_angle() - naive_sg.ray_angle();

        let mut modified_pg = naive_pg + translation;
        modified_pg.rotate(&modified_pg.to_segments()[c.src_index].i, rotation);

        PlacedTile {
            pg2: modified_pg,
            tile: self,
        }
    }
}

// place tile sg #{usize} along real segment {Sg2}.
// because girih tiles all have the same length, this will involve rotation and
// translation but never scaling.
pub struct Constraint {
    pub src_index: usize,
    pub target: Sg2,
}

pub struct PlacedTile {
    pg2: Pg2,
    tile: Tile,
}

impl PlacedTile {
    pub fn to_strapwork(&self) -> Vec<Sg2> {
        let g = self.tile.enum_type;
        let mut strapwork = vec![];

        for (edge1, edgeb) in self
            .pg2
            .to_segments()
            .iter()
            .zip(self.pg2.to_segments().iter().cycle().skip(1))
        {
            let a_ray_angle = {
                let a_angle = edge1.ray_angle();

                let angle_1 = a_angle + (3.0 * PI / 10.0);
                let angle_2 = a_angle + (-7.0 * PI / 10.0);

                let sg_1_f = edge1.midpoint() + PolarPt(0.1, angle_1);
                let sg_2_f = edge1.midpoint() + PolarPt(0.1, angle_2);
                match (self.pg2.contains_pt(&sg_1_f), self.pg2.contains_pt(&sg_2_f)) {
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
                self.pg2.point_is_inside_or_on_border(&sg.i)
                    && self.pg2.point_is_inside_or_on_border(&sg.f)
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
                            self.pg2.to_segments()[0].rays_perpendicular_both();

                        let pt_inside = match (
                            self.pg2.point_is_inside_or_on_border(&s.i),
                            self.pg2.point_is_inside_or_on_border(&s.f),
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

    pub fn to_styledobjs(&self) -> Vec<StyledObj2> {
        let mut v: Vec<StyledObj2> = vec![];

        let outline: StyledObj2 = StyledObj2::new(self.pg2.clone())
            .with_thickness(1.0)
            .with_color(self.tile.color());
        v.push(outline);

        let axis = Pt2(0, 0);
        let offset = 0.01;
        let straps: Vec<_> = (PlacedTile {
            pg2: {
                let mut m = self.pg2.clone();
                m.rotate(&axis, offset);
                m
            },
            tile: self.tile.clone(),
        })
        .to_strapwork()
        .into_iter()
        .map(|mut s| {
            s.rotate(&axis, -offset);
            StyledObj2::new(s)
                .with_thickness(3.0)
                .with_color(self.tile.color())
        })
        .collect();
        v.extend(straps);

        v
    }
}

// #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
// struct PieSlice(Girih, usize, K);
//
// const ALL_SLICES: &[PieSlice] = &[
//     // TABL => (4/5, K::A)
//     PieSlice(Girih::Tabl, 4, K::A),
//     // PANGE => (3/5, K::A)
//     PieSlice(Girih::Pange, 3, K::A),
//     // SHESHBAND => (2/5, K::A), (4/5, K::B), (4/5, K::C)
//     PieSlice(Girih::SheshBand, 2, K::A),
//     PieSlice(Girih::SheshBand, 4, K::B),
//     PieSlice(Girih::SheshBand, 4, K::C),
//     // TORANGE => (2/5, K::A), (3/5, K::B)
//     PieSlice(Girih::Torange, 2, K::A),
//     PieSlice(Girih::Torange, 3, K::B),
//     // SORMEHDAN => (6/5, K::A), (2/5, K::B), (2/5, K::C)
//     PieSlice(Girih::SormehDan, 6, K::A),
//     PieSlice(Girih::SormehDan, 2, K::B),
//     PieSlice(Girih::SormehDan, 2, K::C),
// ];
//
// #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
// struct PieChart {
//     pie_slices: Vec<PieSlice>,
// }
//
// impl PieChart {
//     fn is_complete(&self) -> bool {
//         self.pie_slices
//             .iter()
//             .map(|PieSlice(_, n, _)| n)
//             .sum::<usize>()
//             == 10
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_foo() {
        // assert_eq!(1, 2);
        let t = Tile::new(Girih::Tabl);
        let p = Tile::new(Girih::Pange);
    }
}
