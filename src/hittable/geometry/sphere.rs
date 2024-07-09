use {
    crate::{
        hittable::{HitRecord, Hittable},
        material::Material,
        prelude::*,
    },
    std::{
        fmt::{Debug, Formatter},
        ops::Range,
    },
};

#[derive(Clone)]
pub struct Sphere<M> {
    center: Point3,
    radius: f64,
    speed: Vec3,
    material: M,
    radius_squared: f64,
}

impl<M> Debug for Sphere<M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Sphere {{ center: {:?}, radius: {}, speed: {:?} }}",
            self.center, self.radius, self.speed,
        ))
    }
}

impl<M> Sphere<M> {
    pub fn new(center: Point3, radius: f64, material: M) -> Self {
        Self {
            center,
            radius,
            material,
            speed: Vec3::default(),
            radius_squared: radius * radius,
        }
    }

    pub const fn with_speed(mut self, speed: Vec3) -> Self {
        self.speed = speed;
        self
    }

    pub fn center_at(&self, t: f64) -> Point3 {
        &self.center + &self.speed * t
    }
}

impl<M: Material> Hittable for Sphere<M> {
    fn normal(&self, point: &Point3) -> crate::prelude::Vec3 {
        (point - &self.center) / self.radius
    }

    fn material(&self) -> &dyn Material {
        &self.material
    }

    fn uv(&self, point: &Point3) -> (f64, f64) {
        let point = (point - &self.center).unit();
        let phi = (-point.z).atan2(point.x); // [-pi, pi]
        let theta = point.y.asin(); // [-pi / 2 , pi / 2]
        let u = phi / 2.0 / PI + 0.5;
        let v = theta / PI + 0.5;
        (u, v)
    }

    // Ray(t) = O + tD
    // Sphere surface = (X - C)^2 = r^2
    // (O + tD - C)^2 = r^2
    // let O - C = L
    // (tD + L)^2 = r^2
    // D^2 t^2 + 2DLt + L^2- r^2 = 0
    // a = D^2, b = 2(DL), c = L^2 - r^2
    // Delta = b^2 - 4ac = 4(DL)^2 - 4 D^2 (L^2 - r2)
    // So, check (DL)^2 - D^2(L^2 - r^2)
    // root is
    fn hit(&self, ray: &Ray, unit_limit: &Range<f64>) -> Option<HitRecord<'_>> {
        let current_center = self.center_at(ray.departure_time);
        let l = &ray.origin - current_center;
        let half_b = ray.direction.dot(&l);
        let a = ray.direction.length_squared();
        let c = l.length_squared() - self.radius_squared;
        #[allow(clippy::suspicious_operation_groupings)]
        let delta = half_b * half_b - a * c;

        if delta < 0.0 {
            return None;
        }

        let sqrt = delta.sqrt();

        let t1 = (-half_b - sqrt) / a;
        let t2 = (-half_b + sqrt) / a;
        if unit_limit.contains(&t1) {
            return Some(HitRecord::new(ray, self, t1, t2));
        }

        if unit_limit.contains(&t2) {
            return Some(HitRecord::new(ray, self, t2, t2));
        }

        None
    }

    fn bbox(&self, time_limit: &Range<f64>) -> Option<AABB> {
        Some(
            if self.speed.x == 0.0 && self.speed.y == 0.0 && self.speed.z == 0.0 {
                AABB::new(
                    &self.center - Vec3::new(self.radius, self.radius, self.radius),
                    &self.center + Vec3::new(self.radius, self.radius, self.radius),
                )
            } else {
                let start = AABB::new(
                    self.center_at(time_limit.start)
                        - Vec3::new(self.radius, self.radius, self.radius),
                    self.center_at(time_limit.start)
                        + Vec3::new(self.radius, self.radius, self.radius),
                );

                let end = AABB::new(
                    self.center_at(time_limit.end)
                        - Vec3::new(self.radius, self.radius, self.radius),
                    self.center_at(time_limit.end)
                        + Vec3::new(self.radius, self.radius, self.radius),
                );

                start | end
            },
        )
    }

    /**
     * This is only called if the object is a light source. It is used to check the probability of a
     * particular direction to be scattered from this object.
     */
    fn pdf_value(&self, origin: &Point3, direction: &Vec3) -> f64 {

        if let Some(_hit) = self.hit(&Ray::new(origin.clone(), direction.clone(), 0.0), &(0.001..f64::INFINITY)) {
            let cos_theta =
                (1.0 - self.radius * self.radius / (&self.center - origin).length_squared()).sqrt();

            let solid_angle = 2.0 * PI * (1.0 - cos_theta);

            if solid_angle == 0.0 {
                return 1e10;
            }

            println!("Light hit. pdf value={}", 1.0 / solid_angle);
        
            return 1.0 / solid_angle;
        }

        println!("Light miss. pdf value=0.0");

        0.0
    }


    /**
     * This is only called if the object is a light source. It is used to generate
     * an extra ray towards the light source.
     */
    fn random(&self, origin: &Point3, rng: &mut FastRng) -> Vec3 {

        let direction = &self.center - origin;
        let uvw = ONB::build_from(&direction);

        loop {
            let u = &uvw.axis[0] * rng.gen();
            let v = &uvw.axis[1] * rng.gen();

            let uv = u + &v;

            if uv.length_squared() < 1.0 {
                return (uv + &self.center) - origin
            }
        }
    }
}
