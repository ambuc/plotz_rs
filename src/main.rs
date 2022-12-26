#![deny(missing_docs)]

//! The point of entry for plotz. Call this executable to parse geojson to svg.

use argh::FromArgs;
use glob::glob;
use plotz_core::{
    map::{Map, MapConfig},
    svg::Size,
};
use tracing::*;
use tracing_subscriber::FmtSubscriber;

#[derive(FromArgs, Debug)]
#[argh(description = "...")]
struct Args {
    #[argh(option, description = "all geojson")]
    input_glob: String,
    #[argh(option, description = "output file prefix")]
    output_directory: std::path::PathBuf,
    #[argh(option, description = "width")]
    width: usize,
    #[argh(option, description = "height")]
    height: usize,
    #[argh(switch, description = "draw frame")]
    draw_frame: bool,
    #[argh(option, description = "scale factor", default = "0.9")]
    scale_factor: f64,
    #[argh(option, description = "shift x", default = "0.0")]
    shift_x: f64,
    #[argh(option, description = "shift y", default = "0.0")]
    shift_y: f64,
}

fn main() {
    let subscriber = FmtSubscriber::builder()
        .compact()
        .with_max_level(Level::TRACE)
        .without_time()
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let args: Args = argh::from_env();
    main_inner(args);
}

fn main_inner(args: Args) {
    let map_config = MapConfig::builder()
        .input_files(
            glob(&args.input_glob)
                .expect("failed to read glob pattern")
                .into_iter()
                .collect::<Result<Vec<_>, _>>()
                .unwrap(),
        )
        .output_directory(args.output_directory)
        .size(Size {
            width: args.width,
            height: args.height,
        })
        .draw_frame(args.draw_frame)
        .scale_factor(args.scale_factor)
        .shift_x(args.shift_x)
        .shift_y(args.shift_y)
        .build();

    let map = Map::new(&map_config).expect("failed to create map");

    let () = map.render(&map_config).expect("failed to render map");
}

#[cfg(test)]
mod test_super {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use tempdir::TempDir;

    fn write_svg_to_pixmap(size: Size, svg: &str) -> tiny_skia::Pixmap {
        let usvg_options = usvg::Options::default();
        let svg_tree = usvg::Tree::from_str(svg, &usvg_options.to_ref()).expect("invalid svg");
        let mut actual_png =
            tiny_skia::Pixmap::new(size.width as u32, size.height as u32).expect("make pixmap");
        assert!(resvg::render(
            &svg_tree,
            usvg::FitTo::Original,
            tiny_skia::Transform::identity(),
            actual_png.as_mut()
        )
        .is_some());
        actual_png
    }

    #[test]
    fn test_main_inner() {
        let tmp_dir = TempDir::new("tmp").unwrap();
        let size = Size {
            width: 1024,
            height: 1024,
        };

        main_inner(Args {
            input_glob: "testdata/wuppertal*.geojson".to_string(),
            output_directory: tmp_dir.path().to_path_buf(),
            width: size.width,
            height: size.height,
            draw_frame: true,
            scale_factor: 0.9,
            shift_x: 0.0,
            shift_y: 0.0,
        });

        let output_svg = std::fs::read_to_string(tmp_dir.path().join("0.svg")).expect("foo");

        assert_eq!(
            {
                let mut s = DefaultHasher::new();
                output_svg.hash(&mut s);
                s.finish()
            },
            11932351637306413113
        );

        assert!(write_svg_to_pixmap(size, &output_svg)
            .save_png("/tmp/output.png")
            .is_ok());
    }
}
