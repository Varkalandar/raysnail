use crate::prelude::Vec3;

#[derive(Debug)]
pub struct ONB {
    pub axis: [Vec3; 3],
}    

// orthonormal base
impl ONB {

    pub fn local(&self, a: f64, b: f64, c: f64) -> Vec3 {
        &self.axis[0] * a + &self.axis[1] * b + &self.axis[2] * c
    }
  
    pub fn vec_local(&self, a: &Vec3) -> Vec3 {
        self.local(a.x, a.y, a.z)
    }
  
    pub fn build_from_w(w: &Vec3) -> ONB {
        let unit_w = w.unit();

        let a = if unit_w.x.abs() > 0.9 {Vec3::new(0.0, 1.0, 0.0)} else {Vec3::new(1.0, 0.0, 0.0)};
        let v = unit_w.cross(&a).unit();        
        let u = unit_w.cross(&v);

        ONB {
            axis: [u, v, unit_w],
        }
    }
}