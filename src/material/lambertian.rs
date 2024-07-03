use crate::{
    hittable::HitRecord,
    material::{Material, ScatterRecord},
    prelude::*,
    texture::Texture,
};


#[derive(Debug, Clone)]
pub struct Lambertian<T: Texture> {
    texture: T,
}

impl<T: Texture> Lambertian<T> {
    #[must_use]
    pub fn new(texture: T) -> Self {
        Self {
            texture,
        }
    }
}

impl<T: Texture> Material for Lambertian<T> {

    fn scatter(&self, ray: &Ray, hit: &HitRecord<'_>) -> Option<ScatterRecord> {

        let color = self.texture.color(hit.u, hit.v, &hit.point);

        Some(ScatterRecord {
            color,
            ray: None,
            pdf: Box::new(CosinePdf::new(&hit.normal)), 
            skip_pdf: false,
        })
    }
    
    fn scattering_pdf(&self, ray: &Ray, rec: &HitRecord<'_>, scattered: &Ray) -> f64 {
        let cos_theta = rec.normal.dot(&scattered.direction.unit());        
        
        // println!("cos_theta={}",cos_theta);

        if cos_theta <= 0.0 {0.0} else {cos_theta / PI}
    }        
}
