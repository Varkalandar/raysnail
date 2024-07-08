use {
    crate::{hittable::HitRecord, prelude::*},
    std::sync::Arc,
};

use std::fmt::Debug;
use std::fmt::Formatter;

pub(crate) mod dielectric;
pub(crate) mod isotropic;
pub(crate) mod lambertian;
pub(crate) mod light;
pub(crate) mod metal;
pub(crate) mod blinn_phong;

pub use {
    dielectric::{Dielectric, Glass},
    isotropic::Isotropic,
    lambertian::Lambertian,
    light::DiffuseLight,
    metal::{Metal, DiffuseMetal},
    blinn_phong::BlinnPhong,
};


pub struct ScatterRecord {
    pub color: Color,
    pub ray: Option<Ray>,

    pub pdf: Box::<dyn PDF>,
    pub skip_pdf: bool,
}

impl Debug for ScatterRecord {
    fn fmt(&self, _: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        Ok(())
    }
}

pub trait Material: Send + Sync {

    fn scatter(&self, _ray: &Ray, _hit: &HitRecord<'_>) -> Option<ScatterRecord> {
        None
    }

    fn emitted(&self, _u: f64, _v: f64, _point: &Point3) -> Option<Vec3> {
        None
    }

    fn scattering_pdf(&self, _ray: &Ray, _hit: &HitRecord<'_>, _scattered: &Ray) -> f64 {
        0.0
    }
}

impl<M: Material> Material for Arc<M> {
    fn scatter(&self, ray: &Ray, hit: &HitRecord<'_>) -> Option<ScatterRecord> {
        self.as_ref().scatter(ray, hit)
    }

    fn emitted(&self, u: f64, v: f64, point: &Point3) -> Option<Vec3> {
        self.as_ref().emitted(u, v, point)
    }

    fn scattering_pdf(&self, ray: &Ray, hit: &HitRecord<'_>, scattered: &Ray) -> f64 {
        self.as_ref().scattering_pdf(ray, hit, scattered)
    }
}

pub(crate) fn reflect(ray: &Ray, hit: &HitRecord<'_>) -> Ray {

    assert!((ray.direction.length_squared() - 1.0).abs() < 0.00001);

    let reflected_dir = &ray.direction - 2.0 * ray.direction.dot(&hit.normal) * &hit.normal;
    Ray::new(hit.point.clone(), reflected_dir, ray.departure_time)
}
