# Development

Some useful cargo commands for working with a repo with many crates:

## Build

```bash
cargo build --workspace --color=always
```

## Test

```bash
cargo test --release --workspace -- --nocapture --color=always
cargo nextest run --workspace --color=always --release
```

## Run

```bash
# Running the main mapper in src/main.rs
RUST_LOG=info cargo run --release -- --input-glob testdata/wuppertal.geojson --output-directory "/tmp/" --width 1024 --height 1024

# Running an example image; in mk-flowfield, for example
cargo run --release -p mk-flowifeld -- --output-path-prefix /tmp/foo
```