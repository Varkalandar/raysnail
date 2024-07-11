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

    fn scatter(&self, _ray: &Ray, hit: &HitRecord<'_>) -> Option<ScatterRecord> {

        let color = self.texture.color(hit.u, hit.v, &hit.point);

        Some(ScatterRecord {
            color,
            ray: None,
            pdf: Box::new(CosinePdf::new(&hit.normal)), 
            // pdf: Box::new(CosinePdfExponent::new(&hit.normal, 4000.0)), 
            skip_pdf: false,
        })
    }
    
    fn scattering_pdf(&self, _ray: &Ray, hit: &HitRecord<'_>, scattered: &Ray) -> f64 {

        assert!((scattered.direction.length_squared() - 1.0).abs() < 0.00001);
        
        let cos_theta = hit.normal.dot(&scattered.direction);        

        if cos_theta <= 0.0 {0.0} else {cos_theta / PI}
    }        
}
