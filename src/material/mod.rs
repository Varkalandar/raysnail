use {
    crate::{hittable::HitRecord, prelude::*},
    std::sync::Arc,
};

pub(crate) mod dielectric;
pub(crate) mod isotropic;
pub(crate) mod lambertian;
pub(crate) mod light;
pub(crate) mod metal;

pub use {
    dielectric::{Dielectric, Glass},
    isotropic::Isotropic,
    lambertian::Lambertian,
    light::DiffuseLight,
    metal::Metal,
};

#[derive(Debug)]
pub struct ScatterRecord {
    pub color: Color,
    pub ray: Ray,
}

pub trait Material: Send + Sync {
    fn scatter(&self, ray: &Ray, hit: &HitRecord<'_>) -> Option<ScatterRecord>;

    #[allow(unused_variables)]
    fn emitted(&self, u: f64, v: f64, point: &Point3) -> Option<Vec3> {
        None
    }

    fn scattering_pdf(&self, ray: &Ray, rec: &HitRecord<'_>, scattered: &Ray) -> f64 {
        let v = scattered.direction.unit();        
        let cos_theta = rec.normal.dot(&v);        
        if cos_theta < 0.0 {0.0} else {cos_theta / PI}
    }    
}

impl<M: Material> Material for Arc<M> {
    fn scatter(&self, ray: &Ray, hit: &HitRecord<'_>) -> Option<ScatterRecord> {
        self.as_ref().scatter(ray, hit)
    }

    fn emitted(&self, u: f64, v: f64, point: &Point3) -> Option<Vec3> {
        self.as_ref().emitted(u, v, point)
    }
}

pub(crate) fn reflect(ray: &Ray, hit: &HitRecord<'_>) -> Ray {
    let dir = ray.direction.unit();
    let reflected_dir = &dir - 2.0 * dir.dot(&hit.normal) * &hit.normal;
    Ray::new(hit.point.clone(), reflected_dir, ray.departure_time)
}
