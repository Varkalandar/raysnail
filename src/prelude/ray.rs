use crate::prelude::vec3::{Point3, Vec3};

#[derive(Debug)]
pub struct Ray {
    pub origin: Point3,
    pub direction: Vec3,
    pub departure_time: f64,
}

impl Ray {
    #[must_use]
    pub const fn new(origin: Point3, direction: Vec3, departure_time: f64) -> Self {
        Self {
            origin,
            direction,
            departure_time,
        }
    }

    #[must_use]
    pub fn position_after(&self, t: f64) -> Vec3 {
        // &self.origin + &self.direction * t

        // self.direction.mul_add(t, &self.origin)

        Vec3 {
            x: self.direction.x.mul_add(t, self.origin.x),
            y: self.direction.y.mul_add(t, self.origin.y),
            z: self.direction.z.mul_add(t, self.origin.z),
        }

    }
}
