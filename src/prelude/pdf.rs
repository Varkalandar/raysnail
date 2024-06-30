use crate::prelude::Vec3;
use crate::prelude::ONB;
use crate::prelude::PI;
use crate::prelude::Random;
use crate::prelude::Point3;
use crate::hittable::collection::HittableList;

use std::fmt::Debug;
use std::fmt::Formatter;


pub trait PDF {
    fn value(&self, direction: &Vec3) -> f64;
    fn generate(&self) -> Vec3;
}


#[derive(Debug)]
pub struct CosinePdf {
    uvw: ONB,
}

impl CosinePdf {
    
    pub fn new(n: &Vec3) -> Self { 
        CosinePdf {
            uvw: ONB::build_from(n),
        } 
    }
}


impl PDF for CosinePdf {

    fn value(&self, direction: &Vec3) -> f64 {
        let cosine_theta = direction.unit().dot(&self.uvw.axis[2]);
        let v = cosine_theta / PI;

        if v < 0.0 { 0.0 } else { v }
    }
  
    fn generate(&self) -> Vec3 {
        self.uvw.local(&Vec3::random_cosine_direction())
    }
}


#[derive(Debug)]
pub struct SpherePdf {

}

impl SpherePdf {
    pub fn new() -> SpherePdf {
        SpherePdf {}
    }
}

impl PDF for SpherePdf {

    fn value(&self, direction: &Vec3) -> f64 {
        1.0 / (4.0 * PI)
    }
  
    fn generate(&self) -> Vec3 {
        Vec3::random_unit()
    }
}


#[derive(Debug)]
pub struct HittablePdf<'a> {
    objects: &'a HittableList,
    origin: Point3,
}

impl HittablePdf <'_> {
    pub fn new<'a>(objects: &'a HittableList, origin: &'a Point3) -> HittablePdf<'a> { 
        HittablePdf {
            objects: objects,
            origin: origin.clone(),
        }
    }
}


impl PDF for HittablePdf<'_> {
  
    fn value(&self, direction: &Vec3) -> f64 {
        self.objects.pdf_value(&self.origin, direction)
    }
  
    fn generate(&self) -> Vec3 {
        self.objects.random(&self.origin)
    }
}


pub struct MixturePdf<'a> {
    p0: Box<&'a dyn PDF>, 
    p1: Box<&'a dyn PDF>, 
}


impl <'a> MixturePdf<'a> {
    pub fn new(p0: &'a dyn PDF, p1: &'a dyn PDF) -> MixturePdf<'a> {
        MixturePdf {
            p0: Box::new(p0),
            p1: Box::new(p1),
        }
    }
}

impl Debug for MixturePdf<'_> {
    fn fmt(&self, _: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        Ok(())
    }
}

impl PDF for MixturePdf<'_> {

    fn value(&self, direction: &Vec3) -> f64 {
        return 0.5 * self.p0.value(direction) + 0.5 * self.p1.value(direction);
    }
  
    fn generate(&self) -> Vec3 {
        if Random::gen() < 0.5 {
            return self.p0.generate();
        }
        else {
            return self.p1.generate();
        }
    }  
}
