use crate::{
    hittable::HitRecord,
    material::{Material, ScatterRecord},
    prelude::*,
    texture::Texture,
};
use crate::material::CommonMaterialSettings;


#[inline]
fn reflect(ray: &Ray, hit: &HitRecord<'_>) -> Ray {

    assert!((ray.direction.length_squared() - 1.0).abs() < 0.00001);

    let reflected_dir = &ray.direction - 2.0 * ray.direction.dot(&hit.normal) * &hit.normal;
    Ray::new(hit.point.clone(), reflected_dir, ray.departure_time)
}


#[derive(Debug, Clone)]
pub struct DiffuseMetal<T: Texture> {
    texture: T,
    exponent: f64,
    settings: CommonMaterialSettings,
}

impl<T: Texture> DiffuseMetal<T> {
    
    /**
     * Smaller exponent values are more diffuse. Can go up to several hundred
     */
    #[must_use]
    pub fn new(exponent: f64, texture: T) -> Self {
        Self {
            exponent,
            texture,
            settings: CommonMaterialSettings::new(),
        }
    }
}


impl<T: Texture> Material for DiffuseMetal<T> {
    fn scatter(&self, ray: &Ray, hit: &HitRecord<'_>) -> Option<ScatterRecord> {
        let color = self.texture.color(hit.u, hit.v, &hit.point);
        let reflected = reflect(ray, &hit);
        
        if reflected.direction.dot(&hit.normal) > 0.0 {
            Some(ScatterRecord {
                color,
                ray: Some(reflected),
                pdf: Box::new(ReflectionPdf::new(ray.direction.clone(), hit.normal.clone(), self.exponent)),
                skip_pdf: false,            
            })
        } else {
            None
        }
    }

    fn scattering_pdf(&self, _ray: &Ray, rec: &HitRecord<'_>, scattered: &Ray) -> f64 {

        assert!((scattered.direction.length_squared() - 1.0).abs() < 0.00001);

        let cos_theta = rec.normal.dot(&scattered.direction);

        // println!("cos_theta={}",cos_theta);

        if cos_theta <= 0.0 {0.0} else {cos_theta / PI}
    }        

    fn settings(&self) -> CommonMaterialSettings {
        self.settings.clone()
    }
}


#[derive(Debug, Clone)]
pub struct Metal<T: Texture> {
    texture: T,
    settings: CommonMaterialSettings,
}

impl<T: Texture> Metal<T> {
    #[must_use]
    pub fn new(texture: T) -> Self {
        Self {
            texture,
            settings: CommonMaterialSettings::new(),
        }
    }
}

impl<T: Texture> Material for Metal<T> {
    fn scatter(&self, ray: &Ray, hit: &HitRecord<'_>) -> Option<ScatterRecord> {
        let color = self.texture.color(hit.u, hit.v, &hit.point);
        let reflected = reflect(ray, &hit);
        
        if reflected.direction.dot(&hit.normal) > 0.0 {
            Some(ScatterRecord {
                color,
                ray: Some(reflected),
                pdf: Box::new(CosinePdf::new(&hit.normal)), 
                skip_pdf: true,            
            })
        } else {
            None
        }
    }

    fn settings(&self) -> CommonMaterialSettings {
        self.settings.clone()
    }
}