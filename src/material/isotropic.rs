use crate::{
    material::{Material, ScatterRecord, HitRecord},
    prelude::*,
};


#[derive(Debug, Clone)]
pub struct Isotropic {
    color: Color,
}

impl Isotropic {
    #[must_use]
    pub const fn new(color: Color) -> Self {
        Self { color }
    }
}

impl Material for Isotropic {
    fn scatter(&self, ray: &Ray, hit: &HitRecord<'_>) -> Option<ScatterRecord> {
        let scattered_ray = Ray::new(hit.point.clone(), Vec3::random_in_unit_sphere(), ray.departure_time);
        Some(ScatterRecord {
            ray: scattered_ray,
            color: self.color.clone(),
        })
    }

    fn scattering_pdf(&self, ray: &Ray, rec: &HitRecord<'_>, scattered: &Ray) -> f64 {
        1.0 / (4.0 * PI)
    }
}
