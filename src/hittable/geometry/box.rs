use {
    crate::{
        hittable::{collection::HittableList, AARect, AARectMetrics, Hittable, HitRecord},
        material::Material,
        prelude::*,
    },
    std::{ops::Range, sync::Arc},
};
use std::fmt::Formatter;
use std::fmt::Debug;

pub struct Box {
    point_min: Point3,
    point_max: Point3,
    material: Arc<dyn Material>,
    faces: HittableList,
}

impl Debug for Box {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Box {{ min: {:?}, max: {:?} }}",
            self.point_min, self.point_max,
        ))
    }
}

impl Clone for Box {
    fn clone(&self) -> Self {
        Self::new_inner(
            self.point_min.clone(),
            self.point_max.clone(),
            Arc::clone(&self.material),
        )
    }
}

impl Box {
    #[allow(clippy::needless_pass_by_value)] // for api consistency
    pub fn new(p0: Point3, p1: Point3, material: Arc<dyn Material>) -> Self {
        let point_min = Point3::new_min(&p0, &p1);
        let point_max = Point3::new_max(&p0, &p1);
        let shared_material = material;
        Self::new_inner(point_min, point_max, shared_material)
    }

    #[allow(clippy::too_many_lines)]
    fn new_inner(point_min: Point3, point_max: Point3, material: Arc<dyn Material>) -> Self {
        let mut faces = HittableList::default();
        faces
            .add(AARect::new_xy(
                // back
                AARectMetrics::new(
                    point_min.z,
                    (point_min.x, point_max.x),
                    (point_min.y, point_max.y),
                ),
                Arc::clone(&material),
            ))
            .add(AARect::new_xy(
                // front
                AARectMetrics::new(
                    point_max.z,
                    (point_min.x, point_max.x),
                    (point_min.y, point_max.y),
                ),
                Arc::clone(&material),
            ))
            .add(AARect::new_yz(
                // left
                AARectMetrics::new(
                    point_min.x,
                    (point_min.y, point_max.y),
                    (point_min.z, point_max.z),
                ),
                Arc::clone(&material),
            ))
            .add(AARect::new_yz(
                // right
                AARectMetrics::new(
                    point_max.x,
                    (point_min.y, point_max.y),
                    (point_min.z, point_max.z),
                ),
                Arc::clone(&material),
            ))
            .add(AARect::new_xz(
                // down
                AARectMetrics::new(
                    point_min.y,
                    (point_min.x, point_max.x),
                    (point_min.z, point_max.z),
                ),
                Arc::clone(&material),
            ))
            .add(AARect::new_xz(
                // up
                AARectMetrics::new(
                    point_max.y,
                    (point_min.x, point_max.x),
                    (point_min.z, point_max.z),
                ),
                Arc::clone(&material),
            ));

        Self {
            point_min,
            point_max,
            material,
            faces,
        }
    }
}

impl Hittable for Box {

    fn normal(&self, _point: &Point3) -> crate::prelude::Vec3 {
        Vec3::new(0.0, 1.0, 0.0)
    }

    fn material(&self) -> Arc<dyn Material> {
        self.material.clone()
    }

    fn hit(&self, ray: &Ray, unit_limit: &Range<f64>) -> Option<HitRecord> {
        let mut hits = self.faces.hit(ray, unit_limit);

        assert!(hits.len() < 3);

        if hits.len() == 1 {
            // ray must have started inside the box, should we record this somewhere?
            return Some(hits.remove(0));
        } else if hits.len() == 2 {
            let h1 = &hits[0];
            let h2 = &hits[1];

            if h1.t1 < h2.t1 {
                return Some(HitRecord::with_normal(h1.point.clone(), h1.normal.clone(), h1.material.clone(), 
                                                   (h1.u, h1.v), h1.t1, h2.t1));
            }
            else {
                return Some(HitRecord::with_normal(h2.point.clone(), h2.normal.clone(), h2.material.clone(), 
                                                   (h2.u, h2.v), h2.t1, h1.t1));
            }
        }

        None
    }

    fn contains(&self, point: &Vec3) -> bool
    {
        point.x >= self.point_min.x && point.x <= self.point_max.x &&
        point.y >= self.point_min.y && point.y <= self.point_max.y &&
        point.z >= self.point_min.z && point.z <= self.point_max.z
    }    


    fn bbox(&self, time_limit: &Range<f64>) -> Option<AABB> {
        self.faces.bbox(time_limit)
    }

    fn random(&self, _origin: &Point3, _rng: &mut FastRng) -> Vec3 {
        Vec3::new(1.0, 0.0, 0.0)
    }
}
