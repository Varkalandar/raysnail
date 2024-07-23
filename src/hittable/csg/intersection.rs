use std::ops::Range;
use std::fmt::Formatter;
use std::fmt::Debug;

use crate::prelude::FastRng;
use crate::prelude::Vec3;
use crate::prelude::AABB;
use crate::prelude::Ray;
use crate::prelude::Point3;
use crate::hittable::HitRecord;
use crate::hittable::Hittable;


pub struct Intersection {
    o1: Box<dyn Hittable>, 
    o2: Box<dyn Hittable>
}

impl Debug for Intersection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Intersection",
        ))
    }
}


impl Intersection {
    pub fn new(o1: Box<dyn Hittable>, o2: Box<dyn Hittable>) -> Self {
        Intersection {
            o1,
            o2,
        }
    }
}

impl Hittable for Intersection {
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
        
        let hit1 = self.o1.hit(ray, unit_limit);
        let hit2 = self.o2.hit(ray, unit_limit);
    
        if hit1.is_some() && hit2.is_some() {
            let hit1 = hit1.unwrap();
            let hit2 = hit2.unwrap();

            // sort the hits and objects
            let hits;
            let objs;

            if hit1.t1 < hit2.t1 {
                hits = [&hit1, &hit2];
                objs = [&self.o1, &self.o2];
            }
            else {
                hits = [&hit2, &hit1];
                objs = [&self.o2, &self.o1];
            }


            if objs[1].contains(&hits[0].point) {
                // hit[0] is on the nearest surface and inside the farer object
                // we can use it directly
                return Some(hits[0].clone());
            }
            else {
                // hit[0] was not inside the farer object, so we must check
                // the second hit
                if objs[0].contains(&hits[1].point) {
                    return Some(hits[1].clone());
                }
            }
        }

        None
    }

    fn contains(&self, point: &Vec3) -> bool {
        self.o1.contains(point) && self.o2.contains(point)
    }

    fn bbox(&self, time_limit: &Range<f64>) -> Option<AABB> {
        Some(self.o1.bbox(time_limit).unwrap() | self.o2.bbox(time_limit).unwrap())
    }

    /**
     * This is only called if the object is a light source. It is used to generate
     * an extra ray towards the light source.
     */
     fn random(&self, origin: &Point3, rng: &mut FastRng) -> Vec3 {

        self.o1.random(origin, rng)
    }


}