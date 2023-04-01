use {
    argh::FromArgs,
    plotz_color::*,
    plotz_core::{canvas::Canvas, frame::make_frame, svg::Size},
    plotz_geometry::{
        curve::CurveArc, draw_obj::DrawObj, point::Pt, polygon::Multiline, segment::Segment,
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

#[derive(Debug, PartialEq)]
enum LR {
    Left,
    Right,
}

#[derive(Debug, PartialEq)]
enum Pipe {
    Straight,
    Bend(LR),
    Zag,
}

impl Distribution<Pipe> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Pipe {
        match rng.gen_range(0..=2) {
            0 => Pipe::Straight,
            1 => Pipe::Bend(LR::Right),
            _ => Pipe::Bend(LR::Left),
            //_ => Pipe::Zag,
        }
    }
}

fn main() {
    let args: Args = argh::from_env();

    let image_width: f64 = 500.0;
    let margin = 10.0;

    let mut draw_obj_vec: Vec<DrawObj> = vec![];

    let mut xy: Pt = Pt(3.0, 3.0);
    let mut dxdy: Pt = Pt(1.0, 0.0);

    for _step in 0..=5 {
        let pipe: Pipe = rand::random();

        let (d_o, xy_, dxdy_) = match pipe {
            Pipe::Straight => {
                let xy_ = xy + dxdy;
                (
                    DrawObj::new(Segment(xy, xy_))
                        .with_color(&BLACK)
                        .with_thickness(5.0),
                    xy_,
                    dxdy,
                )
            }
            Pipe::Bend(turn) => {
                let dxdy_ = dxdy.rotate(
                    &Pt(0.0, 0.0),
                    match turn {
                        LR::Left => FRAC_PI_2,
                        LR::Right => -1.0 * FRAC_PI_2,
                    },
                );
                let xy_ = xy + dxdy + dxdy_;

                let ctr = xy + dxdy_;
                let sweep = match (dxdy, dxdy_) {
                    // right
                    (a, b) if a == Pt(1.0, 0.0) && b == Pt(0.0, 1.0) => (3.0 * FRAC_PI_2)..=TAU,
                    (a, b) if a == Pt(0.0, 1.0) && b == Pt(-1.0, 0.0) => (0.0)..=FRAC_PI_2,
                    (a, b) if a == Pt(-1.0, 0.0) && b == Pt(0.0, -1.0) => (FRAC_PI_2)..=(PI),
                    (a, b) if a == Pt(0.0, -1.0) && b == Pt(1.0, 0.0) => (PI)..=(3.0 * FRAC_PI_2),
                    // left
                    (a, b) if a == Pt(1.0, 0.0) && b == Pt(0.0, -1.0) => 0.0..=FRAC_PI_2,
                    (a, b) if a == Pt(0.0, -1.0) && b == Pt(-1.0, 0.0) => 3.0 * FRAC_PI_2..=TAU,
                    (a, b) if a == Pt(-1.0, 0.0) && b == Pt(0.0, 1.0) => PI..=(3.0 * FRAC_PI_2),
                    (a, b) if a == Pt(0.0, 1.0) && b == Pt(1.0, 0.0) => FRAC_PI_2..=PI,
                    (a, b) => {
                        dbg!(a, b);
                        unimplemented!("uhoh")
                    }
                };

                (
                    DrawObj::new(CurveArc(ctr, sweep, 1.0))
                        .with_color(match turn {
                            LR::Left => &BLACK,
                            LR::Right => &BLACK,
                        })
                        .with_thickness(5.0),
                    xy_,
                    dxdy_,
                )
            }
            Pipe::Zag => {
                let xy_ = xy + dxdy;

                let d_o = DrawObj::new(
                    Multiline(vec![
                        xy,
                        xy + (dxdy) * 0.25,
                        (xy + (dxdy * 0.5)).rotate(&xy, FRAC_PI_6),
                        (xy + (dxdy * 0.5)).rotate(&(xy + dxdy), FRAC_PI_6),
                        xy + dxdy * 0.75,
                        xy + dxdy * 1.0,
                    ])
                    .unwrap(),
                )
                .with_color(&YELLOW)
                .with_thickness(5.0);
                (d_o, xy_, dxdy)
            }
        };

        xy = xy_;
        dxdy = dxdy_;
        draw_obj_vec.push(d_o);
    }

    let mut canvas = Canvas::from_objs(draw_obj_vec)
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
