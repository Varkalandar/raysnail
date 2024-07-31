use crate::{
    hittable::{HitRecord, Hittable},
    material::Isotropic,
    prelude::*,
};

use std::ops::Range;
use std::sync::Arc;
use std::fmt::Formatter;
use std::fmt::Debug;
use crate::material::Material;

pub struct ConstantMedium<T> {
    boundary: T,
    material: Arc<dyn Material>,
    density: f64,
    neg_inv_density: f64,
}

impl<T> Debug for ConstantMedium<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "ConstantMedium",
        ))
    }
}


impl<T> ConstantMedium<T> {
    #[must_use]
    pub fn new(boundary: T, color: Color, density: f64) -> Self {
        Self {
            boundary,
            material: Arc::new(Isotropic::new(color)),
            density,
            neg_inv_density: -1.0 / density,
        }
    }
}

impl<T: Hittable> Hittable for ConstantMedium<T> {
    fn hit(
        &self, ray: &Ray, unit_limit: &std::ops::Range<f64>,
    ) -> Option<crate::hittable::HitRecord> {
        let mut rec1 = self.boundary.hit(ray, &(f64::NEG_INFINITY..f64::INFINITY))?;
        let mut rec2 = self.boundary.hit(ray, &(rec1.t1 + 0.0001..f64::INFINITY))?;
        if rec1.t1 < unit_limit.start {
            rec1.t1 = unit_limit.start;
        }
        if rec2.t1 > unit_limit.end {
            rec2.t1 = unit_limit.end;
        }
        if rec1.t1 >= rec2.t1 {
            return None;
        }
        if rec1.t1 < 0.0 {
            rec1.t1 = 0.0;
        }

        let length_per_unit = ray.direction.length();
        let distance_inside = (rec2.t1 - rec1.t1) * length_per_unit;
        let hit_distance = self.neg_inv_density * Random::normal().ln();

        if hit_distance > distance_inside {
            return None;
        }

        let hit_point_unit = rec1.t1 + hit_distance / length_per_unit;

        Some(HitRecord {
            point: ray.at(hit_point_unit),
            normal: Vec3::new(1.0, 0.0, 0.0), // useless,
            material: Some(self.material.clone()),
            t1: hit_point_unit,
            t2: hit_point_unit,
            u: 0.0,         // useless
            v: 0.0,         // useless
            outside: false, // useless
        })
    }

    fn contains(&self, point: &Vec3) -> bool
    {
        unimplemented!(
            "{}'s constains function is not yet implemented",
            std::any::type_name::<Self>()
        )
    }

    fn bbox(&self, time_limit: &Range<f64>) -> Option<AABB> {
        self.boundary.bbox(time_limit)
    }

    fn random(&self, _origin: &Point3, _rng: &mut FastRng) -> Vec3 {
        Vec3::new(1.0, 0.0, 0.0)
    }
}
