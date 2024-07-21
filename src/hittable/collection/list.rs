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

    pub fn random(&self, origin: &Point3, rng: &mut FastRng) -> Vec3 {
        let size = self.objects.len();
        return self.objects[rng.irange(0, size)].random(origin, rng);
    }    

    pub fn hit(&self, r: &Ray, unit_limit: &Range<f64>) -> Vec<HitRecord> {

        let mut hits = Vec::new();

        for object in &self.objects {
            let hit_opt = object.hit(r, unit_limit);

            if hit_opt.is_some() {
                let hit = hit_opt.unwrap();
                hits.push(hit);
            }        
        }

        hits
    }

    pub fn bbox(&self, time_limit: &Range<f64>) -> Option<AABB> {
        if self.objects.is_empty() {
            return None;
        }

        let mut result: Option<AABB> = None;

        for object in &self.objects {
            let bbox = object.bbox(time_limit)?;
            result = result.map(|last| last | &bbox).or(Some(bbox))
        }

        result
    }
}
