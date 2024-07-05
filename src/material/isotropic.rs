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
        Some(ScatterRecord {
            color: self.color.clone(),
            ray: None,
            pdf: Box::new(SpherePdf::new()), 
            skip_pdf: false,            
        })
    }

    fn scattering_pdf(&self, ray: &Ray, rec: &HitRecord<'_>, scattered: &Ray) -> f64 {
        1.0 / (4.0 * PI)
    }
}
