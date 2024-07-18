use {
    crate::{
        hittable::{
            collection::{HittableList, World},
            Hittable,
        },
        painter::{Painter, PainterTarget, PassivePainterTarget},
        prelude::*,
    },
    std::path::Path,
};
use crate::painter::PainterController;
use crate::painter::PassivePainterController;
use log::info;

#[derive(Debug)]
pub struct Camera {
    origin: Point3,
    lb: Point3,
    horizontal_full: Vec3,
    vertical_full: Vec3,
    horizontal_unit: Vec3,
    vertical_unit: Vec3,
    aperture: f64,
    shutter_speed: f64,

    picture_width: usize,
    picture_height: usize,
}


impl Camera {

    #[allow(clippy::too_many_arguments, clippy::needless_pass_by_value)] // internal
    pub(self) fn new(
        look_from: &Point3, look_at: &Point3, vup: &Vec3, fov: f64, aspect_ratio: f64,
        aperture: f64, focus_distance: f64, shutter_speed: f64,
        picture_width: usize, picture_height: usize,            
    ) -> Self {

        // Determine viewport dimensions.
        let theta = fov.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h * focus_distance;
        let viewport_width = viewport_height * aspect_ratio;

        // Calculate the u,v,w unit basis vectors for the camera coordinate frame.
        let w = (look_at - look_from).unit();
        let horizontal_unit = w.cross(vup).unit();
        let vertical_unit = horizontal_unit.cross(&w).unit();

        // Calculate the vectors across the horizontal and down the vertical viewport edges.
        let viewport_u = viewport_width * &horizontal_unit;
        let viewport_v = viewport_height * &vertical_unit;

        // Calculate the location of the upper left pixel.
        let lb = look_from - &viewport_u / 2.0 - &viewport_v / 2.0 + focus_distance * w;

        Self {
            origin: look_from.clone(),
            lb,
            horizontal_full: viewport_u,
            vertical_full: viewport_v,
            horizontal_unit,
            vertical_unit,
            aperture,
            shutter_speed,
            picture_width,
            picture_height,
        }
    }


    #[must_use]
    pub fn ray(&self, u: f64, v: f64, rng: &mut FastRng) -> Ray {

        let rd = self.aperture / 2.0 * Vec3::random_unit_disk(rng);
        let offset = &self.horizontal_unit * rd.x + &self.vertical_unit * rd.y;
        let origin = &self.origin + offset;
        let direction = &self.lb + u * &self.horizontal_full + v * &self.vertical_full - &origin;

        Ray::new(origin, direction.unit(), self.shutter_speed * rng.gen())        
    }


    #[must_use]
    pub fn take_photo(&self, world: HittableList) -> TakePhotoSettings<'_> {
        let world = World::new(world, HittableList::default(), &(0.0..self.shutter_speed));
        TakePhotoSettings::new(self, world)
    }


    pub fn take_photo_with_lights(&self, world: HittableList, lights: HittableList) -> TakePhotoSettings<'_> {
        let world = World::new(world, lights, &(0.0..self.shutter_speed));
        TakePhotoSettings::new(self, world)
    }

}


fn phong_highlight(dir_to_light: &Vec3, ray_dir: &Vec3, normal: &Vec3, exponent: i32, factor: f64) -> f64 {
    let reflected = dir_to_light - 2.0 * dir_to_light.dot(normal) * normal;
    let specular = reflected.dot(&-ray_dir.clone()).max(0.0).powi(exponent);
    // println!("specular = {}", specular * factor);

    specular * factor
}


#[derive(Debug)]
pub struct TakePhotoSettings<'c> {
    camera: &'c Camera,
    world: World,
    depth: usize,
    gamma: bool,
    samples: usize,
    threads: usize,
    parallel: bool,
}

impl<'c> TakePhotoSettings<'c> {
    #[must_use]
    pub const fn new(camera: &'c Camera, world: World) -> Self {
        Self {
            camera,
            world,
            depth: 8,
            gamma: true,
            samples: 50,
            threads: 0,
            parallel: true,
        }
    }

    pub fn background<BG: Fn(&Ray) -> Color + Send + Sync + 'static>(mut self, bg: BG) -> Self {
        self.world.set_bg(bg);
        self
    }

    #[must_use]
    pub const fn depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }

    #[must_use]
    pub const fn gamma(mut self, gamma: bool) -> Self {
        self.gamma = gamma;
        self
    }

    #[must_use]
    pub const fn samples(mut self, samples: usize) -> Self {
        self.samples = samples;
        self
    }

    #[must_use]
    pub const fn threads(mut self, threads: usize) -> Self {
        self.threads = threads;
        self
    }

    #[must_use]
    pub const fn parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }

    fn ray_color(ray: &Ray, world: &World, depth: usize, rng: &mut FastRng) -> Vec3 {
        
        // info!("ray_color depth={}", depth);

        // If we've exceeded the ray bounce limit, no more light is gathered.
        if depth == 0 {
            return Vec3::default();
        }

        if let Some(hit) = world.hit(ray, &(0.0001..f64::INFINITY)) {
            let material = hit.material.clone();
            let emitted = material
                .emitted(hit.u, hit.v, &hit.point)
                .unwrap_or_default();

            if let Some(srec) = material.scatter(ray, &hit) {

                if srec.skip_pdf {
                    // If the material skips the pdf it must provide a ray in the record
                    if srec.ray.is_some() {
                        return emitted + srec.color * Self::ray_color(&srec.ray.unwrap(), world, depth-1, rng);
                    }
                    else {
                        // dead end material, doesn't scatter light
                        return emitted + srec.color * Vec3::new(1.0, 1.0, 1.0);
                    }
                }
/*
                let light_pdf = HittablePdf::new(&world.lights, &hit.point, &hit.normal);
                let mixture = MixturePdf::new(&light_pdf, srec.pdf.as_ref());
                let scatter_direction = mixture.generate(rng);
*/

                let mut light_multi = 1.0;
                let mut pdf_val;

                let scattered_ray = 
                    if rng.gen() < 0.5 { 
                        pdf_val = 0.3183098861837907;

                        let dir_to_light = world.lights.random(&hit.point, rng).unit();
                        let settings = material.settings();
                        if settings.phong_factor > 0.0 {
                            light_multi += phong_highlight(&-dir_to_light.clone(), &ray.direction, &hit.normal, 
                                                           settings.phong_exponent, settings.phong_factor);
                        }

                        // we must trace back some distance, a beam to the light source can easily
                        // hit the same body again, but if the ŕay starts to close, the intersection
                        // will be ignored
                        let start = ray.at(hit.t1 - 0.0002);
                        Ray::new(start, dir_to_light, ray.departure_time)
                    } 
                    else {
                        let scatter_direction = srec.pdf.generate(rng);
                        pdf_val = srec.pdf.value(&scatter_direction);
                        Ray::new(hit.point.clone(), scatter_direction, ray.departure_time)
                    };

                // println!("hit normal={:?} scatter dir={:?}", hit.normal, scatter_direction);

/*
                let light_pdf = HittablePdf::new(&world.lights, &hit.point, &scattered.direction);
                let mixture = MixturePdf::new(&light_pdf, srec.pdf.as_ref());                
                let mut pdf_val = mixture.value(&scattered.direction);

                // let light_pdf_val = CosinePdf::new(&scattered.direction).value(&scattered.direction);                
                let light_pdf_val = 0.3183098861837907;

                let mut pdf_val =
                    0.5 * srec.pdf.value(&scattered_ray.direction) +
                    0.5 * light_pdf_val;
*/

                // clean NaNs and extreme cases
                if pdf_val <= 0.0 || pdf_val != pdf_val {
                    pdf_val = 1e-5;
                }

                // let scattering_pdf_val = material.scattering_pdf(ray, &hit, &scattered_ray);
                let scattering_pdf_val = srec.pdf.value(&scattered_ray.direction);
                let pdf_multiplicator = scattering_pdf_val / pdf_val;

                let sample_color = light_multi * Self::ray_color(&scattered_ray, world, depth-1, rng);
                let color_from_scatter = (srec.color * sample_color) * pdf_multiplicator;
                
                return emitted + color_from_scatter;
            }
            
            return emitted;
        }

        // If the ray hits nothing, return the background color.
        world.background(ray).into()
    }


    /// # Errors
    /// When open or save to file failed
    #[allow(clippy::needless_pass_by_value)] // Directly used public API, add & will make it harder to use
    pub fn shot_to_target<P: AsRef<Path>>(&self, path: Option<P>, 
                                          target: &mut dyn PainterTarget,
                                          controller: &mut dyn PainterController) -> std::io::Result<()> {
        // because picture height/width is always positive and small enough in practice
        #[allow(
            clippy::cast_sign_loss,
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation
        )]
        
        Painter::new(self.camera.picture_width, self.camera.picture_height)
        .gamma(self.gamma)
        .samples(self.samples)
        .threads(self.threads)
        .parallel(self.parallel)
        .draw(&path, target, controller,
            |i: f64, j: f64, rng: &mut FastRng| -> Vec3 {

                // info!("uv_color 1 {}, {}", i, j);

                let ray = self.camera.ray(i, j, rng);
                // info!("uv_color 2 {}, {}", i, j);
                Self::ray_color(&ray, &self.world, self.depth, rng)
            })
    }


    pub fn shot<P: AsRef<Path>>(&self, path: Option<P>) -> std::io::Result<()> {
        let mut target = PassivePainterTarget {};
        let mut controller = PassivePainterController {};
        self.shot_to_target(path, &mut target, &mut controller)
    }
}


#[derive(Debug)]
pub struct CameraBuilder {
    look_from: Point3,
    look_at: Point3,
    vup: Vec3,
    fov: f64,
    aperture: f64,
    focus_distance: f64,
    shutter_speed: f64,

    picture_width: usize,
    picture_height: usize,
    aspect_ratio: f64,    
}

impl Default for CameraBuilder {
    fn default() -> Self {
        Self {
            look_from: Point3::default(),
            look_at: Point3::new(0.0, 0.0, -1.0),
            vup: Vec3::new(0.0, 1.0, 0.0),
            fov: 90.0,
            aperture: 0.0,
            focus_distance: 1.0,
            shutter_speed: 0.0,
            picture_width: 400,
            picture_height: 200,
            aspect_ratio: 2.0,
        }
    }
}

impl CameraBuilder {
    #[must_use]
    pub const fn look_from(mut self, look_from: Point3) -> Self {
        self.look_from = look_from;
        self
    }

    #[must_use]
    pub const fn look_at(mut self, look_at: Point3) -> Self {
        self.look_at = look_at;
        self
    }

    #[must_use]
    pub const fn vup(mut self, vup: Vec3) -> Self {
        self.vup = vup;
        self
    }

    #[must_use]
    pub fn fov(mut self, fov: f64) -> Self {
        debug_assert!(0.0 < fov && fov <= 180.0, "fov = {}", fov);
        self.fov = fov;
        self
    }

    #[must_use]
    pub fn aperture(mut self, aperture: f64) -> Self {
        debug_assert!(aperture >= 0.0, "aperture = {}", aperture);
        self.aperture = aperture;
        self
    }

    #[must_use]
    pub fn focus(mut self, distance: f64) -> Self {
        debug_assert!(distance >= 0.0, "distance = {}", distance);
        self.focus_distance = distance;
        self
    }

    #[must_use]
    pub fn focus_to_look_at(self) -> Self {
        let distance = (&self.look_at - &self.look_from).length();
        self.focus(distance)
    }

    #[must_use]
    pub fn shutter_speed(mut self, duration: f64) -> Self {
        debug_assert!(duration >= 0.0, "duration = {}", duration);
        self.shutter_speed = duration;
        self
    }

    pub fn width(mut self, width: usize) -> Self {
        self.picture_width = width;
        self.aspect_ratio = self.picture_width as f64 / self.picture_height as f64;

        self
    }

    #[must_use]
    pub fn height(mut self, height: usize) -> Self {
        self.picture_height = height;
        self.aspect_ratio = self.picture_width as f64 / self.picture_height as f64;

        self
    }

    #[must_use]
    pub fn build(self) -> Camera {
        Camera::new(
            &self.look_from,
            &self.look_at,
            &self.vup,
            self.fov,
            self.aspect_ratio,
            self.aperture,
            self.focus_distance,
            self.shutter_speed,
            self.picture_width,
            self.picture_height,
        )
    }
}
