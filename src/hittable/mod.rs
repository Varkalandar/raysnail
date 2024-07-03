pub(crate) mod hit;

pub mod collection;
pub mod geometry;
pub mod medium;
pub mod transform;

pub use {
    geometry::{AARect, AARectMetrics, Box, Sphere},
    hit::{HitRecord, Hittable},
};
