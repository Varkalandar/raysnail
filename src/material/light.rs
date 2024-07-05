use crate::{
    material::Material,
    prelude::*,
    texture::Texture,
};

#[derive(Debug, Clone)]
pub struct DiffuseLight<T> {
    texture: T,
    multiplier: f64,
}

impl<T> DiffuseLight<T> {
    pub const fn new(texture: T) -> Self {
        Self {
            texture,
            multiplier: 1.0,
        }
    }

    pub const fn multiplier(mut self, multiplier: f64) -> Self {
        self.multiplier = multiplier;
        self
    }
}

impl<T: Texture> Material for DiffuseLight<T> {

    fn emitted(&self, u: f64, v: f64, point: &Point3) -> Option<Vec3> {
        Some(<Color as Into<Vec3>>::into(self.texture.color(u, v, point)) * self.multiplier)
    }

    /*
    fn scattering_pdf(&self, ray: &Ray, rec: &HitRecord<'_>, scattered: &Ray) -> f64 {
        1.0 / (4.0 * PI)
    }
    */       
}
