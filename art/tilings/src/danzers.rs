// https://tilings.math.uni-bielefeld.de/substitution/danzers-7-fold-original/

use float_cmp::assert_approx_eq;
use lazy_static::lazy_static;
use plotz_color::*;
use plotz_geometry::{
    obj2::Obj2,
    shading::{shade_config::ShadeConfig, shade_polygon},
    shapes::{
        point::{Point, PolarPt},
        polygon::Polygon,
    },
    style::Style,
};
use std::f64::consts::PI;

lazy_static! {
    static ref A: f64 = (PI / 7.0).sin();
    static ref B: f64 = (2.0 * PI / 7.0).sin();
    static ref C: f64 = (3.0 * PI / 7.0).sin();
    static ref A_B: f64 = *A + *B;
    static ref B_C: f64 = *B + *C;
    static ref A_B_C: f64 = *A + *B + *C;
    static ref A_C_C: f64 = *A + *C + *C;
    static ref B_B_C: f64 = *B + *B + *C;
    static ref B_C_C: f64 = *B + *C + *C;
    static ref T0: Scalene = Scalene::from_sm_lg(PI / 7.0, 4.0 * PI / 7.0);
    static ref T1: Isosceles = Isosceles::from_base_vertex(3.0 * PI / 7.0, PI / 7.0);
    static ref T2: Isosceles = Isosceles::from_base_vertex(2.0 * PI / 7.0, 3.0 * PI / 7.0);
}

struct Scalene {
    #[allow(dead_code)]
    angle_sm_rad: f64,
    angle_md_rad: f64,
    #[allow(dead_code)]
    angle_lg_rad: f64,
}
impl Scalene {
    fn from_sm_lg(sm: f64, lg: f64) -> Scalene {
        Scalene {
            angle_sm_rad: sm,
            angle_md_rad: PI - sm - lg,
            angle_lg_rad: lg,
        }
    }
}

struct Isosceles {
    angle_base_rad: f64,
    angle_vertex_rad: f64,
}

impl Isosceles {
    fn from_base_vertex(base: f64, vertex: f64) -> Isosceles {
        Isosceles {
            angle_base_rad: base,
            angle_vertex_rad: vertex,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Kind {
    T0,
    T1,
    T2,
}
impl Kind {
    fn as_usize(&self) -> usize {
        match self {
            Kind::T0 => 0,
            Kind::T1 => 1,
            Kind::T2 => 2,
        }
    }
    fn color(&self) -> &'static ColorRGB {
        match self {
            Kind::T0 => &PINK,
            Kind::T1 => &PLUM,
            Kind::T2 => &MAGENTA,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Tile {
    kind: Kind,
    pts: [Point; 3],
}

#[allow(non_snake_case)]
fn Tile(kind: Kind, p1: Point, p2: Point, p3: Point) -> Tile {
    let e = 1000.0 * f64::EPSILON;
    match kind {
        Kind::T0 => {
            assert_approx_eq!(f64, p1.dist(&p2) / p2.dist(&p3), *A / *B, epsilon = e);
            assert_approx_eq!(f64, p2.dist(&p3) / p3.dist(&p1), *B / *C, epsilon = e);
        }
        Kind::T1 => {
            assert_approx_eq!(f64, p1.dist(&p2) / p2.dist(&p3), *A / *C, epsilon = e);
            assert_approx_eq!(f64, p2.dist(&p3) / p3.dist(&p1), 1.0, epsilon = e);
        }
        Kind::T2 => {
            //
            assert_approx_eq!(f64, p1.dist(&p2) / p2.dist(&p3), 1.0, epsilon = e);
            assert_approx_eq!(f64, p2.dist(&p3) / p3.dist(&p1), *B / *C, epsilon = e);
        }
    }
    Tile {
        kind,
        pts: [p1, p2, p3],
    }
}

fn expand_tile(tile: &Tile) -> Vec<Tile> {
    match tile.kind {
        Kind::T0 => {
            let a = tile.pts[2];
            let f = tile.pts[1];
            let h = tile.pts[0];
            let b = a + (h - a) / *B_C_C * *C;
            let d = a + (h - a) / *B_C_C * *B_C;
            let c = a + (f - a) / *A_B_C * *C;
            let e = a + (f - a) / *A_B_C * *B_C;
            let g = f + (h - f) / *A_B * *B;

            vec![
                Tile(Kind::T1, b, c, a),
                Tile(Kind::T0, c, b, d),
                Tile(Kind::T2, c, e, d),
                Tile(Kind::T0, f, e, d),
                Tile(Kind::T2, d, g, f),
                Tile(Kind::T0, h, g, d),
            ]
        }
        Kind::T1 => {
            let a = tile.pts[0];
            let h = tile.pts[1];
            let d = tile.pts[2];
            let b = a + (d - a) / *B_C_C * *C;
            let c = a + (d - a) / *B_C_C * *B_C;
            let g = a + (h - a) / *A_B * *A;
            let i = h + (d - h) / *B_C_C * *C;
            let e = h + (i - h) / *C * *B_C;
            let f = g + (e - g) / *B_C * *B;

            vec![
                Tile(Kind::T1, f, b, a),
                Tile(Kind::T0, a, g, f),
                Tile(Kind::T2, h, g, f),
                Tile(Kind::T1, i, f, h),
                Tile(Kind::T0, f, i, e),
                Tile(Kind::T0, f, b, c),
                Tile(Kind::T1, c, e, f),
                Tile(Kind::T1, c, e, d),
            ]
        }
        Kind::T2 => {
            let e = tile.pts[0];
            let k = tile.pts[1];
            let a = tile.pts[2];
            let i = e + (k - e) / *A_B_C * *C;
            let j = e + (k - e) / *A_B_C * *B_C;
            let c = k + (a - k) / *A_B_C * *B_C;
            let h = k + (a - k) / *A_B_C * *C;
            let g = e + (h - e) / *A_B_C * *B_C;
            let f = e + (h - e) / *A_B_C * *C;
            let b = e + (a - e) / *B_C_C * *B_C;
            let d = e + (a - e) / *B_C_C * *C;

            vec![
                Tile(Kind::T1, d, f, e),
                Tile(Kind::T0, f, d, b),
                Tile(Kind::T2, b, g, f),
                Tile(Kind::T0, h, g, b),
                Tile(Kind::T2, b, c, h),
                Tile(Kind::T0, a, c, b),
                Tile(Kind::T1, f, i, e),
                Tile(Kind::T0, i, f, g),
                Tile(Kind::T2, g, j, i),
                Tile(Kind::T0, k, j, g),
                Tile(Kind::T1, g, h, k),
            ]
        }
    }
}

pub fn make() -> Vec<(Obj2, Style)> {
    let origin = Point(0.1, 0.1);

    let t0 = Tile(
        Kind::T0,
        origin,
        origin + PolarPt(*A, PI - T0.angle_md_rad),
        origin + (-1.0 * *C, 0),
    );

    let t1 = Tile(
        Kind::T1,
        origin,
        origin + PolarPt(*A, -1.0 * T1.angle_base_rad),
        origin + (*C, 0),
    );

    let t2 = Tile(
        Kind::T2,
        origin,
        origin + PolarPt(*B, -1.0 * T1.angle_vertex_rad),
        origin + PolarPt(*C, T1.angle_vertex_rad),
    );

    let mut all_tiles = vec![];

    for (idx, t) in [t0, t1, t2].iter().enumerate() {
        for (jdx, expansion_depth) in (0..3).enumerate() {
            let mut t_copy = *t;

            // centerings
            t_copy.pts.iter_mut().for_each(|pt| {
                pt.rotate_inplace(&Point(0, 0), 0.0 * PI);
                *pt *= (1, -1);
                *pt *= 270.0;
                *pt += (40.0 + 270.0 * (jdx as f64), 150.0 + 150.0 * (idx as f64));
                match t.kind {
                    Kind::T0 => {
                        *pt += (230, 0);
                    }
                    Kind::T1 => {
                        *pt += (-20, 25);
                    }
                    Kind::T2 => {
                        *pt += (0, 110);
                    }
                }
            });

            let mut tiles = vec![t_copy];

            for _ in 0..expansion_depth {
                let next_layer = tiles.iter().flat_map(expand_tile).collect::<Vec<_>>();
                tiles = next_layer;
            }

            all_tiles.extend(tiles);
        }
    }

    let dos: Vec<(Obj2, Style)> = all_tiles
        .into_iter()
        .flat_map(|tile| {
            let color = tile.kind.color();

            let p = Polygon(tile.pts).unwrap();

            let config = ShadeConfig::builder()
                .gap(1.0)
                .slope([-1.0, 0.0, 1.0][tile.kind.as_usize()])
                .switchback(true)
                .build();
            let segments = shade_polygon(&config, &p).unwrap();

            let mut ret: Vec<(Obj2, Style)> = vec![];
            ret.push((Obj2::Polygon(p), Style::default()));
            ret.extend(segments.into_iter().map(|s| {
                (
                    Obj2::Segment(s),
                    Style {
                        color: *color,
                        ..Default::default()
                    },
                )
            }));

            ret
        })
        .collect();

    dos
}
