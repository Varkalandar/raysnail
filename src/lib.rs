#![deny(clippy::all, clippy::pedantic, clippy::nursery)]
#![deny(missing_debug_implementations, rust_2018_idioms)]
// #![deny(warnings)]
#![allow(clippy::module_name_repetitions, clippy::cast_possible_truncation)]

pub mod camera;
pub mod hittable;
pub mod material;
pub mod painter;
pub mod prelude;
pub mod texture;
pub mod sdl_parser;
