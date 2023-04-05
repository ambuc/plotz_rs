use plotz_geometry::draw_obj_inner::DrawObjInner;
use {
    argh::FromArgs,
    plotz_color::*,
    plotz_core::{canvas::Canvas, frame::make_frame, svg::Size},
    plotz_geometry::{
        curve::CurveArc,
        draw_obj::DrawObj,
        point::Pt,
        polygon::{Multiline, Rect},
        segment::Segment,
    },
    rand::{distributions::Standard, prelude::Distribution, Rng},
    std::f64::consts::*,
};

#[derive(FromArgs)]
#[argh(description = "...")]
struct Args {
    #[argh(option, description = "output path")]
    output_path_prefix: String,
}

impl Distribution<Tile> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Tile {
        match rng.gen_range(0..=6) {
            0 => Tile::Cross,
            1 => Tile::OverUnder,
            2 => Tile::Swerve,
            3 => Tile::Clover,
            4 => Tile::CloverIn,
            5 => Tile::Clover3,
            _ => Tile::Clover2,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum Tile {
    Cross,
    OverUnder,
    Swerve,
    Clover,
    CloverIn,
    Clover3,
    Clover2,
}
impl Tile {
    fn to_dos(&self) -> Vec<DrawObj> {
        self.to_dois()
            .into_iter()
            .map(|doi| DrawObj::new(doi).with_color(&BLACK).with_thickness(2.0))
            .collect::<Vec<_>>()
    }

    // scaled to a unit square.
    fn to_dois(&self) -> Vec<DrawObjInner> {
        let a = Pt(0.0, 0.0);
        let b = Pt(0.25, 0.0);
        let c = Pt(0.5, 0.0);
        let d = Pt(0.75, 0.0);
        let e = Pt(1.0, 0.0);
        let f = Pt(0.0, 0.25);
        let g = Pt(0.25, 0.25);
        let h = Pt(0.5, 0.25);
        let i = Pt(0.75, 0.25);
        let j = Pt(1.0, 0.25);
        let k = Pt(0.0, 0.5);
        let l = Pt(0.25, 0.5);
        let m = Pt(0.5, 0.5);
        let n = Pt(0.75, 0.5);
        let o = Pt(1.0, 0.5);
        let p = Pt(0.0, 0.75);
        let q = Pt(0.25, 0.75);
        let r = Pt(0.5, 0.75);
        let s = Pt(0.75, 0.75);
        let t = Pt(1.0, 0.75);
        let u = Pt(0.0, 1.0);
        let v = Pt(0.25, 1.0);
        let w = Pt(0.5, 1.0);
        let x = Pt(0.75, 1.0);
        let y = Pt(1.0, 1.0);
        match self {
            Tile::Cross => {
                vec![Segment(k, o).into(), Segment(c, w).into()]
            }
            Tile::OverUnder => {
                vec![
                    Segment(k, l).into(),
                    Segment(n, o).into(),
                    Segment(c, w).into(),
                ]
            }
            Tile::Swerve => {
                vec![
                    Segment(k, l).into(),
                    CurveArc(g, 0.0..=FRAC_PI_2, 0.25).into(),
                    Segment(c, h).into(),
                    Segment(n, o).into(),
                    CurveArc(s, PI..=(3.0 * FRAC_PI_2), 0.25).into(),
                    Segment(r, w).into(),
                ]
            }
            Tile::Clover => {
                vec![
                    Segment(c, h).into(),
                    Segment(k, l).into(),
                    Segment(n, o).into(),
                    Segment(r, w).into(),
                    CurveArc(g, 0.0..=FRAC_PI_2, 0.25).into(),
                    CurveArc(i, FRAC_PI_2..=PI, 0.25).into(),
                    CurveArc(s, PI..=(3.0 * FRAC_PI_2), 0.25).into(),
                    CurveArc(q, (3.0 * FRAC_PI_2)..=TAU, 0.25).into(),
                ]
                //
            }
            Tile::CloverIn => {
                vec![
                    //
                    Segment(k, o).into(),
                    Segment(c, h).into(),
                    Segment(r, w).into(),
                    CurveArc(g, 0.0..=FRAC_PI_2, 0.25).into(),
                    CurveArc(q, (3.0 * FRAC_PI_2)..=TAU, 0.25).into(),
                ]
            }
            Tile::Clover3 => {
                vec![
                    Segment(c, h).into(),
                    Segment(k, l).into(),
                    Segment(n, o).into(),
                    Segment(r, w).into(),
                    CurveArc(g, 0.0..=FRAC_PI_2, 0.25).into(),
                    CurveArc(i, FRAC_PI_2..=PI, 0.25).into(),
                    CurveArc(s, PI..=(3.0 * FRAC_PI_2), 0.25).into(),
                ]
            }
            Tile::Clover2 => {
                vec![
                    Segment(c, w).into(),
                    Segment(k, l).into(),
                    Segment(n, o).into(),
                    CurveArc(g, 0.0..=FRAC_PI_2, 0.25).into(),
                    CurveArc(s, PI..=(3.0 * FRAC_PI_2), 0.25).into(),
                ]
            }
        }
    }
}

fn main() {
    let args: Args = argh::from_env();

    let image_width: f64 = 500.0;
    let margin = 10.0;

    let mut draw_obj_vec: Vec<DrawObj> = vec![];

    let width = 10;
    let height = 10;
    for dx in 0..=width {
        for dy in 0..=height {
            let tile: Tile = rand::random();
            draw_obj_vec.extend(tile.to_dos().into_iter().map(|d_o| d_o + Pt(dx, dy)));
        }
    }

    let mut canvas = Canvas::from_objs(draw_obj_vec, /*autobucket=*/ false)
        .with_frame(make_frame((image_width, image_width), Pt(margin, margin)));

    canvas.scale_to_fit_frame().unwrap();

    let () = canvas
        .write_to_svg(
            Size {
                width: (image_width + 2.0 * margin) as usize,
                height: (image_width + 2.0 * margin) as usize,
            },
            &args.output_path_prefix,
        )
        .expect("write");
}
