use {
    crate::{material::Material, prelude::*},
    std::{
        fmt::{Debug, Formatter},
        ops::Range,
    },
};
use std::sync::Arc;

#[derive(Clone)]
pub struct HitRecord {
    pub point: Point3,
    pub normal: Vec3,
    pub material: Option<Arc<dyn Material>>,
    pub t1: f64,
    pub t2: f64,
    pub u: f64,
    pub v: f64,
    pub outside: bool,
}

impl Debug for HitRecord {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "HitRecord {{ t1: {}, t2: {}, hit: {:?}, normal: {:?}, outside: {} }}",
            self.t1, self.t2, self.point, self.normal, self.outside
        ))
    }
}

impl HitRecord {
    pub fn new<G: Hittable>(ray: &Ray, object: &G, t1: f64, t2: f64) -> Self {
        let point = ray.at(t1);

        let mut normal = object.normal(&point);
        let outside = ray.direction.dot(&normal) < 0.0;
        if !outside {
            normal.reverse();
        }

        let material = object.material();
        let (u, v) = object.uv(&point);
        Self {
            point,
            normal,
            material,
            t1,
            t2,
            u,
            v,
            outside,
        }
    }

    pub fn with_normal(point: Point3, normal: Vec3, material: Option<Arc<dyn Material>>, uv: (f64, f64), t1: f64, t2: f64) -> Self {

        Self {
            point,
            normal,
            material,
            t1,
            t2,
            u: uv.0,
            v: uv.1,
            outside: true,
        }
    }
}

#[allow(unused_variables)]
pub trait Hittable: Send + Sync {
    fn normal(&self, _point: &Point3) -> Vec3 {
        unimplemented!(
            "{}'s normal function should not be called directly",
            std::any::type_name::<Self>()
        )
    }
    fn material(&self) -> Option<Arc<dyn Material>> {
        unimplemented!(
            "{}'s material function should not be called directly",
            std::any::type_name::<Self>()
        )
    }
    fn uv(&self, point: &Point3) -> (f64, f64) {
        unimplemented!(
            "{}'s uv function should not be called directly",
            std::any::type_name::<Self>()
        )
    }

    fn hit(&self, ray: &Ray, unit_limit: &Range<f64>) -> Option<HitRecord>;

    fn contains(&self, point: &Vec3) -> bool;

    fn bbox(&self, time_limit: &Range<f64>) -> Option<AABB>;

    /**
     * This is only called if the object is a light source. It is used to generate
     * an extra ray towards the light source.
     */
     fn random(&self, origin: &Point3, rng: &mut FastRng) -> Vec3;
}
