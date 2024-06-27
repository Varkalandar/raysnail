extern crate sdl2;

#[allow(dead_code)]
mod common;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::video::Window;
use sdl2::render::WindowCanvas;
use sdl2::rect::Point;

use std::time::Duration;

use remda::painter::PainterTarget;
use remda::painter::PainterCommand;

use rayon::spawn;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;


pub struct Renderer { 
    canvas: WindowCanvas 
}


impl Renderer {

    pub fn new(window: Window ) -> Result<Renderer, String> {
        let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        canvas.set_draw_color(Color::RGB(128, 128, 128));
        canvas.clear();
        Ok(Renderer { canvas })
    }

    pub fn setpix(&mut self, x: i32, y: i32, color:[u8; 4]) {
        self.canvas.set_draw_color(Color::RGB(color[0], color[1], color[2]));
        let _ = self.canvas.draw_point(Point::new(x, y));
    }

    pub fn present(&mut self) {
        self.canvas.present();
    }
}


struct PixelQueue {
    sender: Sender<[u8; 4]>,
    command_receiver: Receiver<PainterCommand>,
}


impl PainterTarget for PixelQueue {
    fn register_pixels(&mut self, pixels: &Vec<(u8, u8, u8)>) -> PainterCommand {
        println!("Got {} pixels", pixels.len());

        for pixel in pixels {
            let pix = [pixel.0, pixel.1, pixel.2, 255];
            let status = self.sender.send(pix);
        }

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

    let (sender, receiver) = channel::<[u8; 4]>();
    let (command_sender, command_receiver) = channel::<PainterCommand>();

    let mut queue = PixelQueue {sender, command_receiver};

    spawn(|| boot_sdl(receiver, command_sender));

    render(&mut queue);

    Ok(())
}


fn boot_sdl(receiver: Receiver<[u8; 4]>, command_sender: Sender<PainterCommand>) {
    common::init_log("info");

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("rust-sdl2 demo: Video", 1067, 600)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string()).unwrap();

    // let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    let mut renderer = Renderer::new(window).unwrap();
    let mut x = 0;
    let mut y = 0;

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

        let data = receiver.recv().unwrap();
        renderer.setpix(x, y, data);
        x += 1;

        if x >= 1067 {
            x = 0;
            y += 1;
            renderer.present();
        }
        // ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
    }

    let status = command_sender.send(PainterCommand::Quit);

    println!("Sending Quit to render engine. Status={:?}", status);
}


fn render(target: &mut dyn PainterTarget) {

    // Change `7` to another number to generate different scene
    // Or use `None` to use random seed
    let (camera, world) = common::ray_tracing_in_one_weekend::final_scene(Some(7));

    camera
        .take_photo(world)
        .height(600)
        // .samples(128)
        .samples(128)
        .shot(Some("rtow_13_1.ppm"), target)
        .unwrap();
}
