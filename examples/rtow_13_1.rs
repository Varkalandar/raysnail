use std::sync::Arc;

use raysnail::prelude::Color;
use raysnail::prelude::Vec3;
use raysnail::prelude::Ray;
use raysnail::material::DiffuseLight;
use raysnail::hittable::Sphere;
use raysnail::hittable::collection::HittableList;
use raysnail::hittable::collection::World;


#[allow(dead_code)]
mod common;

fn main() {
    common::init_log("info");

    // Change `7` to another number to generate different scene
    // Or use `None` to use random seed
    let (camera_builder, mut world) = common::ray_tracing_in_one_weekend::final_scene(Some(7));

    let mut lights = HittableList::default();

    let rs = 
        Sphere::new(Vec3::new(300.0, 400.0, 100.0), 
            12.0, 
            Some(Arc::new(DiffuseLight::new(Color::new(1.0, 0.9, 0.7, 1.0)).multiplier(1.5)))
        );

    lights.add(rs.clone());
    world.add(rs);

    let camera = camera_builder
        .width(800)
        .height(500)
        .build();

    fn background(ray: &Ray) -> Color {
        let t = (ray.direction.y + 1.0) * 0.5;  // norm to range 0..1
        Color::new(0.3, 0.4, 0.5, 1.0).gradient(&Color::new(0.7, 0.89, 1.0, 1.0), t)
    }
    
    let world = World::new(world, 
                           lights, 
                           background,
                           &(0.0 .. camera.shutter_speed));

    camera
        .take_photo()
        .samples(122)
        .shot(Some("rtow_13_1.ppm"), &world);
}
