
use {
    argh::FromArgs,
    plotz_color::BLACK,
    plotz_core::{
        colored_obj::{ColoredObj, Obj},
        svg::{write_layer_to_svg, Size},
    },
    plotz_geometry::{point::Pt, polygon::Polygon, shading_02::{shade_polygon, ShadeConfig}},
    rand::Rng,
};

#[derive(FromArgs)]
#[argh(description = "...")]
struct Args {
    #[argh(option, description = "output path")]
    output_path: String,
}

fn main() {
    let args: Args = argh::from_env();
    //
    println!("args");

    let mut rng = rand::thread_rng();

    let sites = (0..50)
        .step_by(1)
        .map(|_| {
            let x: f64 = rng.gen();
            let y: f64 = rng.gen();
            voronoice::Point { x, y }
        })
        .collect();

    let vornoi = voronoice::VoronoiBuilder::default()
        .set_sites(sites)
        .set_bounding_box(voronoice::BoundingBox::new(
            voronoice::Point { x: 0.5, y: 0.5 },
            1.0,
            1.0,
        ))
        .set_lloyd_relaxation_iterations(5)
        .build()
        .expect("build vornoi");

    // inspect cells through iterators
    let mut polygons: Vec<Polygon> = vornoi
        .iter_cells()
        .map(|cell| {
            Polygon(cell.iter_vertices().map(|vertex| Pt(vertex.x, vertex.y)))
                .expect("valid polygon")
        })
        .collect();

    polygons.iter_mut().for_each(|p| {
        *p *= 400.0;
        *p += Pt(50.0, 50.0);
    });

    let colored_objs: Vec<ColoredObj> = polygons
        .into_iter()
        .map(|p| {
            let shade_config = ShadeConfig {
                gap: 10.0,
                slope: rng.gen_range(-2.0, 2.0),
                thickness: 1.0,
            };

            let segments = shade_polygon(&shade_config, &p).expect("failed to shade");

            let mut colored_objects: Vec<ColoredObj> = segments
                .into_iter()
                .map(|segment| ColoredObj {
                    obj: Obj::Segment(segment),
                    color: BLACK,
                    thickness: 1.0,
                })
                .collect();

            colored_objects.push(ColoredObj {
                obj: Obj::Polygon(p),
                color: BLACK,
                thickness: 1.0,
            });

            colored_objects
        })
        .flatten()
        .collect();

    let num = write_layer_to_svg(
        Size {
            width: 500,
            height: 500,
        },
        &args.output_path,
        &colored_objs,
    )
    .expect("failed to write");

    println!("Wrote {:?} lines", num);
}
