use {
    crate::prelude::*,
    std::ops::{BitOr, BitOrAssign, Range},
};

/// Axis aligned bounding box
#[derive(Debug, Clone)]
pub struct AABB {
    pub min: Point3,
    pub max: Point3,
}

impl AABB {
    #[must_use]
    pub const fn new(min: Point3, max: Point3) -> Self {
        Self { min, max }
    }

    #[must_use]
    pub fn hit(&self, ray: &Ray, unit_limit: &Range<f64>) -> bool {
        let mut t_min = unit_limit.start;
        let mut t_max = unit_limit.end;
        for i in 0..3 {
            // TODO: when inv = Inf and min - origin = 0, the calculation will give a NaN
            let inv = 1.0 / ray.direction[i];
            let mut t0 = (self.min[i] - ray.origin[i]) * inv;
            let mut t1 = (self.max[i] - ray.origin[i]) * inv;
            if inv < 0.0 {
                std::mem::swap(&mut t0, &mut t1);
            }
            t_min = t_min.max(t0);
            t_max = t_max.min(t1);
            if t_max <= t_min {
                return false;
            }
        }
        true
    }

    /*
    pub fn longest_axis() usize {
        // Returns the index of the longest axis of the bounding box.

        let x_size = max.x - min.x;
        let y_size = max.y - min.y;
        let z_size = max.z - min.z;

        if x_size > y_size {
            if x_size > z_size {0} else {2}
        }
        else {
            if y_size > z_size {1} else {2}
        }
    }
    */
}

impl BitOr<Self> for &AABB {
    type Output = AABB;

    fn bitor(self, rhs: Self) -> Self::Output {
        let min = Point3::new_min(&self.min, &rhs.min);
        let max = Point3::new_max(&self.max, &rhs.max);

        AABB::new(min, max)
    }
}

impl BitOr<Self> for AABB {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        &self | &rhs
    }
}

impl BitOr<&Self> for AABB {
    type Output = Self;

    fn bitor(self, rhs: &Self) -> Self::Output {
        &self | rhs
    }
}

impl BitOr<AABB> for &AABB {
    type Output = AABB;

    fn bitor(self, rhs: AABB) -> Self::Output {
        self | &rhs
    }
}

impl BitOrAssign<&Self> for AABB {
    fn bitor_assign(&mut self, rhs: &Self) {
        self.min = Point3::new_min(&self.min, &rhs.min);
        self.max = Point3::new_max(&self.max, &rhs.max);
    }
}

impl BitOrAssign<Self> for AABB {
    fn bitor_assign(&mut self, rhs: Self) {
        *self |= &rhs
    }
}
