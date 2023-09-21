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
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Girih {
    Tabl,
    SheshBand,
    SormehDan,
    Torange,
    Pange,
}

fn all_girih_tiles() -> Vec<Girih> {
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

fn make_strapwork(g: Girih, tile: &Pg2) -> Vec<Sg2> {
    let mut strapwork = vec![];

    for (edge1, edgeb) in tile
        .to_segments()
        .iter()
        .zip(tile.to_segments().iter().cycle().skip(1))
    {
        let a_ray_angle = {
            let a_angle = edge1.ray_angle();

            let angle_1 = a_angle + (3.0 * PI / 10.0);
            let angle_2 = a_angle + (-7.0 * PI / 10.0);

            let sg_1_f = edge1.midpoint() + PolarPt(0.1, angle_1);
            let sg_2_f = edge1.midpoint() + PolarPt(0.1, angle_2);
            match (tile.contains_pt(&sg_1_f), tile.contains_pt(&sg_2_f)) {
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
            tile.point_is_inside_or_on_border(&sg.i) && tile.point_is_inside_or_on_border(&sg.f)
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
                    let (perp_ray_1, perp_ray_2) = tile.to_segments()[0].rays_perpendicular_both();

                    let pt_inside = match (
                        tile.point_is_inside_or_on_border(&s.i),
                        tile.point_is_inside_or_on_border(&s.f),
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

// Kind
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum K {
    A,
    B,
    C,
}

// accepts a list of interior angles, in degrees.
fn make_girih_polygon_from_vertex_turn_angles(vertex_turn_angles: &[f64]) -> Pg2 {
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
    Pg2(accumulated)
}

pub fn make_tile(g: Girih) -> (Pg2, Vec<K>) {
    let (vertexes, pointtypes) = match g {
        Girih::Tabl => (vec![144.0; 10], vec![K::A; 10]),
        Girih::SheshBand => (
            vec![72.0, 144.0, 144.0, 72.0, 144.0, 144.0],
            vec![K::A, K::B, K::C, K::A, K::B, K::C],
        ),
        Girih::SormehDan => (
            vec![72.0, 72.0, 216.0, 72.0, 72.0, 216.0],
            vec![K::A, K::B, K::C, K::A, K::B, K::C],
        ),
        Girih::Torange => (vec![72.0, 108.0, 72.0, 108.0], vec![K::A, K::B, K::A, K::B]),
        Girih::Pange => (vec![108.0; 5], vec![K::A; 5]),
    };

    let pg2 = make_girih_polygon_from_vertex_turn_angles(&vertexes);
    (pg2, pointtypes)
}

pub fn make_girih_tile_and_strapwork(g: Girih) -> (Pg2, Vec<Sg2>) {
    let (mut tile, _) = make_tile(g);

    // NB must offset or vertical line tangents don't work, lmfao
    tile.rotate(&Pt2(0, 0), 0.00001);

    let strapwork = make_strapwork(g, &tile);

    (tile, strapwork)
}
