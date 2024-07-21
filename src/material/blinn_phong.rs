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

    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<ScatterRecord> {

        let color = self.texture.color(hit.u, hit.v, &hit.point);

        Some(ScatterRecord {
            color,
            ray: None,
            pdf: Box::new(BlinnPhongPdf::new(ray.direction.clone(), hit.normal.clone(), self.k_specular, self.exponent)), 
            skip_pdf: false,
        })
    }

    fn settings(&self) -> CommonMaterialSettings {
        self.settings.clone()
    }
}
