use std::fmt::Formatter;
use std::fmt::Debug;

use crate::{
    hittable::HitRecord,
    material::{Material, ScatterRecord},
    prelude::*,
    texture::Texture,
};
use crate::material::CommonMaterialSettings;


// #[derive(Debug, Clone)]
pub struct Lambertian {
    texture: Box<dyn Texture>,
    pub settings: CommonMaterialSettings,
}

impl Debug for Lambertian {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Lambertian: {{ Texture: ? }}"
        ))
    }
}

impl Lambertian {
    #[must_use]
    pub fn new(texture: Box<dyn Texture>) -> Self {
        Self {
            texture,
            settings: CommonMaterialSettings::new(),
        }
    }
}

impl Material for Lambertian {

    fn scatter(&self, _ray: &Ray, hit: &HitRecord) -> Option<ScatterRecord> {

        let color = self.texture.color(hit.u, hit.v, &hit.point);

        Some(ScatterRecord {
            color,
            ray: None,
            pdf: Box::new(CosinePdf::new(&hit.normal)), 
            // pdf: Box::new(CosinePdfExponent::new(&hit.normal, 4000.0)), 
            skip_pdf: false,
        })
    }
    
    fn scattering_pdf(&self, _ray: &Ray, hit: &HitRecord, scattered: &Ray) -> f64 {

        assert!((scattered.direction.length_squared() - 1.0).abs() < 0.00001);
        
        let cos_theta = hit.normal.dot(&scattered.direction);        

        if cos_theta <= 0.0 {0.0} else {cos_theta / PI}
    }        

    fn settings(&self) -> CommonMaterialSettings {
        self.settings.clone()
    }
}
