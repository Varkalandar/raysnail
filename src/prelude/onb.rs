use crate::prelude::Vec3;

#[derive(Debug)]
pub struct ONB {
    pub axis: [Vec3; 3],
}    

// orthonormal base
impl ONB {

    pub fn local(&self, a: &Vec3) -> Vec3 {
        // self.axis[0].clone() * a.x + self.axis[1].clone() * a.y + self.axis[2].clone() * a.z    

        let a0 = &self.axis[0];
        let a1 = &self.axis[1];
        let a2 = &self.axis[2];

        Vec3::new(
            a0.x * a.x + a1.x * a.y + a2.x * a.z,
            a0.y * a.x + a1.y * a.y + a2.y * a.z,
            a0.z * a.x + a1.z * a.y + a2.z * a.z)

    }
  
    pub fn build_from(n: &Vec3) -> ONB {
        let w = n.unit();
        let up = Vec3::new(0.0, 1.0, 0.0);

        let uc = up.cross(&w);        
        let u = if uc.length_squared() < 0.00000001 {
                Vec3::new(1.0, 0.0, 0.0).cross(&w).unit()
            } else{
                uc.unit()
            };

        let v = w.cross(&u);

        ONB {
            axis: [u, v, w],
        }
    }
}