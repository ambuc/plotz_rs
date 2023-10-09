use {
    argh::FromArgs,
    plotz_color::*,
    plotz_core::{canvas::Canvas, frame::make_frame, svg::Size},
    plotz_geometry::{
        bounded::Bounded,
        crop::Croppable,
        grid::Grid,
        shapes::{curve::CurveArcs, pt2::Pt2},
        styled_obj2::StyledObj2,
    },
    rand::Rng,
    std::f64::consts::*,
};

#[derive(FromArgs)]
#[argh(description = "...")]
struct Args {
    #[argh(option, description = "output path")]
    output_path_prefix: String,
}

fn main() {
    let args: Args = argh::from_env();

    let mut dos = vec![];
    let mgn = 25.0;

    let mut rng = rand::thread_rng();

    let frame: StyledObj2 = make_frame(
        (1000.0 - 2.0 * mgn, 800.0 - 2.0 * mgn),
        /*offset=*/ Pt2(mgn, mgn),
    );
    {
        let frame_polygon = frame.inner.to_pg2().unwrap();

        let frame_ctr = frame.inner.bbox_center();

        for i in 1..200 {
            let i: f64 = i as f64;

            let ctr = frame_ctr;

            let d = (200.0 - i) / 50.0;
            let angle_1 = 0.0 + d * 3.0 + (rng.gen_range(0.0..d));
            let angle_2 = angle_1 + PI;

            let radius = i * 1.6;

            let cas = CurveArcs(ctr, angle_1..=angle_2, radius);

            dos.extend(
                cas.iter()
                    .flat_map(|ca| ca.crop_to(frame_polygon))
                    .map(|ca| StyledObj2::new(ca).with_color(&GREEN).with_thickness(0.30)),
            );
        }

        dos.extend(
            Grid::builder()
                .width(800)
                .height(1000)
                .build()
                .to_segments(),
        );
    }

    let objs = Canvas::from_objs(
        dos.into_iter().map(|so2| (so2.inner, so2.style)),
        /*autobucket=*/ false,
    )
    .with_frame(frame);

    objs.write_to_svg_or_die(
        Size {
            width: 800,
            height: 1000,
        },
        &args.output_path_prefix,
    )
}
