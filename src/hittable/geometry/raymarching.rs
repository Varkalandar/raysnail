use std::ops::Range;
use std::sync::Arc;
use std::fmt::Formatter;
use std::fmt::Debug;

use crate::prelude::PI;
use crate::prelude::Vec3;
use crate::prelude::Point3;
use crate::prelude::AABB;
use crate::prelude::Ray;
use crate::prelude::FastRng;
use crate::hittable::HitRecord;
use crate::hittable::Hittable;
use crate::material::Material;


#[derive(Clone)]
pub struct RayMarcher {

    material: Arc<dyn Material>,
}

impl Debug for RayMarcher {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Raymarcher"
        ))
    }
}

impl RayMarcher {
    pub fn new(material: Arc<dyn Material>) -> Self {
        RayMarcher {
            material,
        }
    }

    fn binary_search_surface(outside: &Vec3, inside: &Vec3, depth: usize) -> Option<Vec3> {

        if depth <= 0 {
            return Some(outside.clone());
        }

        let midpoint = (outside + inside) * 0.5;

        if is_inside(&midpoint, 100) {
            return Self::binary_search_surface(outside, &midpoint, depth - 1);
        }
        else {
            return Self::binary_search_surface(&midpoint, inside, depth - 1);
        }
    }

    fn search_surface(&self, p: &Vec3, direction: &Vec3) -> Option<Vec3> {
    
        let df = direction * 0.05;
        let mut v = p.clone();
    
        for _i in 0 .. 200 {
            // println!("step {}", i);
            if is_inside(&v, 100) {
                // println!("step {} is inside at {:?}", i, v);
                // return Some(v - &df);
                return Self::binary_search_surface(&(&v - &df), &v, 8);
            }
            v += &df;
        }

        // println!("{:?} is outside", v);
        None
    }
}


impl Hittable for RayMarcher {
    fn normal(&self, pos: &Point3) -> crate::prelude::Vec3 {
     
        let d = 0.01;
        let xdir = Vec3::new(d, 0.0, 0.0);
        let ydir = Vec3::new(0.0, d, 0.0);
        let zdir = Vec3::new(0.0, 0.0, d);

        Vec3::new(
            distance_est(pos+&xdir, 100) - distance_est(pos-&xdir, 100),
            distance_est(pos+&ydir, 100) - distance_est(pos-&ydir, 100),
            distance_est(pos+&zdir, 100) - distance_est(pos-&zdir, 100)
        ).unit()

    }

    fn material(&self) -> Arc<dyn Material> {
        self.material.clone()
    }

    fn uv(&self, point: &Point3) -> (f64, f64) {
        let center = Vec3::new(0.0, 0.0, 0.0);

        let point = (point - &center).unit();
        let phi = (-point.z).atan2(point.x); // [-pi, pi]
        let theta = point.y.asin(); // [-pi / 2 , pi / 2]
        let u = phi / 2.0 / PI + 0.5;
        let v = theta / PI + 0.5;
        (u, v)
    }

    fn hit(&self, ray: &Ray, unit_limit: &Range<f64>) -> Option<HitRecord> {
        
        let ray_direction_length = ray.direction.length();
        let direction = &ray.direction * (1.0 / ray_direction_length);
        let center = Vec3::new(0.0, 0.0, 0.0);
        let mut current = ray.origin.clone();

        // walk the ray in steps

        let mut steps = 0;
        let mut best_distance = 1000000.0;
        let mut est_distance = 0.0;
        let limit = 1000;

        while steps < limit && est_distance < best_distance + 1.0 {
            
            steps += 1;

            // estimated distance
            est_distance = (&current - &center).length();
            // println!("Estimated distance from {} is {} units", current, est_distance);

            if est_distance < 1.3 {
                let check = self.search_surface(&current, &direction);

                if check.is_some() {                    
                    let p = check.unwrap();
                    let length = (p - &ray.origin).length();
                    if length > unit_limit.start {
                        let t1 = length / ray_direction_length;
                        return Some(HitRecord::new(ray, self, t1, t1));
                    }
                }
                
                return None;
            }

            if est_distance < best_distance {
                best_distance = est_distance;
            }

            // forward 
            current += &direction * est_distance * 0.05;
        }

        // println!("Marcher quit after {} steps", limit);
        None
    }

    fn contains(&self, point: &Vec3) -> bool
    {
        is_inside(point, 100)
    }

    fn bbox(&self, _time_limit: &Range<f64>) -> Option<AABB> {
        let center = Vec3::new(0.0, 0.0, 0.0);
        let radius = 1.3;
        Some(
            AABB::new(
                &center - Vec3::new(radius, radius, radius),
                &center + Vec3::new(radius, radius, radius),
            )
        )
    }


    /**
     * This is only called if the object is a light source. It is used to generate
     * an extra ray towards the light source.
     */
    fn random(&self, _origin: &Point3, rng: &mut FastRng) -> Vec3 {
        Vec3::random_unit(rng)
    }
}


pub fn is_inside(p: &Vec3, iterations: i32) -> bool {

    let mut x: f64 = 0.0;
    let mut y: f64 = 0.0;
    let mut z: f64 = 0.0;
    let power: f64 = 8.0;

    for _i in 0 .. iterations {
        //Convert to spherical coordinates
        let mut r: f64 = (x*x + y*y + z*z).sqrt();
        // let mut theta: f64 = (z / r).acos();
        let mut theta: f64 = (x*x + y*y).sqrt().atan2(z);
        let mut phi: f64 = y.atan2(x);

        //Scale and rotate
        r = r.powf(power);
        theta *= power;
        phi *= power;

        //Convert back to cartesian coordinates
        x = r * theta.sin() * phi.cos();
        y = r * theta.sin() * phi.sin(); 
        z = r * theta.cos();

        //Add c
        x += p.x;
        y += p.y;
        z += p.z;

        //Check if the radius is not beyond the bailout.
        if x*x + y*y + z*z > 8.0 {
            return false;
        }
    }

    true
}


pub fn distance_est(p: Vec3, iterations: i32) -> f64 {

    let mut x: f64 = 0.0;
    let mut y: f64 = 0.0;
    let mut z: f64 = 0.0;
    let power: f64 = 8.0;
    let mut r: f64 = 0.0;
    let mut dr: f64 = 0.0;

    for _i in 0 .. iterations {
        //Convert to spherical coordinates
        r = (x*x + y*y + z*z).sqrt();
        let mut theta: f64 = (x*x + y*y).sqrt().atan2(z);
        let mut phi: f64 = y.atan2(x);

        //Scale and rotate
        r = r.powf(power);
        theta *= power;
        phi *= power;
        dr =  r.powf(power-1.0) * power * dr + 1.0;

        //Convert back to cartesian coordinates
        x = r * theta.sin() * phi.cos();
        y = r * theta.sin() * phi.sin(); 
        z = r * theta.cos();

        //Add c
        x += p.x;
        y += p.y;
        z += p.z;

        //Check if the radius is not beyond the bailout.
        if x*x + y*y + z*z > 8.0 {
            break;
        }
    }

    0.5 * r.ln() * r / dr
}
