use crate::{
    hittable::HitRecord,
    material::{Material, ScatterRecord},
    prelude::*,
    texture::Texture,
};


#[derive(Debug, Clone)]
pub struct BlinnPhong<T: Texture> {
    texture: T,
    k_specular: f64,
    exponent: f64,
}

impl<T: Texture> BlinnPhong<T> {
    #[must_use]
    pub fn new(k_specular: f64, exponent: f64, texture: T) -> Self {
        Self {
            texture,
            exponent,
            k_specular,
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
        //let cosine = hit.normal.dot(&scattered.direction.unit());
        
        let half = (-&ray.direction.unit() + scattered.direction.unit()).unit();
        
        let cos_theta = half.dot(&hit.normal).max(0.0);
        let specular = cos_theta.powf(self.exponent);

        ((((1.0 - self.k_specular) * cos_theta).max(0.0)  
            + (self.k_specular * specular).max(0.0))) * 0.30
    }
}
