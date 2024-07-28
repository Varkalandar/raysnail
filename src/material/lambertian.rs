use std::fmt::Formatter;
use std::fmt::Debug;
use std::sync::Arc;

use crate::{
    hittable::HitRecord,
    material::{Material, ScatterRecord},
    prelude::*,
    texture::Texture,
};
use crate::material::CommonMaterialSettings;


pub struct Lambertian {
    texture: Arc<dyn Texture>,
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
    pub fn new(texture: Arc<dyn Texture>) -> Self {
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

    fn settings(&self) -> CommonMaterialSettings {
        self.settings.clone()
    }

    fn set(&mut self, settings: CommonMaterialSettings) {

        println!("lambertian.set(), settings={{phong: {}, phong_size: {}}}", settings.phong_factor, settings.phong_exponent);

        self.settings = settings;
    }
}
