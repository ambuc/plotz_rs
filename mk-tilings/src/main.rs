// https://tilings.math.uni-bielefeld.de/substitution/danzers-7-fold-original/

use plotz_core::draw_obj::DrawObjInner;
use plotz_geometry::bounded::Bounded;

use {
    argh::FromArgs,
    float_cmp::approx_eq,
    float_ord::FloatOrd,
    lazy_static::lazy_static,
    plotz_color::{BLUE, RED, YELLOW},
    plotz_core::{
        draw_obj::{DrawObj, DrawObjs},
        frame::make_frame,
        svg::Size,
    },
    plotz_geometry::{point::PolarPt, segment::Segment},
    plotz_geometry::{
        point::Pt,
        polygon::Polygon,
        shading_02::{shade_polygon, ShadeConfig},
    },
    std::{cmp::min, f64::consts::PI},
};

static DIM: f64 = 600.0;

lazy_static! {
    static ref S1: f64 = (PI / 7.0).sin();
    static ref S2: f64 = (2.0 * PI / 7.0).sin();
    static ref S3: f64 = (3.0 * PI / 7.0).sin();
    // static ref SF: f64 = 1.0 + ((2.0 * PI / 7.0).sin() / (PI / 7.0).sin());
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
enum Ori {
    CW,
    CCW,
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
    orientation: Ori,
    p1: Pt,
    p2: Pt,
    p3: Pt,
}

#[allow(non_snake_case)]
// Accepts three points in no particular order or orientation.
fn Tile(kind: Kind, orientation: Ori, p1: Pt, p2: Pt, p3: Pt) -> Tile {
    // let l12 = p1.dist(&p2);
    // let l23 = p2.dist(&p3);
    // let l13 = p1.dist(&p3);

    // assert that the points are given in a canonical order.
    // assert!(l12 < l23 || approx_eq!(f64, l12, l23));
    // assert!(l23 < l13 || approx_eq!(f64, l23, l13));

    // if isosceles, assert ccw
    // if (approx_eq!(f64, l12, l23) || approx_eq!(f64, l23, l13)) {
    //     assert!(Segment(p1, p2).cross_z(&Segment(p2, p3)) > 0.0);
    //     assert!(Segment(p2, p3).cross_z(&Segment(p3, p1)) > 0.0);
    // }

    Tile {
        kind,
        orientation,
        p1,
        p2,
        p3,
    }
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
}

impl Tile {
    fn expand(&self) -> Vec<Tile> {
        match self.kind {
            Kind::T0 => {
                let a = self.p3;
                let f = self.p2;
                let h = self.p1;
                let b = a + (h - a) / (*S3 + *S2 + *S3) * (*S3);
                let d = a + (h - a) / (*S3 + *S2 + *S3) * (*S3 + *S3);
                let c = a + (f - a) / (*S1 + *S2 + *S3) * (*S3);
                let e = a + (f - a) / (*S1 + *S2 + *S3) * (*S3 + *S2);
                let g = f + (h - f) / (*S1 + *S2) * (*S2);

                vec![
                    Tile(Kind::T1, Ori::CCW, b, c, a),
                    Tile(Kind::T0, Ori::CW, c, b, d),
                    Tile(Kind::T2, Ori::CCW, d, e, c),
                    Tile(Kind::T0, Ori::CW, f, e, d),
                    Tile(Kind::T2, Ori::CCW, d, g, f),
                    Tile(Kind::T0, Ori::CCW, h, g, d),
                ]
            }
            Kind::T1 => {
                let a = self.p1;
                let h = self.p2;
                let d = self.p3;
                let b = a + (d - a) / (*S3 + *S2 + *S3) * *S3;
                let f = a + (b - a).and_rotate(&Pt(0.0, 0.0), -1.0 * T1_VERTEX_ANGLE_RAD);
                let g = a + (h - a) / (*S2 + *S1) * *S1;
                let i = h + ((d - h) / (*S3 + *S2 + *S3) * (*S3));
                let e = h + ((i - h) / (*S3) * (*S2 + *S3));
                let c = a + (d - a) / (*S3 + *S2 + *S3) * (*S3 + *S2);

                vec![
                    Tile(Kind::T1, Ori::CW, f, b, a),
                    Tile(Kind::T0, Ori::CW, a, g, f),
                    Tile(Kind::T2, Ori::CCW, f, g, h),
                    Tile(Kind::T1, Ori::CCW, i, f, h),
                    Tile(Kind::T0, Ori::CW, f, i, e),
                    Tile(Kind::T0, Ori::CCW, f, b, c),
                    Tile(Kind::T1, Ori::CW, e, c, f),
                    Tile(Kind::T1, Ori::CCW, c, e, d),
                ]
            }
            Kind::T2 => {
                let e = self.p1;
                let k = self.p2;
                let a = self.p3;
                let i = e + (k - e) / (*S3 + *S2 + *S1) * (*S3);
                let f = e + (i - e).and_rotate(&Pt(0.0, 0.0), T1_VERTEX_ANGLE_RAD);
                let d = e + (i - e).and_rotate(&Pt(0.0, 0.0), 2.0 * T1_VERTEX_ANGLE_RAD);
                let b = e + (d - e) / *S3 * (*S3 + *S2);
                let g = e + (f - e) / *S3 * (*S3 + *S2);
                let h = e + (f - e) / *S3 * (*S3 + *S2 + *S1);
                let j = e + (k - e) / (*S3 + *S2 + *S1) * (*S3 + *S2);
                let c =
                    b + (a - b).and_rotate(&Pt(0.0, 0.0), -1.0 * T0_ANGLE_OPP_S1_RAD) / *S3 * *S2;

                vec![
                    Tile(Kind::T1, Ori::CW, f, d, e),
                    Tile(Kind::T0, Ori::CCW, f, d, b),
                    Tile(Kind::T2, Ori::CW, f, g, b),
                    Tile(Kind::T0, Ori::CCW, h, g, b),
                    Tile(Kind::T2, Ori::CW, h, c, b),
                    Tile(Kind::T0, Ori::CCW, a, c, b),
                    Tile(Kind::T1, Ori::CW, i, f, e),
                    Tile(Kind::T0, Ori::CCW, i, f, g),
                    Tile(Kind::T2, Ori::CW, i, j, g),
                    Tile(Kind::T0, Ori::CCW, k, j, g),
                    Tile(Kind::T1, Ori::CW, h, g, k),
                ]
            }
        }
    }
}

fn main() {
    let args: Args = argh::from_env();

    let origin = Pt(0.0, 0.0);

    let t0a = origin;
    let t0b = t0a + PolarPt(*S1, PI - T0_ANGLE_OPP_S2_RAD);
    let t0c = t0a + Pt(-1.0 * *S3, 0.0);
    let t0 = Tile(Kind::T0, Ori::CW, t0a, t0b, t0c);

    let t1a = origin;
    let t1b = t1a + PolarPt(*S1, -1.0 * T1_BASE_ANGLE_RAD);
    let t1c = t1a + Pt(*S3, 0.0);
    let t1 = Tile(Kind::T1, Ori::CW, t1a, t1b, t1c);

    let t2a = origin;
    let t2b = t2a + PolarPt(*S2, -1.0 * T1_VERTEX_ANGLE_RAD);
    let t2c = t2a + PolarPt(*S3, T1_VERTEX_ANGLE_RAD);
    let t2 = Tile(Kind::T2, Ori::CCW, t2a, t2b, t2c);

    let mut t_copy = t2;
    t_copy.rotate(&Pt(0.0, 0.0), 0.0 * PI);

    let mut tiles = vec![];
    tiles.push(t_copy.clone());

    for _ in 0..2 {
        tiles = tiles
            .into_iter()
            .map(|tile| tile.expand())
            .flatten()
            .collect();
    }

    let mut dos: Vec<DrawObj> = tiles
        .into_iter()
        .map(|tile| {
            DrawObj::from_polygon(Polygon(tile.pts().into_iter()).unwrap()).with_color(
                match tile.kind {
                    Kind::T0 => &BLUE,
                    Kind::T1 => &RED,
                    Kind::T2 => &YELLOW,
                },
            )
        })
        .collect();

    let mut draw_objs = DrawObjs::from_objs(dos).with_frame(make_frame((DIM, DIM), Pt(50.0, 50.0)));

    let width = draw_objs.width();
    let height = draw_objs.height();

    // invert
    draw_objs.mutate(|pt| {
        *pt = *pt * Pt(1.0, -1.0);
        *pt *= 600.0;
        *pt += Pt(100.0, 400.0);
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
