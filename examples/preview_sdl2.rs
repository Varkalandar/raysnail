extern crate sdl2;

#[allow(dead_code)]
mod common;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color as SDLColor;
use sdl2::video::Window;
use sdl2::render::WindowCanvas;
use sdl2::rect::Point;
use sdl2::render::TextureAccess;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Texture;
use sdl2::rect::Rect;
use sdl2::render::TextureCreator;
use sdl2::surface::SurfaceContext;
use sdl2::video::WindowContext;

use std::time::Duration;
use std::sync::Arc;

use raysnail::prelude::Ray;
use raysnail::prelude::Color;
use raysnail::prelude::Vec3;
use raysnail::prelude::Point3;
use raysnail::camera::CameraBuilder;

use raysnail::painter::PainterTarget;
use raysnail::painter::PainterCommand;
use raysnail::painter::PainterController;
use raysnail::material::*;
use raysnail::hittable::Sphere;
use raysnail::hittable::Box;
use raysnail::hittable::AARect;
use raysnail::hittable::AARectMetrics;
use raysnail::hittable::geometry::RayMarcher;
use raysnail::hittable::geometry::TriangleMesh;
use raysnail::hittable::collection::HittableList;
use raysnail::texture::Checker;
use raysnail::sdl_parser::SdlParser;

use rayon::spawn;
use std::sync::mpsc::sync_channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;


pub struct Renderer { 
    canvas: WindowCanvas, 
}


impl Renderer {

    pub fn new(window: Window) -> Renderer {
        let mut canvas = window.into_canvas().build().map_err(|e| e.to_string()).unwrap();
        canvas.set_draw_color(SDLColor::RGB(64, 64, 64));
        canvas.clear();

        Renderer { 
            canvas,
        }
    }

    pub fn flush_line(&mut self, y: usize, colors: &Vec<u8>, line: &mut Texture) {

        let (width, height) = self.canvas.output_size().unwrap();

        let r = Rect::new(0, y as i32, width, 1);
        line.update(Some(r), colors, width as usize * 3).unwrap();

        let s = Rect::new(0, 0, width, height);
        let d = Rect::new(0, 0, width, height);
        self.canvas.copy(&line, Some(s), Some(d)).unwrap();
    }

    pub fn present(&mut self) {
        self.canvas.present();
        // ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
    }
}


struct PixelQueue {
    sender: SyncSender<(usize, Vec<[u8; 4]>)>,
    // command_receiver: Receiver<PainterCommand>,
}

impl PainterTarget for PixelQueue {
    fn register_pixels(&self, y: usize, pixels: &Vec<[u8; 4]>) {
        // println!("Got {} pixels", pixels.len());

        let status = self.sender.send((y, pixels.clone()));

        if status.is_err() {
        }
    }
}

struct RenderPainterController {
    command_receiver: Receiver<PainterCommand>,
}

impl PainterController for RenderPainterController {
    fn receive_command(&self) -> PainterCommand {
        let status = self.command_receiver.try_recv();
        let mut result = PainterCommand::None;

        if status.is_ok() {
            result = status.unwrap();
            println!("PainterTarget is requesting Quit");
        }

        result
    }
}

pub fn main() -> Result<(), String> {

    let (sender, receiver) = sync_channel::<(usize, Vec<[u8; 4]>)>(1 << 16);
    let (command_sender, command_receiver) = sync_channel::<PainterCommand>(256);

    let mut queue = PixelQueue {sender};
    let mut controller = RenderPainterController {command_receiver};

    let width:usize = 1000;
    let height:usize = 600;

    spawn(move || boot_sdl(width, height, receiver, command_sender));

    // render_ball_scene(width, height, &mut queue, &mut controller);
    // render_time_test(width, height, &mut queue, &mut controller);
    // render_raymarching_test(width, height, &mut queue, &mut controller);
    // render_object_test(width, height, &mut queue, &mut controller);
    render_parser_test(width, height, &mut queue, &mut controller);

    Ok(())
}


fn boot_sdl(width: usize, height: usize, receiver: Receiver<(usize, Vec<[u8; 4]>)>, command_sender: SyncSender<PainterCommand>) {
    common::init_log("info");

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();


    let window = video_subsystem
        .window("rust-sdl2 demo: Video", width as u32, height as u32)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string()).unwrap();

    // let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    let mut renderer = Renderer::new(window);
    let mut pixels: Vec<u8> = Vec::with_capacity(width * 3);

    let creator = 
        renderer.canvas.texture_creator();
    let mut line =
        creator 
        .create_texture(PixelFormatEnum::RGB24, TextureAccess::Static, width as u32, height as u32).unwrap();
        // .create_texture(None, TextureAccess::Static, width, 1).unwrap();

    println!("Color mod={:?}", line.color_mod());
    println!("query={:?}", line.query());

    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        let data_result = receiver.recv();

        if data_result.is_ok() {
            let (y, data) = data_result.unwrap();

            for pixel in data {
                pixels.push(pixel[0]);
                pixels.push(pixel[1]);
                pixels.push(pixel[2]);
            }

            renderer.flush_line(y, &pixels, &mut line);

            renderer.present();
            pixels.clear();
        } 
        else {
            let error = data_result.err().unwrap();
            println!("Receiving window could not read pixels: {:?}", error.to_string());        
        }
    }

    let status = command_sender.send(PainterCommand::Quit);

    println!("Sending Quit to render engine. Status={:?}", status);
}


fn render_time_test(width: usize, height: usize, 
                    target: &mut dyn PainterTarget, controller: &mut dyn PainterController) {
    
    let builder = CameraBuilder::default()
        .look_from(Point3::new(13.0, 2.0, 3.0) * 0.5)
        .look_at(Point3::new(0.0, 0.0, 0.0))
        .fov(15.0)
        .aperture(0.01)
        .focus(10.0)
        .width(width)
        .height(height);

    let camera = builder.build();    

    let mut world = HittableList::default();
    let mut lights = HittableList::default();
/*
    let rs = 
        AARect::new_xz(AARectMetrics::new(200.0, (-15.0, 15.0), (-15.0, 15.0)),
            DiffuseLight::new(Color::new(1.0, 0.9, 0.8)).multiplier(200.0));
*/

    let rs = 
        Sphere::new(Vec3::new(50.0, 200.0, 200.0), 
            12.0, 
            Arc::new(DiffuseLight::new(Color::new(1.0, 0.9, 0.8)).multiplier(2.0))
        );

    lights.add(rs.clone());
    world.add(rs);

    world.add(Sphere::new(
        Point3::new(0.0, 0.0, 0.0),
        1.0,
        Arc::new(BlinnPhong::new(0.5, 4.0, Color::new(0.99, 0.69, 0.2))),
        // Lambertian::new(Color::new(0.99, 0.69, 0.2)),
        // DiffuseMetal::new(200.0, Color::new(0.99, 0.69, 0.2)),
    ));

/*
    let color = Color::new(0.99, 0.8, 0.2);
    // let material = Lambertian::new(color);
    let material = BlinnPhong::new(0.5, 4.0, color);
    // let material = Metal::new(color);
    world.add(RayMarcher::new(material));
*/    

    world.add(Sphere::new(
        Point3::new(0.0, -1001.0, 0.0),
        1000.0,
        Arc::new(Lambertian::new(Checker::new(
            Color::new(0.3, 0.3, 0.3),
            Color::new(0.1, 0.1, 0.1),
        )))
    ));

    fn background(ray: &Ray) -> Color {

        assert!((ray.direction.length_squared() - 1.0).abs() < 0.00001);

        let t = 0.5 * (ray.direction.y + 1.0);
        // Color::new(0.68, 0.80, 0.95).gradient(&Color::new(0.2, 0.4, 0.7), t)
 
        Color::new(0.9, 0.9, 0.9)
        // Color::new(0.1, 0.12, 0.3)
    }

    camera
        .take_photo_with_lights(world, lights)
        .background(background)
        .samples(5)
        //.samples(257)
        .depth(8)
        .shot_to_target(Some("rtow_13_1.ppm"), target, controller)
        .unwrap();
}


fn render_raymarching_test(width: usize, height: usize, 
                           target: &mut dyn PainterTarget, controller: &mut dyn PainterController) {
    
    let builder = CameraBuilder::default()
        .look_from(Point3::new(13.0, -1.7, 3.0) * 0.7)
        .look_at(Point3::new(0.0, -0.4, 0.0))
        .fov(20.0)
        .aperture(0.01)
        .focus(10.0)
        .width(width)
        .height(height);

    let camera = builder.build();    

    let mut world = HittableList::default();
    let mut lights = HittableList::default();

    let rs = 
        Sphere::new(Vec3::new(300.0, 400.0, 100.0), 
            12.0, 
            Arc::new(DiffuseLight::new(Color::new(1.0, 0.9, 0.8)).multiplier(1.5))
        );

    lights.add(rs.clone());
    world.add(rs);

    // let color = Color::new(0.8, 0.8, 0.8);
    let color = Color::new(0.5, 0.5, 0.5);
    let mut material = Lambertian::new(color);
    material.settings.phong_factor = 4.0;
    material.settings.phong_exponent = 2;
    // let material = BlinnPhong::new(0.5, 4.0, color);
    // let material = Metal::new(color);
    world.add(RayMarcher::new(Arc::new(material)));

    world.add(Sphere::new(
        Point3::new(0.0, -1002.0, 0.0),
        1000.0,
        Arc::new(DiffuseMetal::new(800.0, Checker::new(
            Color::new(0.26, 0.3, 0.16),
            Color::new(0.1, 0.1, 0.1),
        )))
    ));

    fn background(ray: &Ray) -> Color {

        // assert!((ray.direction.length_squared() - 1.0).abs() < 0.00001);

        let t = 0.5 * (ray.direction.y + 1.0);
        Color::new(0.68, 0.80, 0.95).gradient(&Color::new(0.2, 0.4, 0.7), t)
 
        // Color::new(0.9, 0.9, 0.9)
        // Color::new(0.06, 0.06, 0.25)
    }

    camera
        .take_photo_with_lights(world, lights)
        .background(background)
        .samples(122)
        .depth(8)
        .shot_to_target(Some("raymarching.ppm"), target, controller)
        .unwrap();
}


fn render_ball_scene(width: usize, height: usize, 
                     target: &mut dyn PainterTarget, controller: &mut dyn PainterController) {

    // Change `7` to another number to generate different scene
    // Or use `None` to use random seed
    let (camera, mut world) = common::ray_tracing_in_one_weekend::final_scene(Some(7));
    
    let camera = camera.width(width);
    let camera = camera.height(height);

    let mut lights = HittableList::default();

    let rs = 
        Sphere::new(Vec3::new(200.0, 400.0, 200.0), 
            12.0, 
            Arc::new(DiffuseLight::new(Color::new(1.0, 0.9, 0.8)).multiplier(1.5))
        );

    lights.add(rs.clone());
    world.add(rs);

    camera
        .build()
        .take_photo_with_lights(world, lights)
        .samples(26)
        //.samples(257)
        .depth(8)
        .shot_to_target(Some("rtow_13_1.ppm"), target, controller)
        .unwrap();
}


fn render_object_test(width: usize, height: usize, 
    target: &mut dyn PainterTarget, controller: &mut dyn PainterController) {

    let builder = CameraBuilder::default()
        .look_from(Point3::new(13.0, 1.5, 3.0) * 1.0)
        .look_at(Point3::new(0.0, 0.3, 0.0))
        .fov(20.0)
        .aperture(0.01)
        .focus(10.0)
        .width(width)
        .height(height);

    let camera = builder.build();    

    let mut world = HittableList::default();
    let mut lights = HittableList::default();

    let rs = 
        Sphere::new(Vec3::new(300.0, 400.0, 100.0), 
            12.0, 
            Arc::new(DiffuseLight::new(Color::new(1.0, 0.9, 0.8)).multiplier(1.5))
        );

    lights.add(rs.clone());
    world.add(rs);

    // let color = Color::new(0.8, 0.8, 0.8);
    let color = Color::new(0.9, 0.3, 0.1);
    let mut material = Lambertian::new(color);
    material.settings.phong_factor = 4.0;
    material.settings.phong_exponent = 4;

    let mesh = TriangleMesh::load(
        "objects/dragon.obj",
        0.2, // scale: f64,
        Vec3::new(0.0, 0.0, 0.0), // offset: Vec3,
        120.0, // rotation_angle: f64,
        1, // axis: i32,
        Arc::new(material),
    );

    for tri in mesh.triangles {
        world.add(tri);
    }

    world.add(Sphere::new(
            Point3::new(0.0, -1000.0, 0.0),
            1000.0,
            // DiffuseMetal::new(5000.0, Checker::new(
            // Metal::new(Checker::new(
            //    Color::new(0.26, 0.3, 0.16),
            //    Color::new(0.08, 0.1, 0.06),
            //    )
            Arc::new(Metal::new(Color::new(0.08, 0.1, 0.06)))
    ));

    fn background(ray: &Ray) -> Color {

    // assert!((ray.direction.length_squared() - 1.0).abs() < 0.00001);

    let t = 0.5 * (ray.direction.y + 1.0);
    Color::new(0.68, 0.80, 0.95).gradient(&Color::new(0.2, 0.4, 0.7), t)

    // Color::new(0.9, 0.9, 0.9)
    // Color::new(0.06, 0.06, 0.25)
    }

    camera
    .take_photo_with_lights(world, lights)
    .background(background)
    .samples(122)
    .depth(8)
    .shot_to_target(Some("raymarching.ppm"), target, controller)
    .unwrap();
}


fn render_parser_test(width: usize, height: usize, 
                      target: &mut dyn PainterTarget, controller: &mut dyn PainterController) {


    let mut scene_data = SdlParser::parse("sdl/example.sdl");
    let camera_data = &scene_data.camera.unwrap();

    let builder = CameraBuilder::default()
        .look_from(camera_data.location.clone())
        .look_at(camera_data.look_at.clone())
        .fov(20.0)
        .aperture(0.01)
        .focus(10.0)
        .width(width)
        .height(height);

    let camera = builder.build();    

    let mut lights = HittableList::default();

    let rs = 
        Sphere::new(Vec3::new(300.0, 400.0, 100.0), 
            12.0, 
            Arc::new(DiffuseLight::new(Color::new(1.0, 0.9, 0.8)).multiplier(1.5))
        );

    lights.add(rs.clone());
    scene_data.hittables.add(rs);

    fn background(ray: &Ray) -> Color {

    // assert!((ray.direction.length_squared() - 1.0).abs() < 0.00001);

    let t = 0.5 * (ray.direction.y + 1.0);
    Color::new(0.68, 0.80, 0.95).gradient(&Color::new(0.2, 0.4, 0.7), t)

    // Color::new(0.9, 0.9, 0.9)
    // Color::new(0.06, 0.06, 0.25)
    }

    camera
        .take_photo_with_lights(scene_data.hittables, lights)
        .background(background)
        .samples(122)
        .depth(8)
        .shot_to_target(Some("sample_scene.ppm"), target, controller)
        .unwrap();
}
