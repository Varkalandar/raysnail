use crate::{
    hittable::HitRecord,
    material::{Material, ScatterRecord},
    prelude::*,
    texture::Texture,
};
use crate::material::CommonMaterialSettings;


#[derive(Debug, Clone)]
pub struct BlinnPhong<T: Texture> {
    texture: T,
    k_specular: f64,
    exponent: f64,
    settings: CommonMaterialSettings,
}

impl<T: Texture> BlinnPhong<T> {
    #[must_use]
    pub fn new(k_specular: f64, exponent: f64, texture: T) -> Self {
        Self {
            texture,
            exponent,
            k_specular,
            settings: CommonMaterialSettings::new(),
        }
    }
}

impl<T: Texture> Material for BlinnPhong<T> {

    fn scatter(&self, ray: &Ray, hit: &HitRecord<'_>) -> Option<ScatterRecord> {

        let color = self.texture.color(hit.u, hit.v, &hit.point);

        Some(ScatterRecord {
            color,
            ray: None,
            pdf: Box::new(BlinnPhongPdf::new(ray.direction.clone(), hit.normal.clone(), self.k_specular, self.exponent)), 
            skip_pdf: false,
        })
    }
    
    fn scattering_pdf(&self, ray: &Ray, hit: &HitRecord<'_>, scattered: &Ray) -> f64 {

        assert!((ray.direction.length_squared() - 1.0).abs() < 0.00001);
        assert!((scattered.direction.length_squared() - 1.0).abs() < 0.00001);

        let half = (-&ray.direction + &scattered.direction).unit();
        
        let cos_theta = half.dot(&hit.normal).max(0.0);
        let specular = cos_theta.powf(self.exponent);

        ((((1.0 - self.k_specular) * cos_theta).max(0.0)  
            + (self.k_specular * specular).max(0.0))) * 0.5 / PI
    }

    fn settings(&self) -> CommonMaterialSettings {
        self.settings.clone()
    }
}
