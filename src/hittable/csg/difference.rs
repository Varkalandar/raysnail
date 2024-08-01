use std::ops::Range;
use std::sync::Arc;
use std::fmt::Formatter;
use std::fmt::Debug;

use crate::prelude::FastRng;
use crate::prelude::Vec3;
use crate::prelude::AABB;
use crate::prelude::Ray;
use crate::prelude::Point3;
use crate::material::Material;
use crate::hittable::HitRecord;
use crate::hittable::Hittable;


pub struct Difference {
    plus: Box<dyn Hittable>, 
    minus: Box<dyn Hittable>,
    material: Option<Arc<dyn Material>>,
}

impl Debug for Difference {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Difference",
        ))
    }
}


impl Difference {
    pub fn new(plus: Box<dyn Hittable>, minus: Box<dyn Hittable>, material: Option<Arc<dyn Material>>) -> Self {
        Difference {
            plus,
            minus,
            material,
        }
    }
}

impl Hittable for Difference {
    fn normal(&self, point: &Point3) -> Vec3 {
        unimplemented!(
            "{}'s normal function should not be called directly",
            std::any::type_name::<Self>()
        )
    }


    fn uv(&self, point: &Point3) -> (f64, f64) {
        unimplemented!(
            "{}'s uv function should not be called directly",
            std::any::type_name::<Self>()
        )
    }

    fn hit(&self, ray: &Ray, unit_limit: &Range<f64>) -> Option<HitRecord> {
        
        let hit_plus = self.plus.hit(ray, unit_limit);
        let hit_minus = self.minus.hit(ray, unit_limit);
        
        if hit_plus.is_some() {
            
            if hit_minus.is_some() {
                let hit_plus = hit_plus.unwrap();
                let hit_minus = hit_minus.unwrap();

                if hit_plus.t1 < hit_minus.t1 {
                    // visible object was hit first
                    // but there are strange cases if the ray starts in just this object

                    if !self.minus.contains(&hit_plus.point) {
                        return Some(hit_plus.set_material_if_none(self.material.clone()));
                    }
                }
                else {
                    // the back hit of the invisible object could be it

                    if hit_minus.t2 < hit_plus.t1 {
                        // negative object if fully in front of positive object
                        return Some(hit_plus.set_material_if_none(self.material.clone()));
                    }
                    else if hit_minus.t2 < hit_plus.t2 {
                        let p = ray.at(hit_minus.t2);
                        let n = self.minus.normal(&p);

                        // println!("p={:?}" , p);

                        return Some(HitRecord::with_normal(p.clone(), 
                                                        -n, // Vec3::new(-1.0, 0.0, 0.0), 
                                                        self.minus.material().clone(), 
                                                        (0.0, 0.0), // (hit_minus.u, hit_minus.v), 
                                                        hit_minus.t2, 
                                                        hit_plus.t2)
                                                        .set_material_if_none(self.material.clone())
                                                    );
                    }
                }
            }
            else {
                // we can use this directly, ray hit only the positive object
                return hit_plus;
            }
        }
        None
    }


    fn contains(&self, point: &Vec3) -> bool {
        self.plus.contains(point) && !self.minus.contains(point)
    }


    fn bbox(&self, time_limit: &Range<f64>) -> Option<AABB> {
        self.plus.bbox(time_limit)
    }

    /**
     * This is only called if the object is a light source. It is used to generate
     * an extra ray towards the light source.
     */
     fn random(&self, origin: &Point3, rng: &mut FastRng) -> Vec3 {

        self.plus.random(origin, rng)
    }
}