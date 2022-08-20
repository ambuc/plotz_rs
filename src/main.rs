use argh::FromArgs;
use glob::glob;
use plotz_core::map::MapConfig;
use std::path::PathBuf;

#[derive(FromArgs)]
#[argh(description = "...")]
struct Args {
    #[argh(option, description = "all geojson")]
    input_glob: String,
    #[argh(option, description = "output file prefix")]
    output_file_prefix: String,
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
        args.output_file_prefix,
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

    #[test]
    fn test_main_inner() {
        //
    }
}
