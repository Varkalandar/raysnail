use std::sync::Arc;
use rand::thread_rng;
use rand::RngCore;

use crate::prelude::Point3;
use crate::prelude::Vec3;
use crate::prelude::Ray;

use crate::material::Material;
use crate::material::CommonMaterialSettings;
use crate::material::ScatterRecord;
use crate::material::HitRecord;

use std::fmt::Formatter;
use std::fmt::Debug;

pub struct MixedMaterial {
    material_1: Arc<dyn Material>,
    material_2: Arc<dyn Material>,
    probability_1: f64, // probability to use material 1
}

impl Debug for MixedMaterial {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "MixedMaterial"
        ))
    }
}

impl MixedMaterial {
    pub fn new(material_1: Arc<dyn Material>, material_2: Arc<dyn Material>, probability_1: f64) -> Self {
        MixedMaterial {
            material_1,
            material_2,
            probability_1,
        }
    }
}

impl Material for MixedMaterial {

    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<ScatterRecord> {
        if (thread_rng().next_u32() as f64) < u32::MAX as f64 * self.probability_1 {
            self.material_1.scatter(ray, hit)
        }
        else {
            self.material_2.scatter(ray, hit)
        }
    }

    fn emitted(&self, _u: f64, _v: f64, _point: &Point3) -> Option<Vec3> {
        None
    }

    fn settings(&self) -> CommonMaterialSettings {
        self.material_1.settings()
    }
}
