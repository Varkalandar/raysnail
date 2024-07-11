use {
    crate::{material::Material, prelude::*},
    std::{
        fmt::{Debug, Formatter},
        ops::Range,
    },
};

pub struct HitRecord<'m> {
    pub point: Point3,
    pub normal: Vec3,
    pub material: &'m dyn Material,
    pub t1: f64,
    pub t2: f64,
    pub u: f64,
    pub v: f64,
    pub outside: bool,
}

impl Debug for HitRecord<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "HitRecord {{ t1: {}, t2: {}, hit: {:?}, normal: {:?}, outside: {} }}",
            self.t1, self.t2, self.point, self.normal, self.outside
        ))
    }
}

impl<'m> HitRecord<'m> {
    pub fn new<G: Hittable>(ray: &Ray, object: &'m G, t1: f64, t2: f64) -> Self {
        let point = ray.position_after(t1);

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
}

#[allow(unused_variables)]
pub trait Hittable: Send + Sync {
    fn normal(&self, _point: &Point3) -> Vec3 {
        unimplemented!(
            "{}'s normal function should not be called directly",
            std::any::type_name::<Self>()
        )
    }
    fn material(&self) -> &dyn Material {
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

    fn hit(&self, ray: &Ray, unit_limit: &Range<f64>) -> Option<HitRecord<'_>>;
    fn bbox(&self, time_limit: &Range<f64>) -> Option<AABB>;

    /**
     * This is only called if the object is a light source. It is used to generate
     * an extra ray towards the light source.
     */
     fn random(&self, origin: &Point3, rng: &mut FastRng) -> Vec3;
}
