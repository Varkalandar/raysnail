use crate::{
    material::{Material, ScatterRecord, HitRecord},
    prelude::*,
};
use crate::material::CommonMaterialSettings;


#[derive(Debug, Clone)]
pub struct Isotropic {
    color: Color,
    settings: CommonMaterialSettings,
}

impl Isotropic {
    #[must_use]
    pub fn new(color: Color) -> Self {
        Self {
            color,
            settings: CommonMaterialSettings::new(),
        }
    }
}

impl Material for Isotropic {

    fn scatter(&self, _ray: &Ray, _hit: &HitRecord<'_>) -> Option<ScatterRecord> {
        Some(ScatterRecord {
            color: self.color.clone(),
            ray: None,
            pdf: Box::new(SpherePdf::new()), 
            skip_pdf: false,            
        })
    }

    fn scattering_pdf(&self, _ray: &Ray, _rec: &HitRecord<'_>, _scattered: &Ray) -> f64 {
        1.0 / (4.0 * PI)
    }

    fn settings(&self) -> CommonMaterialSettings {
        self.settings.clone()
    }
}
