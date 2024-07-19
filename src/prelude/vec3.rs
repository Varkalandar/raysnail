use {
    crate::prelude::{clamp, Color, FastRng, PI},
    std::{
        fmt::Display,
        iter::Sum,
        ops::{
            Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Range, Sub,
            SubAssign,
        },
    },
};


#[derive(Default, Clone, Debug, PartialOrd, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

pub type Point3 = Vec3;

impl Vec3 {
    #[must_use]
    #[inline(always)]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    #[must_use]
    #[inline]
    pub fn new_min(a: &Self, b: &Self) -> Self {
        Self {
            x: a.x.min(b.x),
            y: a.y.min(b.y),
            z: a.z.min(b.z),
        }
    }

    #[must_use]
    #[inline]
    pub fn new_max(a: &Self, b: &Self) -> Self {
        Self {
            x: a.x.max(b.x),
            y: a.y.max(b.y),
            z: a.z.max(b.z),
        }
    }

    #[must_use]
    pub fn random_in_unit_box(rng: &mut FastRng) -> Self {
        Self::new(
            rng.range(-1.0, 1.0),
            rng.range(-1.0, 1.0),
            rng.range(-1.0, 1.0),
        )
    }

    #[must_use]
    pub fn random_range(r: Range<f64>, rng: &mut FastRng) -> Self {
        Self::new(
            rng.range(r.start, r.end),
            rng.range(r.start, r.end),
            rng.range(r.start, r.end),
        )
    }

    #[must_use]
    pub fn random_in_unit_sphere(rng: &mut FastRng) -> Self {
        loop {
            let p = Self::random_in_unit_box(rng);
            if p.length_squared() < 1.0 {
                return p;
            }
        }
    }

    #[must_use]
    #[inline]
    pub fn random_in_unit_hemisphere(dir: &Self, rng: &mut FastRng) -> Self {
        let u = Self::random_in_unit_sphere(rng);
        if u.dot(dir) > 0.0 {
            u
        } else {
            -u
        }
    }

    #[must_use]
    #[inline]
    pub fn random_unit(rng: &mut FastRng) -> Self {
        let a: f64 = rng.range(0.0, 2.0 * PI);
        let z: f64 = rng.range(-1.0, 1.0);
        let r = (1.0 - z * z).sqrt();
        Self::new(r * a.cos(), r * a.sin(), z)
    }


    #[inline(always)]
    pub fn random_cosine_direction(rng: &mut FastRng) -> Self {
        let r1 = rng.gen();
        let r2 = rng.gen();
        let q2 = r2.sqrt();

        let phi = 2.0 * PI * r1;
        let x = phi.cos() * q2;
        let y = phi.sin() * q2;
        let z = (1.0 - r2).sqrt();
    
        Vec3::new(x, y, z)
    }
    

    #[inline(always)]
    pub fn random_cosine_direction_exponent(exponent: f64, rng: &mut FastRng) -> Self {
        let r1 = rng.gen();
        let r2 = rng.gen().powf(1.0 / (exponent + 1.0));
        let sin_theta = (1.0 - r2 * r2).sqrt();

        let phi = 2.0 * PI * r1;
        let x = phi.cos() * sin_theta;
        let y = phi.sin() * sin_theta;
        let z = r2;

        Vec3::new(x, y, z)
    }


    #[must_use]
    pub fn random_unit_dir(dir: &Self, rng: &mut FastRng) -> Self {
        let u = Self::random_unit(rng);
        if u.dot(dir) > 0.0 {
            u
        } else {
            -u
        }
    }

    #[must_use]
    pub fn random_unit_disk(rng: &mut FastRng) -> Self {
        loop {
            let p = Self::new(rng.range(-1.0, 1.0), rng.range(-1.0, 1.0), 0.0);
            if p.length_squared() < 1.0 {
                return p;
            }
        }
    }


    #[must_use]
    #[inline]
    pub fn length_squared(&self) -> f64 {
        self.z
            .mul_add(self.z, self.x.mul_add(self.x, self.y * self.y))
    }

    #[must_use]
    #[inline]
    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }

    #[inline]
    pub fn reverse(&mut self) {
        self.x = -self.x;
        self.y = -self.y;
        self.z = -self.z;
    }

    #[inline(always)]
    pub fn reflect(&self, n: &Self) -> Self {
        self - (n * (2.0 * self.dot(n)))
    }

    #[must_use]
    #[inline]
    pub fn dot(&self, rhs: &Self) -> f64 {
        self.z.mul_add(rhs.z, self.x.mul_add(rhs.x, self.y * rhs.y))
    }

    #[must_use]
    pub fn cross(&self, rhs: &Self) -> Self {
        Self::new(
            self.y * rhs.z - self.z * rhs.y,
            self.z * rhs.x - self.x * rhs.z,
            self.x * rhs.y - self.y * rhs.x,
        )
    }

    #[must_use]
    #[inline]
    pub fn unit(&self) -> Self {
        self / self.length()
    }

    pub fn rotate(&self, axis: i32, cos: f64, sin: f64) -> Self {
        match axis {
            0 => Vec3::new(
                self.x,
                self.y * cos - self.z * sin,
                self.y * sin + self.z * cos,
            ),
            1 => Vec3::new(
                self.x * cos + self.z * sin,
                self.y,
                -self.x * sin + self.z * cos,
            ),
            _ => Vec3::new(
                self.x * cos - self.y * sin,
                self.x * sin + self.y * cos,
                self.z,
            ),
        }
    }

    #[inline(always)]
    pub fn mul_add(&self, a: f64, b: &Vec3) -> Self {
        Vec3 {
            x: self.x.mul_add(a, b.x),
            y: self.y.mul_add(a, b.y),
            z: self.z.mul_add(a, b.z),
        }
    }

    #[allow(clippy::cast_precision_loss)] // sample count is small enough in practice
    #[must_use]
    pub fn into_color(mut self, sample_count: usize, gamma: bool) -> Color {
        self /= sample_count as f64;
        if gamma {
            self.x = self.x.sqrt();
            self.y = self.y.sqrt();
            self.z = self.z.sqrt();
        }
        Color::new64(
            self.x,
            self.y,
            self.z,
            1.0
        )
    }
}

impl Display for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{} {} {}", self.x, self.y, self.z))
    }
}

impl Index<usize> for Vec3 {
    type Output = f64;
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Vec3 can only index by 0-2, {} provided", index),
        }
    }
}

impl IndexMut<usize> for Vec3 {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("Vec3 can only index by 0-2, {} provided", index),
        }
    }
}

impl Neg for &Vec3 {
    type Output = Vec3;
    #[inline]
    fn neg(self) -> Self::Output {
        Vec3::new(-self.x, -self.y, -self.z)
    }
}

impl Neg for Vec3 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self::Output {
        Vec3::new(-self.x, -self.y, -self.z)
    }
}

impl Add<Self> for &Vec3 {
    type Output = Vec3;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Vec3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Add<Vec3> for &Vec3 {
    type Output = Vec3;
    #[inline]
    fn add(self, rhs: Vec3) -> Self::Output {
        self + &rhs
    }
}

impl Add<Self> for Vec3 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        &self + &rhs
    }
}

impl Add<&Self> for Vec3 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: &Self) -> Self::Output {
        &self + rhs
    }
}

impl Sub<Self> for &Vec3 {
    type Output = Vec3;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Vec3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Sub<Vec3> for &Vec3 {
    type Output = Vec3;
    #[inline]
    fn sub(self, rhs: Vec3) -> Self::Output {
        self - &rhs
    }
}

impl Sub<Self> for Vec3 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        &self - &rhs
    }
}

impl Sub<&Self> for Vec3 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: &Self) -> Self::Output {
        &self - rhs
    }
}

impl Mul<Self> for &Vec3 {
    type Output = Vec3;
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Vec3::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}

impl Mul<Vec3> for &Vec3 {
    type Output = Vec3;
    #[inline]
    fn mul(self, rhs: Vec3) -> Self::Output {
        self * &rhs
    }
}

impl Mul<Self> for Vec3 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        &self * &rhs
    }
}

impl Mul<&Self> for Vec3 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: &Self) -> Self::Output {
        &self * rhs
    }
}

impl Mul<&Color> for &Vec3 {
    type Output = Vec3;
    #[inline]
    fn mul(self, rhs: &Color) -> Self::Output {
        Vec3::new(
            self.x * rhs.r as f64,
            self.y * rhs.g as f64,
            self.z * rhs.b as f64,
        )
    }
}

impl Mul<Color> for &Vec3 {
    type Output = Vec3;
    #[inline]
    fn mul(self, rhs: Color) -> Self::Output {
        self * &rhs
    }
}

impl Mul<Color> for Vec3 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Color) -> Self::Output {
        &self * &rhs
    }
}

impl Mul<&Color> for Vec3 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: &Color) -> Self::Output {
        &self * rhs
    }
}

impl Mul<&Vec3> for &Color {
    type Output = Vec3;
    #[inline]
    fn mul(self, rhs: &Vec3) -> Self::Output {
        rhs * self
    }
}

impl Mul<&Vec3> for Color {
    type Output = Vec3;
    #[inline]
    fn mul(self, rhs: &Vec3) -> Self::Output {
        &self * rhs
    }
}

impl Mul<Vec3> for Color {
    type Output = Vec3;
    #[inline]
    fn mul(self, rhs: Vec3) -> Self::Output {
        &self * &rhs
    }
}

impl Mul<Vec3> for &Color {
    type Output = Vec3;
    #[inline]
    fn mul(self, rhs: Vec3) -> Self::Output {
        self * &rhs
    }
}

impl Mul<f64> for &Vec3 {
    type Output = Vec3;
    #[inline]
    fn mul(self, rhs: f64) -> Self::Output {
        Vec3::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f64) -> Self::Output {
        &self * rhs
    }
}

impl Mul<Vec3> for f64 {
    type Output = Vec3;
    #[inline]
    fn mul(self, rhs: Vec3) -> Self::Output {
        rhs * self
    }
}

impl Mul<&Vec3> for f64 {
    type Output = Vec3;
    #[inline]
    fn mul(self, rhs: &Vec3) -> Self::Output {
        rhs * self
    }
}

impl Div<f64> for &Vec3 {
    type Output = Vec3;
    #[inline]
    fn div(self, rhs: f64) -> Self::Output {
        self * (1.0 / rhs)
    }
}

impl Div<f64> for Vec3 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f64) -> Self::Output {
        self * (1.0 / rhs)
    }
}

impl AddAssign<&Self> for Vec3 {
    #[inline]
    fn add_assign(&mut self, rhs: &Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl AddAssign<Self> for Vec3 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl SubAssign<&Self> for Vec3 {
    #[inline]
    fn sub_assign(&mut self, rhs: &Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl SubAssign<Self> for Vec3 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl MulAssign<Self> for Vec3 {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}

impl MulAssign<&Self> for Vec3 {
    #[inline]
    fn mul_assign(&mut self, rhs: &Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}

impl MulAssign<f64> for Vec3 {
    #[inline]
    fn mul_assign(&mut self, rhs: f64) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl DivAssign<f64> for Vec3 {
    #[inline]
    fn div_assign(&mut self, rhs: f64) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

impl Sum for Vec3 {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::default(), |acc, val| acc + val)
    }
}

impl From<Color> for Vec3 {
    fn from(c: Color) -> Self {
        Self::new(c.r as f64, c.g as f64, c.b as f64)
    }
}
