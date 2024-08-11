use {
    crate::prelude::*,
    log::info,
    std::{
        fs::File,
        io::{BufWriter, Write},
        iter::FromIterator,
        ops::{Index, IndexMut},
        path::Path,
        sync::atomic::{AtomicBool, Ordering},
    },
};

use std::thread;

#[derive(Debug, PartialEq)]
pub enum PainterCommand {
    None,
    Quit,
}


pub trait PainterTarget : Send + Sync {
    fn register_pixels(&self, _y: usize, _pixels: &Vec<[f32; 4]>) {
    }
}

pub trait PainterController : Send {
    fn receive_command(&self) -> PainterCommand {
        PainterCommand::None
    }
}

pub trait PixelController : Send + Sync {
    fn calculate_pixel(&self, _x: usize, _y: usize) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct PassivePainterTarget {
}


impl PainterTarget for PassivePainterTarget {
}


#[derive(Debug)]
pub struct PassivePainterController {
}


impl PainterController for PassivePainterController {
}


#[derive(Debug)]
pub struct PassivePixelController {
}


impl PixelController for PassivePixelController {
}



#[derive(Debug, Clone)]
pub struct Painter {
    pub width: usize,
    pub height: usize,
    samples: usize,
    gamma: bool,
    threads: usize,
    parallel: bool,

    sqrt_spp: usize,         // Square root of number of samples per pixel
}


struct PainterOutputContext<'c> {
    cancel: &'c AtomicBool,
    controller: Option<Box<&'c mut dyn PainterController>>,
}


impl Painter {
    #[must_use]
    pub const fn new(width: usize, height: usize) -> Self {

        Self {
            width,
            height,
            gamma: true,
            samples: 25,
            threads: 0,
            parallel: true,

            sqrt_spp: 5,
        }
    }

    #[must_use]
    pub const fn gamma(mut self, gamma: bool) -> Self {
        self.gamma = gamma;
        self
    }

    #[must_use]
    pub fn samples(mut self, samples_requested: usize) -> Self {

        let sqrt_spp = (samples_requested as f64).sqrt().floor() as usize;

        self.sqrt_spp = sqrt_spp;
        self.samples = sqrt_spp * sqrt_spp;

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

    #[allow(clippy::cast_precision_loss)] // because row and column is small enough in practice
    fn calculate_uv(&self, x: f64, y: f64) -> [f64; 2]  {

        let h = self.height as f64;
        let u = x / self.width as f64;
        let v = (h - 1.0 - y) / h;
        [u, v]
    }


    fn create_output_context<'c>(
        &self, path: Option<&Path>, controller: &'c mut dyn PainterController, cancel: &'c AtomicBool,
    ) -> std::io::Result<PainterOutputContext<'c>> {
        
        Ok(PainterOutputContext { 
            cancel, 
            controller: 
            Some(Box::new(controller)) 
        })
    }


    fn render_pixel<F>(&self, row: usize, column: usize, rng: &mut FastRng, uv_color: &F) -> [f32; 4]
    where
        F: Fn(f64, f64, &mut FastRng) -> Vec3 + Send + Sync,
    {
        // Stratification, randomized subpixels
     
        let x = column as f64;
        let y = row as f64;
     
        let mut color_vec = Vec3::new(0.0, 0.0, 0.0);
        
        // info!("Pixel {}, {}", column, row);

        for s_j in 0 .. self.sqrt_spp {
            for s_i in 0 .. self.sqrt_spp {
                let xo = x + (s_i as f64 + rng.gen()) / self.sqrt_spp as f64;
                let yo = y + (s_j as f64 + rng.gen()) / self.sqrt_spp as f64;
                
                let uv = self.calculate_uv(xo, yo);
                
                let color = uv_color(uv[0], uv[1], rng);

                color_vec = color_vec + &color;
                // last_color = color;
            }
        }

        let color = color_vec.into_color(self.samples, self.gamma);

        [color.r, 
         color.g, 
         color.b, 
         1.0]
    }

    fn render_row<F>(&self, row: usize, uv_color: &F, cancel: &AtomicBool,
                              target: &dyn PainterTarget, 
                              pixel_map: &dyn PixelController,
                     rng: &mut FastRng) -> Vec<[f32; 4]>
    where
        F: Fn(f64, f64, &mut FastRng) -> Vec3 + Send + Sync,
    {
        info!("Processing line: {}", row);

        let pixels = 
            (0..self.width)
            .map(|column| {
                if cancel.load(Ordering::Relaxed) {
                    return [0.0, 0.0, 0.0, 1.0];
                }
                if pixel_map.calculate_pixel(column, row) {
                    return self.render_pixel(row, column, rng, &uv_color)
                }

                // return a fully transparent black pixel for the parts
                // which are not calculated in this pass
                return [0.0, 0.0, 0.0, 0.0];
            })
            .collect::<Vec<_>>();

        target.register_pixels(row, &pixels);

        pixels
    }


    fn append(result: &mut Vec<[f32; 4]>, pixels: &Vec<[f32; 4]>, i: usize, step: usize, width: usize) {

        let lines = pixels.len() / width;

        info!("Thread {} rendered {} lines", i, lines);

        for line in 0 .. lines {
            let src = line * width;
            let dest = line * width * step + i * width;

            if dest < result.len() {
                for x in 0 .. width {
                    result[dest + x] = pixels[src + x]
                }
            }
        }
    }


    fn render_rows<F>(&self, row: usize, step: usize, uv_color: &F, cancel: &AtomicBool,
        target: &dyn PainterTarget, 
        pixel_map: &dyn PixelController) -> Vec<[f32; 4]>
    where
        F: Fn(f64, f64, &mut FastRng) -> Vec3 + Send + Sync,
    {
        let mut rng = FastRng::new();
        let mut pixels = Vec::new();

        for y in (row .. self.height).step_by(step ) {
            let mut row_pixels = self.render_row(y, uv_color, cancel, target, pixel_map, &mut rng);
            pixels.append(&mut row_pixels);
        } 

        pixels
    }

    fn parallel_render_and_output<F>(&self, uv_color: F, path: Option<&Path>, 
                                     target: &dyn PainterTarget,
                                     controller: &mut dyn PainterController,
                                     pixel_map: &dyn PixelController,
                                     threads: usize) -> Vec<[f32; 4]>
    where
        F: Fn(f64, f64, &mut FastRng) -> Vec3 + Send + Sync,

    {
        let cancel = AtomicBool::new(false);

        info!("Starting parallel render");

        let mut result = Vec::with_capacity(self.width * self.height);

        for _i in 0 .. self.width * self.height {
            result.push([0.0, 0.0, 0.0, 0.0]);
        }
        
        thread::scope(|s| {

            let mut handles = Vec::new();

            for i in 0 .. threads {
                let start_row = i;
                let step = threads;
                let uv = &uv_color;
                let stop = &cancel;
                let tar = target;
                let map = pixel_map;

                handles.push(s.spawn(move || self.render_rows(start_row, step, uv, stop, tar, map)));
            }

            for i in 0 .. threads {
                let handle = handles.pop().unwrap();

                let pixels = handle.join();

                Self::append(&mut result, &pixels.unwrap(), threads-i-1, threads, self.width);

                // println!("{:?}", r);            
            }
        });
    
        result
    }

    /// # Errors
    ///
    /// When open or save to file failed
    pub fn draw<P, F>(&self, path: &Option<P>, 
                      target: &mut dyn PainterTarget, 
                      controller: &mut dyn PainterController,
                      pixel_map: &dyn PixelController,
                      uv_color: F) -> Vec<[f32; 4]>
    where
        P: AsRef<Path>,
        F: Fn(f64, f64, &mut FastRng) -> Vec3 + Send + Sync,
    {
        let path = match path {
            Some(ref path) => Some(path.as_ref()),
            None => None,
        };

        let threads = if self.threads == 0 {
                num_cpus::get() + 1
            } else {
                self.threads + 1
            };

        info!("Using {} threads to render the image", threads);

        let result = self.parallel_render_and_output(uv_color, path, target, controller, pixel_map, threads);

        // mark end of rendering pass by sending height and an empty vector
        target.register_pixels(self.height, &Vec::new());


        return result;            
    }
}
