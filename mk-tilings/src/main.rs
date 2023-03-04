// https://tilings.math.uni-bielefeld.de/substitution/danzers-7-fold-original/

use {
    argh::FromArgs,
    lazy_static::lazy_static,
    plotz_color::*,
    plotz_core::{
        draw_obj::{DrawObj, DrawObjs},
        frame::make_frame,
        svg::Size,
    },
    plotz_geometry::{
        point::{PolarPt, Pt},
        polygon::Polygon,
        shading_02::{shade_polygon, ShadeConfig},
    },
    std::f64::consts::PI,
};

static DIM: f64 = 600.0;

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
}

// t0 is a scalene triangle with side lengths (s1, s2, s3) and three kinds of
// interior angles:
static T0_ANGLE_OPP_S1_RAD: f64 = PI / 7.0;
static T0_ANGLE_OPP_S3_RAD: f64 = 4.0 * PI / 7.0;
static T0_ANGLE_OPP_S2_RAD: f64 = PI - T0_ANGLE_OPP_S1_RAD - T0_ANGLE_OPP_S3_RAD;

// t1 is an isosceles triangle, so it has three sides (s1, s3, s3) and two kinds
// of interior angles (vertex angle and base angle).
static T1_BASE_ANGLE_RAD: f64 = 3.0 * PI / 7.0;
static T1_VERTEX_ANGLE_RAD: f64 = PI / 7.0;

// t2 is an isosceles triangle, so it has three sides (s2, s3, s4) and two kinds
// of interior angles (vertex angle and base angle).
#[allow(dead_code)]
static T2_BASE_ANGLE_RAD: f64 = 2.0 * PI / 7.0;
#[allow(dead_code)]
static T2_VERTEX_ANGLE_RAD: f64 = 3.0 * PI / 7.0;

#[derive(FromArgs)]
#[argh(description = "...")]
struct Args {
    #[argh(option, description = "output path")]
    output_path_prefix: String,
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
            Kind::T0 => &BLUE,
            Kind::T1 => &RED,
            Kind::T2 => &GREENYELLOW,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Orientation {
    CW,
    CCW,
}

#[derive(Debug, Clone, Copy)]
struct Tile {
    kind: Kind,
    orientation: Orientation,
    pts: [Pt; 3],
}

#[allow(non_snake_case)]
// Accepts three points in no particular order.
fn Tile(kind: Kind, orientation: Orientation, p1: Pt, p2: Pt, p3: Pt) -> Tile {
    // use orientation here
    Tile {
        kind,
        orientation,
        pts: [p1, p2, p3],
    }
}

impl Tile {
    fn expand(&self) -> Vec<Tile> {
        match self.kind {
            Kind::T0 => {
                let a = self.pts[2];
                let f = self.pts[1];
                let h = self.pts[0];
                let b = a + (h - a) / *B_C_C * *C;
                let d = a + (h - a) / *B_C_C * *B_C;
                let c = a + (f - a) / *A_B_C * *C;
                let e = a + (f - a) / *A_B_C * *B_C;
                let g = f + (h - f) / *A_B * *B;

                vec![
                    Tile(Kind::T1, Orientation::CCW, b, c, a),
                    Tile(Kind::T0, Orientation::CW, c, b, d),
                    Tile(Kind::T2, Orientation::CCW, e, c, d),
                    Tile(Kind::T0, Orientation::CW, f, e, d),
                    Tile(Kind::T2, Orientation::CCW, d, g, f),
                    Tile(Kind::T0, Orientation::CW, h, g, d),
                ]
            }
            Kind::T1 => {
                let a = self.pts[0];
                let h = self.pts[1];
                let d = self.pts[2];
                let b = a + (d - a) / *B_C_C * *C;
                let c = a + (d - a) / *B_C_C * *B_C;
                let g = a + (h - a) / *A_B * *A;
                let i = h + (d - h) / *B_C_C * *C;
                let e = h + (i - h) / *C * *B_C;
                let f = g + (e - g) / *B_C * *B;

                vec![
                    Tile(Kind::T1, Orientation::CCW, f, b, a),
                    Tile(Kind::T0, Orientation::CW, a, g, f),
                    Tile(Kind::T2, Orientation::CCW, f, g, h),
                    Tile(Kind::T1, Orientation::CCW, i, f, h),
                    Tile(Kind::T0, Orientation::CW, f, i, e),
                    Tile(Kind::T0, Orientation::CCW, f, b, c),
                    Tile(Kind::T1, Orientation::CW, c, e, f),
                    Tile(Kind::T1, Orientation::CCW, c, e, d),
                ]
            }
            Kind::T2 => {
                let e = self.pts[0];
                let k = self.pts[1];
                let a = self.pts[2];
                let i = e + (k - e) / *A_B_C * *C;
                let j = e + (k - e) / *A_B_C * *B_C;
                let c = k + (a - k) / *A_B_C * *B_C;
                let h = k + (a - k) / *A_C_C * *C;
                let g = e + (h - e) / *A_B_C * *B_C;
                let f = e + (h - e) / *A_B_C * *C;
                let b = e + (a - e) / *B_C_C * *B_C;
                let d = e + (a - e) / *B_C_C * *C;

                vec![
                    Tile(Kind::T1, Orientation::CW, d, f, e),
                    Tile(Kind::T0, Orientation::CCW, f, d, b),
                    Tile(Kind::T2, Orientation::CW, b, g, f),
                    Tile(Kind::T0, Orientation::CCW, h, g, b),
                    Tile(Kind::T2, Orientation::CW, b, c, h),
                    Tile(Kind::T0, Orientation::CCW, a, c, b),
                    Tile(Kind::T1, Orientation::CW, f, i, e),
                    Tile(Kind::T0, Orientation::CCW, i, f, g),
                    Tile(Kind::T2, Orientation::CW, g, j, i),
                    Tile(Kind::T0, Orientation::CCW, k, j, g),
                    Tile(Kind::T1, Orientation::CW, g, h, k),
                ]
            }
        }
    }
}

fn main() {
    let args: Args = argh::from_env();

    let origin = Pt(0.1, 0.1);

    let t0 = Tile(
        Kind::T0,
        Orientation::CCW,
        origin,
        origin + PolarPt(*A, PI - T0_ANGLE_OPP_S2_RAD),
        origin + Pt(-1.0 * *C, 0.0),
    );

    let t1 = Tile(
        Kind::T1,
        Orientation::CCW,
        origin + PolarPt(*A, -1.0 * T1_BASE_ANGLE_RAD),
        origin,
        origin + Pt(*C, 0.0),
    );

    let t2 = Tile(
        Kind::T2,
        Orientation::CW,
        origin,
        origin + PolarPt(*B, -1.0 * T1_VERTEX_ANGLE_RAD),
        origin + PolarPt(*C, T1_VERTEX_ANGLE_RAD),
    );

    let mut all_tiles = vec![];

    for (idx, t) in [t0, t1, t2].iter().enumerate() {
        for (jdx, expansion_depth) in (0..3).enumerate() {
            //
            let mut t_copy = t.clone();

            // centerings
            t_copy.pts.iter_mut().for_each(|pt| {
                pt.rotate(&Pt(0.0, 0.0), 0.1 * PI);
                *pt = *pt * Pt(1.0, -1.0);
                *pt *= 270.0;
                *pt += Pt(40.0 + 270.0 * (jdx as f64), 150.0 + 150.0 * (idx as f64));
                match t.kind {
                    Kind::T0 => {
                        *pt += Pt(230.0, 0.0);
                    }
                    Kind::T1 => {
                        *pt += Pt(-20.0, 25.0);
                    }
                    Kind::T2 => {
                        *pt += Pt(0.0, 110.0);
                    }
                }
            });

            let mut tiles = vec![];
            tiles.push(t_copy.clone());

            for _ in 0..expansion_depth {
                let next_layer = tiles
                    .iter()
                    .flat_map(|tile| tile.expand())
                    .collect::<Vec<_>>();
                tiles = next_layer;
            }

            all_tiles.extend(tiles);
        }
    }

    let dos: Vec<DrawObj> = all_tiles
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

            let mut ret = vec![];
            ret.push(DrawObj::from_polygon(p.clone()));
            ret.extend(
                segments
                    .into_iter()
                    .map(|s| DrawObj::from_segment(s).with_color(color)),
            );

            ret
        })
        .collect();

    let mut draw_objs = DrawObjs::from_objs(dos).with_frame(make_frame(
        (DIM, DIM * 1.4),
        /*offset=*/ Pt(10.0, 10.0),
    ));

    draw_objs.join_adjacent_segments();

    let () = draw_objs
        .write_to_svg(
            Size {
                width: (750.0 * 1.4) as usize,
                height: 750,
            },
            &args.output_path_prefix,
        )
        .expect("write");
}
