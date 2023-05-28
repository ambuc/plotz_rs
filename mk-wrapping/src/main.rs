use {
    argh::FromArgs,
    plotz_color::*,
    plotz_core::{canvas::Canvas, frame::make_frame, svg::Size},
    plotz_geometry::{
        obj2::Obj2,
        p2,
        shapes::{curve::CurveArc, pt2::Pt2, sg2::Sg2},
        styled_obj2::StyledObj2,
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
    fn to_dos(&self) -> Vec<StyledObj2> {
        self.to_dois()
            .into_iter()
            .map(|doi| StyledObj2::new(doi).with_color(&BLACK).with_thickness(2.0))
            .collect::<Vec<_>>()
    }

    // scaled to a unit square.
    fn to_dois(&self) -> Vec<Obj2> {
        let _a = p2!(0.0, 0.0);
        let _b = p2!(0.25, 0.0);
        let c = p2!(0.5, 0.0);
        let _d = p2!(0.75, 0.0);
        let _e = p2!(1.0, 0.0);
        let _f = p2!(0.0, 0.25);
        let g = p2!(0.25, 0.25);
        let h = p2!(0.5, 0.25);
        let i = p2!(0.75, 0.25);
        let _j = p2!(1.0, 0.25);
        let k = p2!(0.0, 0.5);
        let l = p2!(0.25, 0.5);
        let _m = p2!(0.5, 0.5);
        let n = p2!(0.75, 0.5);
        let o = p2!(1.0, 0.5);
        let _p = p2!(0.0, 0.75);
        let q = p2!(0.25, 0.75);
        let r = p2!(0.5, 0.75);
        let s = p2!(0.75, 0.75);
        let _t = p2!(1.0, 0.75);
        let _u = p2!(0.0, 1.0);
        let _v = p2!(0.25, 1.0);
        let w = p2!(0.5, 1.0);
        let _x = p2!(0.75, 1.0);
        let _y = p2!(1.0, 1.0);
        match self {
            Tile::Cross => {
                vec![Sg2(k, o).into(), Sg2(c, w).into()]
            }
            Tile::OverUnder => {
                vec![Sg2(k, l).into(), Sg2(n, o).into(), Sg2(c, w).into()]
            }
            Tile::Swerve => {
                vec![
                    Sg2(k, l).into(),
                    CurveArc(g, 0.0..=FRAC_PI_2, 0.25).into(),
                    Sg2(c, h).into(),
                    Sg2(n, o).into(),
                    CurveArc(s, PI..=(3.0 * FRAC_PI_2), 0.25).into(),
                    Sg2(r, w).into(),
                ]
            }
            Tile::Clover => {
                vec![
                    Sg2(c, h).into(),
                    Sg2(k, l).into(),
                    Sg2(n, o).into(),
                    Sg2(r, w).into(),
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
                    Sg2(k, o).into(),
                    Sg2(c, h).into(),
                    Sg2(r, w).into(),
                    CurveArc(g, 0.0..=FRAC_PI_2, 0.25).into(),
                    CurveArc(q, (3.0 * FRAC_PI_2)..=TAU, 0.25).into(),
                ]
            }
            Tile::Clover3 => {
                vec![
                    Sg2(c, h).into(),
                    Sg2(k, l).into(),
                    Sg2(n, o).into(),
                    Sg2(r, w).into(),
                    CurveArc(g, 0.0..=FRAC_PI_2, 0.25).into(),
                    CurveArc(i, FRAC_PI_2..=PI, 0.25).into(),
                    CurveArc(s, PI..=(3.0 * FRAC_PI_2), 0.25).into(),
                ]
            }
            Tile::Clover2 => {
                vec![
                    Sg2(c, w).into(),
                    Sg2(k, l).into(),
                    Sg2(n, o).into(),
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

    let mut obj_vec: Vec<StyledObj2> = vec![];

    let width = 10;
    let height = 10;
    for dx in 0..=width {
        for dy in 0..=height {
            let tile: Tile = rand::random();
            obj_vec.extend(tile.to_dos().into_iter().map(|d_o| d_o + p2!(dx, dy)));
        }
    }

    let mut canvas = Canvas::from_objs(obj_vec.into_iter(), /*autobucket=*/ false)
        .with_frame(make_frame((image_width, image_width), p2!(margin, margin)));

    canvas.scale_to_fit_frame().unwrap();

    canvas.write_to_svg_or_die(
        Size {
            width: (image_width + 2.0 * margin) as usize,
            height: (image_width + 2.0 * margin) as usize,
        },
        &args.output_path_prefix,
    );
}
