use {
    super::scene,
    raysnail::{camera::Camera, hittable::collection::HittableList},
};
use crate::CameraBuilder;

#[must_use]
fn final_world(seed: Option<u64>) -> HittableList {
    scene::balls_scene(seed, false, true)
}

#[must_use]
fn final_camera() -> CameraBuilder {
    scene::balls_scene_camera(false)
}

#[must_use]
pub fn final_scene(seed: Option<u64>) -> (CameraBuilder, HittableList) {
    (final_camera(), final_world(seed))
}
