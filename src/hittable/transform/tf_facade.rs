use crate::{
    hittable::{HitRecord, Hittable},
    prelude::*,
};

use crate::hittable::transform::TransformStack;

use std::ops::Range;
use std::fmt::Formatter;
use std::fmt::Debug;

pub struct TfFacade<T> {
    object: T,
    stack: TransformStack,
}

impl<T: std::fmt::Debug> Debug for TfFacade<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "TfFacade {{ object: {:?}, tf_stack: {} }}",
            self.object, self.stack.len(),
        ))
    }
}

impl<T> TfFacade<T> {
    pub const fn new(object: T, stack: TransformStack) -> Self {
        Self { 
            object,
            stack 
        }
    }
}

impl<T: Hittable> Hittable for TfFacade<T> {

    fn hit(&self, ray_in: &Ray, unit_limit: &Range<f64>) -> Option<HitRecord> {
    
        let ray = Ray::new(self.stack.inv_tf_pos(&ray_in.origin), ray_in.direction.clone(), ray_in.departure_time);

        self.object
            .hit(&ray, unit_limit)
            .map(|mut hit| {
                hit.point = self.stack.tf_pos(&hit.point);

                hit
            })
    }

    fn bbox(&self, time_limit: &Range<f64>) -> Option<AABB> {
        self.object
            .bbox(time_limit)
            .map(|bbox| {
                AABB::new(self.stack.tf_pos(&bbox.min()),
                          self.stack.tf_pos(&bbox.max()))
            })
    }

    fn random(&self, origin: &Point3, rng: &mut FastRng) -> Vec3 {
        let r = self.object.random(&self.stack.inv_tf_pos(origin), rng);
        r
    }
}
