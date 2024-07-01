use remda::{painter::Painter, prelude::*};
use remda::painter::PassivePainterTarget;

fn main() {
    env_logger::init();

    Painter::new(256, 256)
        .gamma(false)
        .samples(1)
        .draw(&Some("rtow_2_2.ppm"), &mut PassivePainterTarget {}, |u, v| Vec3::new(u, v, 0.25))
        .unwrap()
}
