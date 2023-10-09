# plotz_rs

A collection of pen plotter libraries, binaries, and utilities.

```mermaid
graph TD;
    src/main.rs-->plotz_core;
    plotz_core-->plotz_color;
    plotz_core-->plotz_geometry;
    plotz_core-->plotz_geojson;
    plotz_geojson-->plotz_geometry;
    plotz_geometry3d-->plotz_color;
    plotz_geometry3d-->plotz_geometry;

    mk_foo-->plotz_core;
    mk_foo-->plotz_geometry;
    mk_foo-->plotz_color;
    mk_foo-->plotz_geometry3d;
```

## Authors

See [`Cargo.toml`](Cargo.toml).

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) .

## Development

See [`DEVELOPMENT.md`](DEVELOPMENT.md).

## License

This project is licensed under the Apache 2.0 license.

## Disclaimer

This is not an official Google product.
