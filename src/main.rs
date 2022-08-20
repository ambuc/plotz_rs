#![deny(missing_docs)]

//! The point of entry for plotz. Call this executable to parse geojson to svg.

use argh::FromArgs;
use glob::glob;
use plotz_core::map::MapConfig;

#[derive(FromArgs)]
#[argh(description = "...")]
struct Args {
    #[argh(option, description = "all geojson")]
    input_glob: String,
    #[argh(option, description = "output file prefix")]
    output_directory: std::path::PathBuf,
}

fn main() {
    let args: Args = argh::from_env();
    main_inner(args);
}

fn main_inner(args: Args) {
    MapConfig::new_from_files(
        /*files=*/
        glob(&args.input_glob)
            .expect("failed to read glob pattern")
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap(),
        args.output_directory,
    )
    .expect("failed to produce MapConfig")
    .make_map()
    .expect("failed to create map")
    .render()
    .expect("failed to render map");
}

#[cfg(test)]
mod test_super {
    use super::*;
    use tempdir::TempDir;

    fn write_svg_to_pixmap((width, height): (u32, u32), svg: &str) -> tiny_skia::Pixmap {
        let usvg_options = usvg::Options::default();
        let svg_tree = usvg::Tree::from_str(&svg, &usvg_options.to_ref()).expect("invalid svg");
        let mut actual_png = tiny_skia::Pixmap::new(width, height).expect("make pixmap");
        assert!(resvg::render(
            &svg_tree,
            usvg::FitTo::Original,
            tiny_skia::Transform::identity(),
            actual_png.as_mut()
        )
        .is_some());
        assert!(actual_png
            .save_png("/Users/jamesbuckland/Desktop/output.png")
            .is_ok());
        actual_png
    }

    #[test]
    fn test_main_inner() {
        let tmp_dir = TempDir::new("tmp").unwrap();

        let args = Args {
            input_glob: "testdata/wuppertal*.geojson".to_string(),
            output_directory: tmp_dir.path().to_path_buf(),
        };

        main_inner(args);

        let output_svg = std::fs::read_to_string(tmp_dir.path().join("0.svg")).expect("foo");
        println!("{}", output_svg);

        // TODO(ambuc): make w/h adjustable.
        let png = write_svg_to_pixmap((1024, 1024), &output_svg);
    }
}
