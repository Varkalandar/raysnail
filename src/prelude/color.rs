use {
    crate::{
        prelude::{vec3::Point3},
        texture::Texture,
    },
    std::ops::Mul,
};


#[derive(Debug, Clone)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for Color {
    fn default() -> Self {
        Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        }
    }
}

impl Color {
    #[must_use]
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color {
            r,
            g,
            b,
            a,
        }
    }

    pub fn new64(r: f64, g: f64, b: f64, a: f64) -> Self {
        Color {
            r: r as f32,
            g: g as f32,
            b: b as f32,
            a: a as f32,
        }
    }

    #[must_use]
    pub fn gradient(&self, rhs: &Self, slide: f64) -> Self {
        let a = slide.max(0.0).min(1.0) as f32;
        let b = 1.0 - a;

        Self::new(self.r * b + rhs.r * a, 
                  self.g * b + rhs.g * a,
                  self.b * b + rhs.b * a,
                  1.0)
    }
}

impl Texture for Color {
    fn color(&self, _u: f64, _v: f64, _point: &Point3) -> Color {
        self.clone()
    }
}

impl Mul<&Color> for &Color {
    type Output = Color;
    fn mul(self, rhs: &Color) -> Self::Output {
        Color::new(self.r * rhs.r, self.g * rhs.g, self.b * rhs.b, self.a * rhs.a)
    }
}

impl Mul<Color> for &Color {
    type Output = Color;
    fn mul(self, rhs: Color) -> Self::Output {
        self * &rhs
    }
}

impl Mul<&Color> for Color {
    type Output = Self;
    fn mul(self, rhs: &Self) -> Self::Output {
        &self * rhs
    }
}

impl Mul<Color> for Color {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        &self * &rhs
    }
}

impl Mul<f64> for Color {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Color::new(
            self.r * rhs as f32,
            self.g * rhs as f32,
            self.b * rhs as f32,
            self.a
        )
    }
}

impl Mul<&Color> for f64 {
    type Output = Color;
    fn mul(self, rhs: &Color) -> Self::Output {
        rhs.clone() * self
    }
}
impl Mul<Color> for f64 {
    type Output = Color;
    fn mul(self, rhs: Color) -> Self::Output {
        rhs * self
    }
}
