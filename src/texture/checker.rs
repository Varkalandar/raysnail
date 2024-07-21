use crate::{
    prelude::{Color, Point3},
    texture::Texture,
};

#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct Checker<T1, T2> {
    odd: T1,
    even: T2,
    scale: f64,
}

impl<T1, T2> Checker<T1, T2> {
    #[must_use]
    pub const fn new(odd: T1, even: T2, scale: f64) -> Self {
        Self { odd, even, scale }
    }
}

impl<T1: Texture, T2: Texture> Texture for Checker<T1, T2> {
    fn color(&self, u: f64, v: f64, point: &Point3) -> Color {
        let value = (self.scale * point.x).sin() * (self.scale * point.y).sin() * (self.scale * point.z).sin();
        if value < 0.0 {
            self.odd.color(u, v, point)
        } else {
            self.even.color(u, v, point)
        }
    }
}
