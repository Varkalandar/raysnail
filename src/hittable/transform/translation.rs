use crate::{
    hittable::{HitRecord, Hittable},
    prelude::*,
};

use std::ops::Range;

#[derive(Debug, Clone)]
pub struct Translation<T> {
    object: T,
    movement: Vec3,
}

impl<T> Translation<T> {
    pub const fn new(object: T, movement: Vec3) -> Self {
        Self { object, movement }
    }
}

impl<T: Hittable> Hittable for Translation<T> {
    fn hit(&self, ray: &Ray, unit_limit: &Range<f64>) -> Option<HitRecord> {
        let moved_ray = Ray::new(
            &ray.origin - &self.movement,
            ray.direction.clone(),
            ray.departure_time,
        );
        self.object.hit(&moved_ray, unit_limit).map(|mut record| {
            record.point += &self.movement;
            record
        })
    }

    fn bbox(&self, time_limit: &Range<f64>) -> Option<AABB> {
        self.object
            .bbox(time_limit)
            .map(|bbox| AABB::new(bbox.min() + &self.movement, bbox.max() + &self.movement))
    }

    fn random(&self, origin: &Point3, rng: &mut FastRng) -> Vec3 {
        self.object.random(origin, rng)
    }
}
