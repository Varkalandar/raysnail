use std::ops::Range;
use std::sync::Arc;
use std::fmt::Formatter;
use std::fmt::Debug;

use crate::prelude::Vec3;
use crate::prelude::Point3;
use crate::material::Material;
use crate::hittable::Hittable;
use crate::hittable::HitRecord;
use crate::prelude::Ray;
use crate::prelude::AABB;
use crate::prelude::FastRng;


#[derive(Clone)]
pub struct Triangle {
    p0: Vec3,
    normal0: Vec3,
    normal1: Vec3,
    normal2: Vec3,
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
    pub f: f64,
    bounding_box: AABB,
    material: Option<Arc<dyn Material>>,
}

impl Debug for Triangle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Triangle {{ p0: {:?} }}",
            self.p0,
        ))
    }
}

impl Triangle {
    pub fn new(p0: Vec3, p1: Vec3, p2: Vec3, material: Option<Arc<dyn Material>>) -> Self {
        let minimum = Vec3::new_min(&(Vec3::new_min(&p0, &p1)), &p2);
        let maximum = Vec3::new_max(&(Vec3::new_max(&p0, &p1)), &p2);
        let bounding_box = AABB::new(minimum, maximum);
        Self {
            p0: p0.clone(),
            normal0: Vec3::new(0.0, 0.0, 0.0),
            normal1: Vec3::new(0.0, 0.0, 0.0),
            normal2: Vec3::new(0.0, 0.0, 0.0),
            a: &p0.x - &p1.x,
            b: &p0.y - &p1.y,
            c: &p0.z - &p1.z,
            d: &p0.x - &p2.x,
            e: &p0.y - &p2.y,
            f: &p0.z - &p2.z,
            bounding_box,
            material,
        }
    }

    pub fn set_normals(
        &mut self,
        normal0: Vec3,
        normal1: Vec3,
        normal2: Vec3,
    ) {
        self.normal0 = normal0;
        self.normal1 = normal1;
        self.normal2 = normal2;
    }
}

impl Hittable for Triangle {

    fn material(&self) -> Option<Arc<dyn Material>> {
        self.material.clone()
    }

    fn uv(&self, _point: &Point3) -> (f64, f64) {
        (0.0, 0.0)
    }

    // #[inline(always)]
    fn hit(&self, ray: &Ray, unit_limit: &Range<f64>) -> Option<HitRecord> {
        let g = ray.direction.x;
        let h = ray.direction.y;
        let i = ray.direction.z;
        let j = self.p0.x - ray.origin.x;
        let k = self.p0.y - ray.origin.y;
        let l = self.p0.z - ray.origin.z;

        let eihf = self.e * i - h * self.f;
        let gfdi = g * self.f - self.d * i;
        let dheg = self.d * h - self.e * g;

        let denom = self.a * eihf + self.b * gfdi + self.c * dheg;
        let beta = (j * eihf + k * gfdi + l * dheg) / denom;

        if beta < 0.0 || beta >= 1.0 {
            return None;
        }

        let akjb = self.a * k - j * self.b;
        let jcal = j * self.c - self.a * l;
        let blkc = self.b * l - k * self.c;

        let gamma = (i * akjb + h * jcal + g * blkc) / denom;
        if gamma <= 0.0 || beta + gamma >= 1.0 {
            return None;
        }

        let t = -(self.f * akjb + self.e * jcal + self.d * blkc) / denom;
        if t >= unit_limit.start && t <= unit_limit.end {
            let normal =
            &self.normal0 * (1.0 - beta - gamma) + &self.normal1 * beta + &self.normal2 * gamma;

            let point = ray.at(t);

            Some(HitRecord::with_normal(
                point.clone(),
                normal,
                self.material(),
                self.uv(&point),
                t,
                f64::MAX,
            ))
        } else {
            None
        }
    }

    fn contains(&self, _point: &Vec3) -> bool
    {
        false
    }

    fn bbox(&self, _time_limit: &Range<f64>) -> Option<AABB> {
        Some(self.bounding_box.clone())
    }

    /**
     * This is only called if the object is a light source. It is used to generate
     * an extra ray towards the light source.
     */
     fn random(&self, origin: &Point3, _rng: &mut FastRng) -> Vec3 {
        origin - &self.p0
     }
}


pub struct TriangleMesh {
    pub triangles: Vec<Triangle>,
}

impl Debug for TriangleMesh {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "TriangleMesh {{ triangles: {:?} }}",
            self.triangles,
        ))
    }
}

impl TriangleMesh {
    pub fn load(
        filename: &str,
        scale: f64,
        offset: Vec3,
        rotation_angle: f64,
        axis: i32,
        material: Option<Arc<dyn Material>>,
    ) -> Self {

        let object = tobj::load_obj(
            filename,
            &tobj::LoadOptions {
                single_index: true,
                triangulate: true,
                ignore_points: false,
                ignore_lines: false,
            },
        );
        assert!(object.is_ok());

        let mut triangles = vec![];
        let cos = rotation_angle.to_radians().cos();
        let sin = rotation_angle.to_radians().sin();

        let (models, _) = object.expect("Failed to load OBJ file");
        let mut i_t = 0;
        for (m_i, m) in models.iter().enumerate() {
            let mesh = &m.mesh;
            println!("loading model {}: \'{}\' with {} vertices", m_i, m.name, mesh.positions.len() / 3);

            let mut v_normal = vec![Vec3::new(0.0, 0.0, 0.0); mesh.indices.len() / 3];
            assert!(mesh.positions.len() % 3 == 0);
            for i in 0..mesh.indices.len() / 3 {
                let ind0 = mesh.indices[3 * i] as usize;
                let ind1 = mesh.indices[3 * i + 1] as usize;
                let ind2 = mesh.indices[3 * i + 2] as usize;

                let p0: Vec3 = Vec3::new(
                    mesh.positions[3 * ind0] as f64,
                    mesh.positions[3 * ind0 + 1] as f64,
                    mesh.positions[3 * ind0 + 2] as f64,
                );
                let p1 = Vec3::new(
                    mesh.positions[3 * ind1] as f64,
                    mesh.positions[3 * ind1 + 1] as f64,
                    mesh.positions[3 * ind1 + 2] as f64,
                );
                let p2 = Vec3::new(
                    mesh.positions[3 * ind2] as f64,
                    mesh.positions[3 * ind2 + 1] as f64,
                    mesh.positions[3 * ind2 + 2] as f64,
                );

                let p0 = p0.rotate(axis, cos, sin).clone();
                let p1 = p1.rotate(axis, cos, sin).clone();
                let p2 = p2.rotate(axis, cos, sin).clone();

                if mesh.normals.is_empty() {
                    let a = &p1 - &p0;
                    let b = &p2 - &p0;
                    let normal = a.cross(&b).unit();
                    v_normal[ind0] += &normal;
                    v_normal[ind1] += &normal;
                    v_normal[ind2] += &normal;
                }

                // triangles.push(Object::get_triangles_vertices(
                triangles.push(
                        Triangle::new(
                    p0 * scale + offset.clone(),
                    p1 * scale + offset.clone(),
                    p2 * scale + offset.clone(),
                    material.clone(),
                ));
            }
            for i in 0..mesh.indices.len() / 3 {
                let ind0 = mesh.indices[3 * i] as usize;
                let ind1 = mesh.indices[3 * i + 1] as usize;
                let ind2 = mesh.indices[3 * i + 2] as usize;

                if mesh.normals.is_empty(){
                triangles[i+i_t].set_normals(
                    v_normal[ind0].unit(),
                    v_normal[ind1].unit(),
                    v_normal[ind2].unit(),
                )
            } else{
                let mut normals = Vec::with_capacity(3);
                for ind in [ind0,ind1,ind2].iter(){
                    let normal_x = mesh.normals[3**ind] as f64;
                    let normal_y = mesh.normals[3**ind+1] as f64;
                    let normal_z = mesh.normals[3**ind+2] as f64;
                    normals.push(Vec3::new(normal_x,normal_y,normal_z).rotate(axis, cos, sin));
                }
                /*dbg!(&normals,v_normal[ind0].unit(),
                v_normal[ind1].unit(),
                v_normal[ind2].unit(),);*/
                triangles[i+i_t].set_normals(
                    normals[0].clone(), 
                    normals[1].clone(), 
                    normals[2].clone()
                )
            }
            }
            i_t+=mesh.indices.len() / 3;
        }

        Self { 
            triangles, 
        }
    }

    /*
    #[allow(dead_code)]
    pub fn rotate_y(mut self, angle: f64) -> TriangleMesh {
        self.triangles
            .iter_mut()
            .for_each(|face| *face = face.clone().rotate_y(angle));
        self
    }

    #[allow(dead_code)]
    pub fn translate(mut self, offset: Vec3) -> TriangleMesh {
        self.triangles
            .iter_mut()
            .for_each(|face| *face = face.clone().translate(offset));
        self
    }
    */

/*
    pub fn push_to_objects(&mut self, objects: &mut Vec<Object>) {
        objects.extend(mem::take(&mut self.triangles));
    }
*/
}
