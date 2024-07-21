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

// Quadric normal form
// qa x^2 + qe y^2 + qh z^2 + 2 qb xy + 2 qc xz + 2 qf yz + 2 qd x + 2 qg y + 2 qi z + j = 0


#[derive(Clone)]
pub struct Quadric {

    qa: f64,
    qb: f64,
    qc: f64,
    qd: f64,
    qe: f64,
    qf: f64,
    qg: f64,
    qh: f64,
    qi: f64,
    qj: f64,

    material: Arc<dyn Material>,
}

impl Debug for Quadric {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Quadric",
        ))
    }
}


impl Quadric {
    pub fn new(qa: f64,
           qb: f64,
           qc: f64,
           qd: f64,
           qe: f64,
           qf: f64,
           qg: f64,
           qh: f64,
           qi: f64,
           qj: f64,
            material: Arc<dyn Material>
    ) -> Self {
        Quadric {
            qa, qb, qc, qd, qe, qf, qg, qh, qi, qj, material            
        }
    }
}


impl Hittable for Quadric {

    fn normal(&self, point: &Point3) -> Vec3 {
        // println!("Quadric: normal called");
    
        let x = 2.0 * self.qa * point.x +
                      self.qb * point.y +
                      self.qc * point.z +
                      self.qd;
    
        let y =       self.qb * point.x +
                2.0 * self.qe * point.y +
                      self.qf * point.z +
                      self.qg;
    
        let z =       self.qc * point.x +
                      self.qf * point.y +
                2.0 * self.qh * point.z +
                      self.qi;
    
        let result = Vec3::new(x, y, z);

        let len = result.length();
    
        if len == 0.0
        {
            // The normal is not defined at this point of the surface.
            // Set it to any arbitrary direction.
    
            return Vec3::new(1.0, 0.0, 0.0);
        }
        else
        {
            return result / len;
        }
    }

    fn material(&self) -> Arc<dyn Material> {
        self.material.clone()
    }

    fn uv(&self, _point: &Point3) -> (f64, f64) {
        let u = 0.0;
        let v = 0.0;
        (u, v)
    }

    fn hit(&self, ray: &Ray, unit_limit: &Range<f64>) -> Option<HitRecord> {

        let xo = ray.origin.x;
        let yo = ray.origin.y;
        let zo = ray.origin.z;

        let xd = ray.direction.x;
        let yd = ray.direction.y;
        let zd = ray.direction.z;

        let a = xd * (self.qa * xd + self.qb * yd + self.qc * zd) +
                yd * (self.qe * yd + self.qf * zd) +
                zd *  self.qh * zd;

        let b = xd * (self.qa * xo + 0.5 * (self.qb * yo + self.qc * zo + self.qd)) +
                yd * (self.qe * yo + 0.5 * (self.qb * xo + self.qf * zo + self.qg)) +
                zd * (self.qh * zo + 0.5 * (self.qc * xo + self.qf * yo + self.qi));

        let c = xo * (self.qa * xo + self.qb * yo + self.qc * zo + self.qd) +
                yo * (self.qe * yo + self.qf * zo + self.qg) +
                zo * (self.qh * zo + self.qi) +
                self.qj;


        if a == 0.0
        {
            // There are no quadratic terms. Solve the linear equation instead.

            if b == 0.0
            {
                // println!("Quadric: No intersection");
                return None;
            }

            let t1 = - 0.5 * c / b;

            // println!("Quadric: 1 intersection");
            if unit_limit.contains(&t1) {
                return Some(HitRecord::new(ray, self, t1, f64::MAX));
            }
        }
        else
        {
            // The equation is quadratic - find its roots

            let d = b * b - a * c;

            if d <= 0.0
            {
                // println!("Quadric: No intersection");
                return None;
            }

            let dr = d.sqrt();

            let t1 = (-b - dr) / a;
            let t2 = (-b + dr) / a;
            // println!("Quadric: 2 intersections {}, {}", t1, t2);

            if unit_limit.contains(&t1) {
                return Some(HitRecord::new(ray, self, t1, t2));
            }
    
            if unit_limit.contains(&t2) {
                return Some(HitRecord::new(ray, self, t2, f64::MAX));
            }
        }

        // println!("Quadric: No intersection in given range");
        None
    }

    fn contains(&self, point: &Vec3) -> bool
    {
        return (point.x * (self.qa * point.x + self.qb * point.y + self.qd) +
                point.y * (self.qe * point.y + self.qf * point.z + self.qg) +
                point.z * (self.qh * point.z + self.qc * point.x + self.qi) + self.qj) <= 0.0;
    }    

    fn bbox(&self, _time_limit: &Range<f64>) -> Option<AABB> {
        Some(AABB::new(
            Vec3::new(-100.0, -100.0, -100.0),
            Vec3::new(100.0, 100.0, 100.0),
        ))
    }

    /**
     * This is only called if the object is a light source. It is used to generate
     * an extra ray towards the light source.
     */
     fn random(&self, origin: &Point3, _rng: &mut FastRng) -> Vec3 {

        - origin
    }
}