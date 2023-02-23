# plotz_rs

A collection of pen plotter libraries, binaries, and utilities.

## Binaries

* [`orchestrator`](orchestrator/README.md), a tool for managing long-running prints.
* [`svg-splitter`](svg-splitter/README.md), a tool for splitting very large SVG files which already have inherent groupings.
* [`mk-voronoi`](mk-voronoi/README.md), a tool for making voronoi patterns.

## Libraries

* [`plotz-color`](plotz-color/README.md), a library with lots of useful predefined RGB colors.
* [`plotz-geojson`](plotz-geojson/README.md), a library for parsing GeoJSON into `plotz-geometry` types.
* [`plotz-geometry`](plotz-geometry/README.md), a library for 2D geometry.

## Questions?

Please file an issue on GitHub.

## Authors

See [`Cargo.toml`](Cargo.toml).

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) and [`NOTES.md`](NOTES.md)

## Commands

Some useful cargo commands for working with a repo with many crates:

* `cargo build --workspace --color=always`
* `cargo test --release --workspace -- --nocapture --color=always`
* `RUST_LOG=info cargo run --release -- --input-glob testdata/wuppertal.geojson --output-directory "/tmp/" --width 1024 --height 1024`

## License

This project is licensed under the Apache 2.0 license.

## Disclaimer

This is not an official Google product.
