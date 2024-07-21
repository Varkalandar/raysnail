pub(crate) mod hit;

pub mod collection;
pub mod geometry;
pub mod medium;
pub mod transform;
pub mod csg;

pub use {
    geometry::{AARect, AARectMetrics, Box, Sphere},
    csg::Intersection,
    hit::{HitRecord, Hittable},
};
