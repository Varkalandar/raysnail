use crate::prelude::Vec3;
use crate::prelude::ONB;
use crate::prelude::PI;



pub trait PDF {
    fn value(direction: &Vec3) -> f64;
    fn generate() -> Vec3;
}


#[derive(Debug)]
pub struct CosinePdf {
    uvw: ONB,
}

impl CosinePdf {
    
    pub fn new(w: &Vec3) -> Self { 
        CosinePdf {
            uvw: ONB::build_from_w(w),
        } 
    }
  
    pub fn value(&self, direction: &Vec3) -> f64 {
        let cosine_theta = direction.unit().dot(&self.uvw.axis[2]);
        let v = cosine_theta / PI;

        if v < 0.0 { 0.0 } else { v }
    }
  
    pub fn generate(&self, ) -> Vec3 {
        self.uvw.vec_local(&Vec3::random_cosine_direction())
    }
}

