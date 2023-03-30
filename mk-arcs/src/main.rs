use plotz_geometry::bounded::Bounded;

use {
    argh::FromArgs,
    plotz_color::*,
    plotz_core::{canvas::Canvas, frame::make_frame, svg::Size},
    plotz_geometry::{
        crop::Croppable, curve::CurveArcs, draw_obj::DrawObj, draw_obj_inner::DrawObjInner,
        point::Pt,
    },
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
    let mgn = 100.0; // 20.0;
    let ell = 500.0; // 720.0;
    let asp = 1.3;

    let frame: DrawObj = make_frame((ell, ell * asp), /*offset=*/ Pt(mgn, mgn));
    {
        let frame_polygon = match frame.obj {
            DrawObjInner::Polygon(ref pg) => pg.clone(),
            _ => unimplemented!(),
        };
        let frame_ctr = frame.obj.bbox_center();
        for i in 1..300 {
            let i: f64 = i as f64;

            let ctr = frame_ctr;

            let angle_1 = 0.0 + 0.1 * i;
            let angle_2 = 1.2 * angle_1 + FRAC_PI_2;

            let radius = 1.0 + 2.0 * i.powf(0.95);

            let cas = CurveArcs(ctr, angle_1..=angle_2, radius);

            // dos.push(DrawObj::new(ca).with_color(&RED).with_thickness(1.0));

            dos.extend(
                cas.iter()
                    .flat_map(|ca| ca.crop_to(&frame_polygon).unwrap())
                    .into_iter()
                    .map(|ca| DrawObj::new(ca).with_color(&GREEN).with_thickness(1.0))
                    .into_iter(),
            );
        }
    }

    let draw_objs = Canvas::from_objs(dos).with_frame(frame);

    let () = draw_objs
        .write_to_svg(
            Size {
                width: (750.0 * 1.3) as usize,
                height: 750,
            },
            &args.output_path_prefix,
        )
        .expect("write");
}
