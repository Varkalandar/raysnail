use {
    crate::{
        hittable::{Hittable, HitRecord},
        prelude::*,
    },
    std::{
        fmt::{Debug, Formatter},
        ops::Range,
    },
};

#[derive(Default)]
pub struct HittableList {
    objects: Vec<Box<dyn Hittable>>,
}

impl Debug for HittableList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "GeometryList {{ objects: {}}}",
            self.objects.len()
        ))
    }
}

impl HittableList {
    pub fn add<G: Hittable + 'static>(&mut self, object: G) -> &mut Self {
        let object: Box<dyn Hittable> = Box::new(object);
        self.objects.push(object);
        self
    }

    pub fn add_ref(&mut self, object: Box<dyn Hittable>) -> &mut Self {
        self.objects.push(object);
        self
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }

    #[must_use]
    pub fn into_objects(self) -> Vec<Box<dyn Hittable>> {
        self.objects
    }

    pub fn pdf_value(&self, origin: &Point3, direction: &Vec3) -> f64 {
        let weight = 1.0 / self.objects.len() as f64;
        let mut sum = 0.0;

        for object in &self.objects {
            sum += weight * object.pdf_value(origin, direction);
        }

        sum
    }

    pub fn random(&self, origin: &Point3) -> Vec3 {
        let size = self.objects.len();
        return self.objects[Random::range(0 .. size)].random(origin);
    }    
}


impl Hittable for HittableList {
    fn hit(&self, r: &Ray, unit_limit: &Range<f64>) -> Option<HitRecord<'_>> {

        let mut best_hit = None;
        let mut best_unit = 0.0;

        for object in &self.objects {
            let hit_opt = object.hit(r, unit_limit);

            if hit_opt.is_some() {
                let hit = hit_opt.unwrap();

                if best_hit.is_none() {
                    best_unit = hit.unit;
                    best_hit = Some(hit);
                }
                else {
                    if hit.unit < best_unit {
                        best_unit = hit.unit;
                        best_hit = Some(hit);
                    }
                }
            }        
        }

        if best_hit.is_some() {
            let best = best_hit.unwrap();
            // println!("{}", best.material.name());

            return Some(best)
        }

        None
    }

    fn bbox(&self, time_limit: Range<f64>) -> Option<AABB> {
        if self.objects.is_empty() {
            return None;
        }

        let mut result: Option<AABB> = None;

        for object in &self.objects {
            let bbox = object.bbox(time_limit.clone())?;
            result = result.map(|last| last | &bbox).or(Some(bbox))
        }

        result
    }
    
    fn pdf_value(&self, origin: &Point3, direction: &Vec3) -> f64 {
        0.0
    }

    fn random(&self, origin: &Point3) -> Vec3 {
        Vec3::new(1.0, 0.0, 0.0)
    }    
}
