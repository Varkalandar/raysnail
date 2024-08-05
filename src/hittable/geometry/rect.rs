use {
    crate::{
        hittable::{Hittable, HitRecord},
        material::Material,
        prelude::*,
    },
    std::ops::Range,
};
use std::sync::Arc;
use std::fmt::Formatter;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct AARectMetrics {
    k: f64,
    a0: f64,
    a1: f64,
    b0: f64,
    b1: f64,
    a_len: f64,
    b_len: f64,
}

impl AARectMetrics {
    #[must_use]
    pub fn new(k: f64, (a0, a1): (f64, f64), (b0, b1): (f64, f64)) -> Self {
        assert!(a0 < a1);
        assert!(b0 < b1);
        Self {
            k,
            a0,
            a1,
            b0,
            b1,
            a_len: a1 - a0,
            b_len: b1 - b0,
        }
    }
}

#[derive(Clone)]
pub struct AARect {
    // 0: a axis, 1: b axis, 2: fixed axis
    axis: (usize, usize, usize),
    metrics: AARectMetrics,
    material: Option<Arc<dyn Material>>,
}

impl Debug for AARect {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "AARect"
        ))
    }
}

impl AARect {
    pub const fn new_xy(metrics: AARectMetrics, material: Option<Arc<dyn Material>>) -> Self {
        Self {
            metrics,
            material,
            axis: (0, 1, 2),
        }
    }

    pub const fn new_xz(metrics: AARectMetrics, material: Option<Arc<dyn Material>>) -> Self {
        Self {
            metrics,
            material,
            axis: (0, 2, 1),
        }
    }

    pub const fn new_yz(metrics: AARectMetrics, material: Option<Arc<dyn Material>>) -> Self {
        Self {
            metrics,
            material,
            axis: (1, 2, 0),
        }
    }
}

impl Hittable for AARect {
    fn normal(&self, _point: &Point3) -> Vec3 {
        let mut n = Vec3::default();
        n[self.axis.2] = 1.0;
        n
    }

    fn material(&self) -> Option<Arc<dyn Material>> {
        self.material.clone()
    }

    fn uv(&self, point: &Point3) -> (f64, f64) {
        (
            (point[self.axis.0] - self.metrics.a0) / self.metrics.a_len,
            (point[self.axis.1] - self.metrics.b0) / self.metrics.b_len,
        )
    }

    fn hit(&self, ray: &Ray, unit_limit: &Range<f64>) -> Option<HitRecord> {
        let t1 = (self.metrics.k - ray.origin[self.axis.2]) / ray.direction[self.axis.2];
        if !unit_limit.contains(&t1) {
            return None;
        }

        let a = t1.mul_add(ray.direction[self.axis.0], ray.origin[self.axis.0]);

        if a < self.metrics.a0 || a > self.metrics.a1 {
            return None;
        }

        let b = t1.mul_add(ray.direction[self.axis.1], ray.origin[self.axis.1]);

        if b < self.metrics.b0 || b > self.metrics.b1 {
            return None;
        }

        Some(HitRecord::new(ray, self, t1, f64::MAX))
    }

    fn contains(&self, _point: &Vec3) -> bool
    {
        false
    }

    fn bbox(&self, _time_limit: &Range<f64>) -> Option<AABB> {
        let mut p0 = Point3::default();
        p0[self.axis.0] = self.metrics.a0;
        p0[self.axis.1] = self.metrics.b0;
        p0[self.axis.2] = self.metrics.k - 0.0001;

        let mut p1 = Point3::default();
        p1[self.axis.0] = self.metrics.a1;
        p1[self.axis.1] = self.metrics.b1;
        p1[self.axis.2] = self.metrics.k + 0.0001;

        Some(AABB::new(p0, p1))
    }

    fn random(&self, origin: &Point3, rng: &mut FastRng) -> Vec3 {
        // axis 2 is distance to the origin
        // axis 0 and 1 are orthoginal 2d vectors to the fixed axis

        // only need axis 2 == y axis at the moment

        let mut root = Vec3::new(0.0, self.metrics.k, 0.0);

        root.x = rng.range(self.metrics.a0, self.metrics.a1);
        root.z = rng.range(self.metrics.b0, self.metrics.b1);

        origin - root
    }
}
