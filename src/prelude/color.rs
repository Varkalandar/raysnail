use {
    crate::{
        prelude::{clamp, vec3::Point3},
        texture::Texture,
    },
    std::{borrow::Cow, ops::Mul},
};

macro_rules! check0to1 {
    ($r: ident, $g: ident, $b: ident) => {
        debug_assert!((0.0_f64..=1.0_f64).contains(&$r), "r = {}", $r);
        debug_assert!((0.0_f64..=1.0_f64).contains(&$g), "g = {}", $g);
        debug_assert!((0.0_f64..=1.0_f64).contains(&$b), "b = {}", $b);
    };
}

#[derive(Debug, Clone, Default)]
pub struct RGBFloat {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl RGBFloat {
    pub fn new(r: f64, g: f64, b: f64) -> Self {
        check0to1!(r, g, b);
        Self { r, g, b }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RGBInt {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RGBInt {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

impl From<&RGBFloat> for RGBInt {
    fn from(c: &RGBFloat) -> Self {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        // because RGBFloat r g b should be in [0..1]
        Self::new(
            (c.r * 255.0) as u8,
            (c.g * 255.0) as u8,
            (c.b * 255.0) as u8,
        )
    }
}

impl From<&RGBInt> for RGBFloat {
    fn from(c: &RGBInt) -> Self {
        let s = 1.0 / 255.0;
        Self::new(f64::from(c.r) * s, f64::from(c.g) * s, f64::from(c.b) * s)
    }
}

#[derive(Debug, Clone)]
pub enum Color {
    Float(RGBFloat),
    Int(RGBInt),
}

impl Default for Color {
    fn default() -> Self {
        Self::Float(RGBFloat::default())
    }
}

impl Color {
    #[must_use]
    pub const fn new_int(r: u8, g: u8, b: u8) -> Self {
        Self::Int(RGBInt::new(r, g, b))
    }

    #[must_use]
    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Self::Float(RGBFloat::new(r, g, b))
    }

    #[must_use]
    pub fn i(&self) -> Cow<'_, RGBInt> {
        match self {
            Self::Float(c) => Cow::Owned(c.into()),
            Self::Int(c) => Cow::Borrowed(c),
        }
    }

    #[must_use]
    pub fn f(&self) -> Cow<'_, RGBFloat> {
        match self {
            Self::Float(c) => Cow::Borrowed(c),
            Self::Int(c) => Cow::Owned(c.into()),
        }
    }

    #[must_use]
    pub fn gradient(&self, rhs: &Self, slide: f64) -> Self {
        let a = slide.max(0.0).min(1.0);
        let b = 1.0 - a;

        let c1 = self.f();
        let c2 = rhs.f();

        Self::new(c1.r * b + c2.r * a, 
                  c1.g * b + c2.g * a,
                  c1.b * b + c2.b * a)
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
        let c1 = self.f();
        let c2 = rhs.f();
        Color::new(c1.r * c2.r, c1.g * c2.g, c1.b * c2.b)
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

impl Mul<f64> for &Color {
    type Output = Color;
    fn mul(self, rhs: f64) -> Self::Output {
        let c = self.f();
        Color::new(
            clamp(c.r * rhs, 0.0..=1.0),
            clamp(c.g * rhs, 0.0..=1.0),
            clamp(c.b * rhs, 0.0..=1.0),
        )
    }
}

impl Mul<f64> for Color {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        &self * rhs
    }
}

impl Mul<&Color> for f64 {
    type Output = Color;
    fn mul(self, rhs: &Color) -> Self::Output {
        rhs * self
    }
}
impl Mul<Color> for f64 {
    type Output = Color;
    fn mul(self, rhs: Color) -> Self::Output {
        &rhs * self
    }
}
