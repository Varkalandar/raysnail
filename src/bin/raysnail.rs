extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color as SDLColor;
use sdl2::video::Window;
use sdl2::render::WindowCanvas;
use sdl2::render::TextureAccess;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Texture;
use sdl2::rect::Rect;

use clap::{Arg, Command};
use clap::crate_version;

use rayon::spawn;

use std::sync::Arc;
use std::sync::mpsc::sync_channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;

use raysnail::prelude::Ray;
use raysnail::prelude::Color;
use raysnail::material::DiffuseLight;
use raysnail::hittable::Sphere;

use raysnail::camera::CameraBuilder;

use raysnail::painter::PainterTarget;
use raysnail::painter::PainterCommand;
use raysnail::painter::PainterController;
use raysnail::hittable::collection::HittableList;
use raysnail::sdl_parser::SdlParser;


pub fn init_log(level: &'static str) {
    let env = env_logger::Env::default().default_filter_or(level);
    env_logger::init_from_env(env);
}


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


fn boot_sdl(width: usize, height: usize, receiver: Receiver<(usize, Vec<[u8; 4]>)>, command_sender: SyncSender<PainterCommand>) {
    
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Raysnail Render Preview", width as u32, height as u32)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string()).unwrap();

    let mut renderer = Renderer::new(window);
    let mut pixels: Vec<u8> = Vec::with_capacity(width * 3);

    let creator = renderer.canvas.texture_creator();
    let mut line =
        creator 
        .create_texture(PixelFormatEnum::RGB24, TextureAccess::Streaming, width as u32, height as u32).unwrap();

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
            // let error = data_result.err().unwrap();

            // there don't seem to be any temporary errors, so the only thing
            // left to do is to quit the loop

            break;
        }
    }

    let status = command_sender.send(PainterCommand::Quit);

    println!("Sending Quit to render engine. Status={:?}", status);
}

fn parse_and_render(width: usize, height: usize, samples: usize, filename: &str,
                    target: &mut dyn PainterTarget, controller: &mut dyn PainterController) -> bool {

    let scene_data_result = SdlParser::parse(filename);

    if let Err(message) = scene_data_result {
        println!("Could not parse scene data: {}", message);
        return false;
    } 

    let mut scene_data = scene_data_result.unwrap();
    let camera_data = &scene_data.camera.unwrap();

    let builder = CameraBuilder::default()
        .look_from(camera_data.location.clone())
        .look_at(camera_data.look_at.clone())
        .fov(camera_data.fov_angle)
        .aperture(0.01)
        .focus(10.0)
        .width(width)
        .height(height);

    let camera = builder.build();    

    let mut lights = HittableList::default();

    for light in scene_data.lights {
        let rs = 
            Sphere::new(light.location, 
                12.0, 
                Arc::new(DiffuseLight::new(light.color).multiplier(1.7))
            );

        lights.add(rs.clone());
        scene_data.hittables.add(rs);
    }

    fn background(ray: &Ray) -> Color {
        let t = (ray.direction.y + 1.0) * 0.5;  // norm to range 0..1
        Color::new(0.3, 0.4, 0.5, 1.0).gradient(&Color::new(0.7, 0.89, 1.0, 1.0), t)
    }

    camera
        .take_photo_with_lights(scene_data.hittables, lights)
        .background(background)
        .samples(samples)
        .depth(8)
        .shot_to_target(Some("sample_scene.ppm"), target, controller)
        .unwrap();

    true
}


pub fn main() -> Result<(), String> {
    let matches = Command::new("raysnail")
        .version(crate_version!())
        .author("H. Malthaner")
        .disable_help_flag(true)
        .arg(
            Arg::new("scene")
                .short('f')
                .long("scene")
                .required(true)
                .num_args(1)
                .help("Render the given scene file"),
        )
        .arg(
            Arg::new("samples")
                .short('s')
                .long("samples")
                .help("More samples per pixel improve the image quality. Usually in range 15 .. 10000"),
        )
        .arg(
            Arg::new("width")
                .short('w')
                .long("width")
                .help("Image width"),
        )
        .arg(
            Arg::new("height")
                .short('h')
                .long("height")
                .help("Image wheight"),
        )
        .get_matches();

    init_log("info");

    let (sender, receiver) = sync_channel::<(usize, Vec<[u8; 4]>)>(1 << 16);
    let (command_sender, command_receiver) = sync_channel::<PainterCommand>(256);

    let mut queue = PixelQueue {sender};
    let mut controller = RenderPainterController {command_receiver};

    let mut width: usize = 800;
    let mut height: usize = 600;
    let mut samples: usize = 122;
    let mut scene = ".";

    if let Some(w) = matches.get_one::<String>("width") {
        width = w.parse::<usize>().unwrap();
    }

    if let Some(h) = matches.get_one::<String>("height") {
        height = h.parse::<usize>().unwrap();
    }

    if let Some(s) = matches.get_one::<String>("samples") {
        samples = s.parse::<usize>().unwrap();
    }

    if let Some(s) = matches.get_one::<String>("scene") {
        scene = s;
    }

    spawn(move || boot_sdl(width, height, receiver, command_sender));

    parse_and_render(width, height, samples, scene, &mut queue, &mut controller);

    Ok(())
}
