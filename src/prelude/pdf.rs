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
    onb: ONB,
}

impl CosinePdf {
    
    pub fn new(n: &Vec3) -> Self { 
        CosinePdf {
            onb: ONB::build_from(n),
        } 
    }
}


impl PDF for CosinePdf {

    fn value(&self, direction: &Vec3) -> f64 {
        let cosine_theta = direction.unit().dot(&self.onb.axis[2]);
        let v = cosine_theta / PI;

        if v < 0.0 { 0.0 } else { v }
    }
  
    fn generate(&self) -> Vec3 {
        self.onb.local(&Vec3::random_cosine_direction())
    }
}


#[derive(Debug)]
pub struct ReflectionPdf {
    onb_normal: ONB,
    onb_reflected: ONB,
    exponent: f64,
}

impl ReflectionPdf {
    
    pub fn new(r_in_direction: Vec3, normal: Vec3, exponent: f64) -> Self {
        let reflected = r_in_direction.unit().reflect(&normal.unit());
        let onb_reflected = ONB::build_from(&reflected);
        let onb_normal = ONB::build_from(&normal);

        Self {
            onb_normal,
            onb_reflected,
            exponent,
        }
    }
}


impl PDF for ReflectionPdf {

    fn value(&self, direction: &Vec3) -> f64 {
        let cosine_theta = direction.unit().dot(&self.onb_reflected.axis[2]);
        let v = cosine_theta / PI;

        if v < 0.0 { 0.0 } else { v }
    }
  
    fn generate(&self) -> Vec3 {

        /*
        self.onb_reflected.axis[2].clone()
        */
        
        loop {
            let direction =
            self
            .onb_reflected
            .local(&Vec3::random_cosine_direction_exponent(self.exponent));

            if direction.dot(&self.onb_normal.axis[2]) > 0.0 {
                return direction;
            }
        }
    }
}


#[derive(Debug)]
pub struct BlinnPhongPdf {
    r_in_direction: Vec3,
    onb_normal: ONB,
    onb_reflected: ONB,
    k_specular: f64,
    exponent: f64,
}

impl BlinnPhongPdf {
    pub fn new(r_in_direction: Vec3, normal: Vec3, k_specular:f64, exponent: f64) -> Self {
        let reflected = r_in_direction.unit().reflect(&normal.unit());
        let onb_reflected = ONB::build_from(&reflected);
        let onb_normal = ONB::build_from(&normal);
        Self {
            r_in_direction,
            onb_normal,
            onb_reflected,
            k_specular,
            exponent,
        }
    }
}

impl PDF for BlinnPhongPdf {

    fn value(&self, direction: &Vec3) -> f64 {
        let random_normal =
            (-&self.r_in_direction.unit() + direction.unit()).unit();
        
        let cosine = direction.unit().dot(&self.onb_normal.axis[2]);
        
        let cosine_specular = random_normal.dot(&self.onb_normal.axis[2]).max(0.0);

        let normal_pdf =
            (self.exponent + 1.0) / (2.0 * PI) * cosine_specular.powf(self.exponent);

        (cosine / PI).max(0.0)*(1.0 - self.k_specular) + normal_pdf 
            / (4.0 * (self.r_in_direction.unit() * -1.0).dot(&random_normal)) * self.k_specular
    }

/*
    fn value(&self, direction: &Vec3) -> f64 {
        let cosine_theta = direction.unit().dot(&self.onb_normal.axis[2]);
        let v = cosine_theta / PI;

        if v < 0.0 { 0.0 } else { v }
    }
*/

    fn generate(&self) -> Vec3 {
        if Random::gen() < self.k_specular {
            loop {
                
                let direction =
                    self
                    .onb_reflected
                    .local(&Vec3::random_cosine_direction_exponent(self.exponent));
                if direction.dot(&self.onb_normal.axis[2]) > 0.0 {
                    return direction;
                }
            }
        }
        self.onb_normal.local(&Vec3::random_cosine_direction())
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

    fn value(&self, _direction: &Vec3) -> f64 {
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
        // Vec3::new(0.0, 1.0, 0.0)    
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
