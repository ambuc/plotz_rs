#![deny(missing_docs)]

//! The core mapping logic of plotz, including coloring and bucketing.

pub mod map;

mod bucket;
mod bucketer;
mod colored_polygon;
mod colorer;
mod colorer_builder;
mod svg;
