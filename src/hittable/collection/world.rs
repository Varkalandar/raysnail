use {
    crate::{
        hittable::{
            collection::{HittableList, BVH},
            HitRecord, Hittable,
        },
        prelude::*,
    },
    std::{
        fmt::{Debug, Formatter},
        ops::Range,
    },
};

use std::sync::Arc;

use crate::material::Lambertian;

#[must_use]
pub fn default_background(ray: &Ray) -> Color {
    let t = 0.5 * (ray.direction.y + 1.0);
    Color::new(1.0, 1.0, 1.0, 1.0).gradient(&Color::new(0.5, 0.7, 1.0, 1.0), t)
}

pub struct World {
    bvh: BVH,
    pub lights: HittableList,
    bg_func: Box<dyn Fn(&Ray) -> Color + Send + Sync>,
    pub default_material: Arc<Lambertian>,
}

impl Debug for World {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("World {}")
    }
}

impl World {
    #[must_use]
    pub fn new<F>(list: HittableList, 
                  lights: HittableList, 
                  background: F,
                  time_range: &Range<f64>) -> Self
        where
                F: Fn(&Ray) -> Color + Send + Sync + 'static,
    {
        Self {
            bvh: BVH::new(list, time_range),
            lights,
            bg_func: Box::new(background),
            default_material: Arc::new(Lambertian::new(Arc::new(Color::new(1.0, 1.0, 1.0, 1.0)))),
        }
    }

    #[must_use]
    pub fn background(&self, ray: &Ray) -> Color {
        let f = &self.bg_func;
        f(ray)
    }
}

impl Hittable for World {
    fn hit(&self, ray: &Ray, unit_limit: &Range<f64>) -> Option<HitRecord> {
        self.bvh.hit(ray, unit_limit)
    }

    fn contains(&self, point: &Vec3) -> bool
    {
        self.bvh.contains(point)
    }

    fn bbox(&self, time_limit: &Range<f64>) -> Option<AABB> {
        self.bvh.bbox(time_limit)
    }

    fn random(&self, _origin: &Point3, _rng: &mut FastRng) -> Vec3 {
        Vec3::new(1.0, 0.0, 0.0)
    }
}
