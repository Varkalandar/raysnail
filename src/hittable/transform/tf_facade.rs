use crate::{
    hittable::{HitRecord, Hittable},
    prelude::*,
};

use crate::hittable::transform::TransformStack;

use std::ops::Range;
use std::fmt::Formatter;
use std::fmt::Debug;
use once_cell::sync::OnceCell;

pub struct TfFacade<T> {
    object: T,
    stack: TransformStack,
    bbox_cache: OnceCell<Option<AABB>>,
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
            stack,
            bbox_cache: OnceCell::new(), 
        }
    }
}

impl<T: Hittable> Hittable for TfFacade<T> {

    fn hit(&self, ray_in: &Ray, unit_limit: &Range<f64>) -> Option<HitRecord> {
    
        let ray = Ray::new(self.stack.inv_tf_pos(&ray_in.origin), 
                           self.stack.inv_tf_vec(&ray_in.direction), 
                           ray_in.departure_time);

        self.object
            .hit(&ray, unit_limit)
            .map(|mut hit| {
                hit.point = self.stack.tf_pos(&hit.point);

                hit
            })
    }

    /*
    fn bbox(&self, time_limit: &Range<f64>) -> Option<AABB> {
        self.object
            .bbox(time_limit)
            .map(|bbox| {
                AABB::new(self.stack.tf_pos(&bbox.min()),
                          self.stack.tf_pos(&bbox.max()))
            })
    }
    */

    fn bbox(&self, time_limit: &Range<f64>) -> Option<AABB> {
        self.bbox_cache
            .get_or_init(|| {
                self.object.bbox(time_limit).map(|bbox| {
                    let mut point_min = Point3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
                    let mut point_max =
                        Point3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);

                    for i in 0..2_usize {
                        for j in 0..2_usize {
                            for k in 0..2_usize {
                                let x =
                                    (i as f64).mul_add(bbox.max().x, (1 - i) as f64 * bbox.min().x);
                                let y =
                                    (j as f64).mul_add(bbox.max().y, (1 - j) as f64 * bbox.min().y);
                                let z =
                                    (k as f64).mul_add(bbox.max().z, (1 - k) as f64 * bbox.min().z);

                                let tf_point = self.stack.tf_pos(&Point3::new(x, y, z));

                                for c in 0..3 {
                                    point_min[c] = point_min[c].min(tf_point[c]);
                                    point_max[c] = point_max[c].max(tf_point[c]);
                                }
                            }
                        }
                    }

                    AABB::new(point_min, point_max)
                })
            })
            .clone()
    }


    fn random(&self, origin: &Point3, rng: &mut FastRng) -> Vec3 {
        let r = self.object.random(&self.stack.inv_tf_pos(origin), rng);
        r
    }
}
