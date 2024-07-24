use crate::{
    hittable::HitRecord,
    material::{Material, ScatterRecord},
    prelude::*,
    texture::Texture,
};
use crate::material::CommonMaterialSettings;

use std::fmt::Formatter;
use std::fmt::Debug;

#[inline]
fn reflect(ray: &Ray, hit: &HitRecord) -> Ray {

    assert!((ray.direction.length_squared() - 1.0).abs() < 0.00001);

    let reflected_dir = &ray.direction - 2.0 * ray.direction.dot(&hit.normal) * &hit.normal;
    Ray::new(hit.point.clone(), reflected_dir, ray.departure_time)
}


pub struct DiffuseMetal {
    texture: Box<dyn Texture>,
    exponent: f64,
    settings: CommonMaterialSettings,
}

impl Debug for DiffuseMetal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "DiffuseMetal",
        ))
    }
}

impl DiffuseMetal {
    
    /**
     * Smaller exponent values are more diffuse. Can go up to several hundred
     */
    #[must_use]
    pub fn new(exponent: f64, texture: Box<dyn Texture>) -> Self {
        Self {
            exponent,
            texture,
            settings: CommonMaterialSettings::new(),
        }
    }
}


impl Material for DiffuseMetal {
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<ScatterRecord> {
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

    fn settings(&self) -> CommonMaterialSettings {
        self.settings.clone()
    }
}


pub struct Metal {
    texture: Box<dyn Texture>,
    settings: CommonMaterialSettings,
}

impl Debug for Metal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Metal",
        ))
    }
}

impl Metal {
    #[must_use]
    pub fn new(texture: Box<dyn Texture>) -> Self {
        Self {
            texture,
            settings: CommonMaterialSettings::new(),
        }
    }
}

impl Material for Metal {
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<ScatterRecord> {
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
