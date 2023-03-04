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

#[derive(Debug, Clone, Copy)]
struct Tile {
    kind: Kind,
    p1: Pt,
    p2: Pt,
    p3: Pt,
}

#[allow(non_snake_case)]
// Accepts three points in no particular order.
fn Tile(kind: Kind, p1: Pt, p2: Pt, p3: Pt) -> Tile {
    Tile { kind, p1, p2, p3 }
}

impl Tile {
    fn pts(&self) -> Vec<Pt> {
        vec![self.p1, self.p2, self.p3]
    }

    fn rotate(&mut self, about: &Pt, by: f64) {
        self.p1.rotate(about, by);
        self.p2.rotate(about, by);
        self.p3.rotate(about, by);
    }

    fn ctr(&self) -> Pt {
        Pt(
            (self.p1.x.0 + self.p2.x.0 + self.p3.x.0) / 3.0,
            (self.p1.y.0 + self.p2.y.0 + self.p3.y.0) / 3.0,
        )
    }
}

impl Tile {
    fn expand(&self) -> Vec<Tile> {
        match self.kind {
            Kind::T0 => {
                let a = self.p3;
                let f = self.p2;
                let h = self.p1;
                let b = a + (h - a) / *B_C_C * *C;
                let d = a + (h - a) / *B_C_C * *B_C;
                let c = a + (f - a) / *A_B_C * *C;
                let e = a + (f - a) / *A_B_C * *B_C;
                let g = f + (h - f) / *A_B * *B;

                vec![
                    Tile(Kind::T1, b, c, a),
                    Tile(Kind::T0, c, b, d),
                    Tile(Kind::T2, d, e, c),
                    Tile(Kind::T0, f, e, d),
                    Tile(Kind::T2, d, g, f),
                    Tile(Kind::T0, h, g, d),
                ]
            }
            Kind::T1 => {
                let a = self.p1;
                let h = self.p2;
                let d = self.p3;
                let b = a + (d - a) / *B_C_C * *C;
                let c = a + (d - a) / *B_C_C * *B_C;
                let g = a + (h - a) / *A_B * *A;
                let i = h + (d - h) / *B_C_C * *C;
                let e = h + (i - h) / *C * *B_C;
                let f = g + (e - g) / *B_C * *B;

                vec![
                    Tile(Kind::T1, f, b, a),
                    Tile(Kind::T0, a, g, f),
                    Tile(Kind::T2, f, g, h),
                    Tile(Kind::T1, i, f, h),
                    Tile(Kind::T0, f, i, e),
                    Tile(Kind::T0, f, b, c),
                    Tile(Kind::T1, e, c, f),
                    Tile(Kind::T1, c, e, d),
                ]
            }
            Kind::T2 => {
                let e = self.p1;
                let k = self.p2;
                let a = self.p3;
                let i = e + (k - e) / *A_B_C * *C;
                let j = e + (k - e) / *A_B_C * *B_C;
                let c = k + (a - k) / *A_B_C * *B_C;
                let h = k + (a - k) / *A_C_C * *C;
                let g = e + (h - e) / *A_B_C * *B_C;
                let f = e + (h - e) / *A_B_C * *C;
                let b = e + (a - e) / *B_C_C * *B_C;
                let d = e + (a - e) / *B_C_C * *C;

                vec![
                    Tile(Kind::T1, f, d, e),
                    Tile(Kind::T0, f, d, b),
                    Tile(Kind::T2, f, g, b),
                    Tile(Kind::T0, h, g, b),
                    Tile(Kind::T2, h, c, b),
                    Tile(Kind::T0, a, c, b),
                    Tile(Kind::T1, i, f, e),
                    Tile(Kind::T0, i, f, g),
                    Tile(Kind::T2, i, j, g),
                    Tile(Kind::T0, k, j, g),
                    Tile(Kind::T1, h, g, k),
                ]
            }
        }
    }
}

fn main() {
    let args: Args = argh::from_env();

    let origin = Pt(0.1, 0.1);

    let t0a = origin;
    let t0b = t0a + PolarPt(*A, PI - T0_ANGLE_OPP_S2_RAD);
    let t0c = t0a + Pt(-1.0 * *C, 0.0);
    let t0 = Tile(Kind::T0, t0a, t0b, t0c);

    let t1a = origin;
    let t1b = t1a + PolarPt(*A, -1.0 * T1_BASE_ANGLE_RAD);
    let t1c = t1a + Pt(*C, 0.0);
    let t1 = Tile(Kind::T1, t1a, t1b, t1c);

    let t2a = origin;
    let t2b = t2a + PolarPt(*B, -1.0 * T1_VERTEX_ANGLE_RAD);
    let t2c = t2a + PolarPt(*C, T1_VERTEX_ANGLE_RAD);
    let t2 = Tile(Kind::T2, t2a, t2b, t2c);

    let mut t_copy = t2;
    t_copy.rotate(&Pt(0.0, 0.0), 0.1 * PI);

    let mut tiles = vec![];
    tiles.push(t_copy.clone());

    for _ in 0..3 {
        let next_layer = tiles
            .iter()
            .flat_map(|tile| tile.expand())
            .collect::<Vec<_>>();
        tiles = next_layer;
    }

    let dos: Vec<DrawObj> = tiles
        .into_iter()
        .flat_map(|tile| {
            let color = match tile.kind {
                Kind::T0 => &BLUE,
                Kind::T1 => &RED,
                Kind::T2 => &YELLOWGREEN,
            };
            let p = Polygon(tile.pts().into_iter()).unwrap();

            vec![
                DrawObj::from_polygon(p.clone()).with_color(color),
                // DrawObj::from_segment(Segment(tile.p1, tile.p2)).with_color(color),
                // DrawObj::from_pt(tile.ctr()).with_color(color),
            ]
        })
        .collect();

    let mut draw_objs = DrawObjs::from_objs(dos).with_frame(make_frame((DIM, DIM), Pt(50.0, 50.0)));

    // invert
    draw_objs.mutate(|pt| {
        *pt = *pt * Pt(1.0, -1.0);
        *pt *= 760.0;
        *pt += Pt(5.0, 660.0);
    });

    // scale

    let () = draw_objs
        .write_to_svg(
            Size {
                width: 750,
                height: 750,
            },
            &args.output_path_prefix,
        )
        .expect("write");
}
