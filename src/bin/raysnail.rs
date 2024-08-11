extern crate sdl2;

use log::info;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color as SDLColor;
use sdl2::video::Window;
use sdl2::render::WindowCanvas;
use sdl2::render::TextureAccess;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Texture;
use sdl2::rect::Rect;

use image::Rgb;
use image::RgbImage;
use image::ImageFormat;

use clap::{Arg, Command};
use clap::crate_version;

use std::thread;

use std::sync::Arc;
use std::sync::mpsc::sync_channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;

use raysnail::prelude::Ray;
use raysnail::prelude::Color;
use raysnail::prelude::clamp;
use raysnail::material::DiffuseLight;

use raysnail::hittable::Sphere;
use raysnail::hittable::collection::HittableList;
use raysnail::hittable::collection::World;

use raysnail::camera::CameraBuilder;

use raysnail::painter::PainterTarget;
use raysnail::painter::PainterCommand;
use raysnail::painter::PainterController;
use raysnail::painter::PixelController;

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
    sender: SyncSender<(usize, Vec<[f32; 4]>)>,
}

impl PainterTarget for PixelQueue {
    fn register_pixels(&self, y: usize, pixels: &Vec<[f32; 4]>) {
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


struct RedoController {
    redo_map: Vec<u8>,
    width: usize,
}


impl PixelController for RedoController {
    fn calculate_pixel(&self, x: usize, y: usize) -> bool {
        let result = self.redo_map[y * self.width + x];
        // info!("Redo pixel {}, {} -> {}", x, y, result);
        result > 0
    }
}


fn color_diff(a: &[f32; 4], b: &[f32; 4]) -> f32 {
    let rd = a[0] - b[0];
    let gd = a[1] - b[1];
    let bd = a[2] - b[2];

    rd * rd + gd * gd + bd * bd
}

fn get_pixel(x: i32, y: i32, def: [f32; 4],
             pixels: &Vec<[f32; 4]>, width: i32, height: i32) -> [f32; 4] {

    if x >= 0 && y >= 0 && x < width && y < height {
        pixels[(y * width + x) as usize]
    }
    else {
        def
    }
}

fn calc_noise(x: usize, y: usize, 
              pixels: &Vec<[f32; 4]>, width: usize, height: usize) -> f32 {

    let def = pixels[y * width + x];
    let mut diff = 0.0;
    
    let x = y as i32;
    let y = y as i32;

    for yy in y-2 .. y+3 {
        for xx in x-2 .. x+3 {
            diff += color_diff(&def, &get_pixel(xx, yy, def, pixels, width as i32, height as i32));
        }
    }

    diff
}


fn combine_pixel(old: &[f32; 4], new: &[f32; 4], p: f32) -> [f32; 4] {
    if new[0] == 0.0 && new[1] == 0.0 && new[2] == 0.0 && new[3] == 0.0 {
        // no new data for this pixel, keep old
        *old
    }
    else {
        let d = p + 1.0;

        let r = old[0] * p + new[0];
        let g = old[1] * p + new[1];
        let b = old[2] * p + new[2];
        let a = old[3] * p + new[3];
    
        // if p > 0.0 { info!("old {} new {} combined {}", old[0], new[0], r/d);}

        [r/d, g/d, b/d, a/d]
    }
}


fn combine_pixels(old_pixels: &Vec<[f32; 4]>, pixels:&Vec<[f32; 4]>, p: f32) -> Vec<[f32; 4]> {

    let mut result = Vec::with_capacity(pixels.len());

    for i in 0 .. pixels.len() {
        let old = &old_pixels[i];
        let new = &pixels[i];

        result.push(combine_pixel(old, new, p));
    }

    result
}


fn boot_sdl(width: usize, height: usize, receiver: Receiver<(usize, Vec<[f32; 4]>)>, command_sender: SyncSender<PainterCommand>) {
    
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

    // println!("Color mod={:?}", line.color_mod());
    // fprintln!("query={:?}", line.query());

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut pass = 0.0;
    let mut all_pixels: Vec<[f32; 4]> = Vec::with_capacity(width * height);

    for _i in 0 .. width * height {
        all_pixels.push([0.0, 0.0, 0.0, 0.0]);
    }


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

            // did a pass fully complete?
            if y == height && data.len() == 0 {
                info!("Pass {} completed.", pass);
                pass += 1.0;
            }
            else {
                // display the received data after combining it with the existing data
                let mut x = 0;

                for pixel in data {
                
                    let c = if pass > 0.0 && pixel[3] > 0.0 {
                        // [1.0, 0.0, 0.0, 1.0]
                        combine_pixel(&all_pixels[y * width + x], &pixel, pass)
                    }    
                    else {    
                        combine_pixel(&all_pixels[y * width + x], &pixel, pass)
                    };

                    pixels.push((clamp(c[0] as f64, 0.0 .. 1.0) * 255.5) as u8);
                    pixels.push((clamp(c[1] as f64, 0.0 .. 1.0) * 255.5) as u8);
                    pixels.push((clamp(c[2] as f64, 0.0 .. 1.0) * 255.5) as u8);
                    
                    all_pixels[y * width + x] = c;
    
                    x += 1;
                }
    
                renderer.flush_line(y, &pixels, &mut line);
    
                renderer.present();
                pixels.clear();
            }
        } 
        else {
            // let error = data_result.err().unwrap();

            // there don't seem to be any temporary errors, so the only thing
            // left to do is to quit the loop

            break;
        }
    }

    let _status = command_sender.send(PainterCommand::Quit);

    println!("Sending Quit to render engine.");
    std::process::exit(1);
}


fn parse_and_render(width: usize, height: usize, samples: usize, passes: usize,
                    filename: &str,
                    target: &mut dyn PainterTarget, 
                    controller: &mut dyn PainterController,
                    output_file: &str) -> bool {


    // in the first pass all pixels must be calculated
    let mut redo_map: Vec<u8> = Vec::with_capacity(width * height);
    let mut old_pixels: Vec<[f32; 4]> = Vec::with_capacity(width * height);
    for y in 0 .. height {
        for x in 0 .. width {
            old_pixels.push([0.0, 0.0, 0.0, 1.0]);
            redo_map.push(1);
        }
    }

    let mut pass = 0.0;

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
                Some(Arc::new(DiffuseLight::new(light.color).multiplier(1.7)))
            );

        lights.add(rs.clone());
        scene_data.hittables.add(rs);
    }

    fn background(ray: &Ray) -> Color {
        let t = (ray.direction.y + 1.0) * 0.5;  // norm to range 0..1
        Color::new(0.3, 0.4, 0.5, 1.0).gradient(&Color::new(0.7, 0.89, 1.0, 1.0), t)
    }

    let redo_controller = RedoController {
        redo_map: redo_map.clone(),
        width,
    };

    let world = World::new(scene_data.hittables, 
                           lights, 
                           background,
                           &(0.0 .. camera.shutter_speed));

    while (pass as usize) < passes {
        let pixels = 
            camera
                .take_photo()
                .samples(samples)
                .depth(8)
                .shot_to_target(Some("sample_scene.ppm"), 
                                &world, target, controller, &redo_controller);

        let pixels = combine_pixels(&old_pixels, &pixels, pass);
        pass += 1.0;

        info!("Render resulted in {} pixels", pixels.len());

        let mut min = 3.0;
        let mut max = 1.0;

        for y in 0 .. height {
            for x in 0 .. width {
                let noise = calc_noise(x, y, &pixels, width, height);

                if noise < min {min = noise;}
                if noise > max {max = noise;}
            }
        }

        let t = 0.01;
        info!("noise min={} max={} t={} pass={}", min, max, t, pass);

        redo_map.clear();

        let mut count = 0; 
        for y in 0 .. height {
            for x in 0 .. width {
                let noise = calc_noise(x, y, &pixels, width, height);

                if noise >= t {
                    redo_map.push(1);
                    count += 1;
                }
                else {
                    redo_map.push(0);
                }
            }
        }
        info!("oversampling {} pixels", count);

        old_pixels = pixels;
    }

    // save the image as png

    let mut img = RgbImage::new(width as u32, height as u32);

    for y in 0..height {
        for x in 0..width {
            let c = old_pixels[y * width + x];

            let r = (clamp(c[0] as f64, 0.0 .. 1.0) * 255.5) as u8;
            let g = (clamp(c[1] as f64, 0.0 .. 1.0) * 255.5) as u8;
            let b = (clamp(c[2] as f64, 0.0 .. 1.0) * 255.5) as u8;

            img.put_pixel(x as u32, y as u32, Rgb([r, g, b]));
        }
    }    

    img.save_with_format(output_file, ImageFormat::Png);

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
            Arg::new("passes")
                .short('p')
                .long("passes")
                .help("No. of passes to oversample the image. Usually in range 1 .. 10"),
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
        .arg(
            Arg::new("out")
                .short('o')
                .long("outfile")
                .help("Image output file"),
        )
        .get_matches();

    init_log("info");

    let (sender, receiver) = sync_channel::<(usize, Vec<[f32; 4]>)>(1 << 16);
    let (command_sender, command_receiver) = sync_channel::<PainterCommand>(256);

    let mut queue = PixelQueue {sender};
    let mut controller = RenderPainterController {command_receiver};

    let mut width: usize = 800;
    let mut height: usize = 600;
    let mut samples: usize = 122;
    let mut passes: usize = 1;
    let mut scene = ".";
    let mut output_file = "output.png";

    if let Some(w) = matches.get_one::<String>("width") {
        width = w.parse::<usize>().unwrap();
    }

    if let Some(h) = matches.get_one::<String>("height") {
        height = h.parse::<usize>().unwrap();
    }

    if let Some(s) = matches.get_one::<String>("samples") {
        samples = s.parse::<usize>().unwrap();
    }

    if let Some(s) = matches.get_one::<String>("passes") {
        passes = s.parse::<usize>().unwrap();
    }

    if let Some(s) = matches.get_one::<String>("scene") {
        scene = s;
    }

    if let Some(s) = matches.get_one::<String>("out") {
        output_file = s;
    }

    thread::spawn(move || boot_sdl(width, height, receiver, command_sender));

    parse_and_render(width, height, samples, passes, scene, &mut queue, &mut controller, output_file);

    Ok(())
}
