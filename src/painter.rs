use {
    crate::{internal::rayon_seq_iter::SeqForEach, prelude::*},
    log::info,
    rayon::{prelude::*, ThreadPool, ThreadPoolBuilder},
    std::{
        fs::File,
        io::{BufWriter, Write},
        iter::FromIterator,
        ops::{Index, IndexMut},
        path::Path,
        sync::atomic::{AtomicBool, Ordering},
    },
};


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


#[derive(Debug)]
pub struct PPMImage {
    width: usize,
    height: usize,
    colors: Vec<Color>,
}


impl PPMImage {
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        let colors = vec![Color::default(); width * height];
        Self {
            width,
            height,
            colors,
        }
    }

    /// # Errors
    /// When open or write to file failed
    pub fn save<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        write!(
            &mut file,
            "P3\n{width} {height}\n255\n",
            width = self.width,
            height = self.height
        )?;

        for row in 0..self.height {
            for column in 0..self.width {
                let index = row * self.width + column;
                let color = &self.colors[index];
                writeln!(
                    &mut file,
                    "{r} {g} {b}",
                    r = (clamp(color.r as f64, 0.0..1.0) * 255.0) as u8,
                    g = (clamp(color.g as f64, 0.0..1.0) * 255.0) as u8,
                    b = (clamp(color.b as f64, 0.0..1.0) * 255.0) as u8
                )?;
            }
        }

        Ok(())
    }

    /// # Errors
    ///
    /// When image pixel count is not divisible by new width
    pub fn reshape(&mut self, width: usize) -> Result<(), &'static str> {
        if self.colors.len() % width == 0 {
            self.width = width;
            self.height = self.colors.len() / width;
            Ok(())
        } else {
            Err("Shape invalid")
        }
    }
}

impl FromIterator<Color> for PPMImage {
    fn from_iter<T: IntoIterator<Item = Color>>(iter: T) -> Self {
        Vec::from_iter(iter).into()
    }
}

impl<T> From<T> for PPMImage
where
    T: Into<Vec<Color>>,
{
    fn from(container: T) -> Self {
        let colors = container.into();
        Self {
            height: 1,
            width: colors.len(),
            colors,
        }
    }
}

impl Index<(usize, usize)> for PPMImage {
    type Output = Color;
    fn index(&self, (row, col): (usize, usize)) -> &Self::Output {
        self.index(row * self.width + col)
    }
}

impl Index<usize> for PPMImage {
    type Output = Color;
    fn index(&self, index: usize) -> &Self::Output {
        self.colors.index(index)
    }
}

impl IndexMut<(usize, usize)> for PPMImage {
    fn index_mut(&mut self, (row, col): (usize, usize)) -> &mut Self::Output {
        self.index_mut(row * self.width + col)
    }
}

impl IndexMut<usize> for PPMImage {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.colors.index_mut(index)
    }
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
    file: BufWriter<Box<dyn Write>>,
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


    fn create_output_file(
        &self, path: Option<&Path>,
    ) -> std::io::Result<BufWriter<Box<dyn Write>>> {
        let mut file: BufWriter<Box<dyn Write>> = if let Some(path) = path {
            BufWriter::new(Box::new(File::create(&path)?))
        } else {
            BufWriter::new(Box::new(std::io::sink()))
        };

        write!(
            &mut file,
            "P3\n{width} {height}\n255\n",
            width = self.width,
            height = self.height
        )?;

        Ok(file)
    }

    fn create_output_context<'c>(
        &self, path: Option<&Path>, controller: &'c mut dyn PainterController, cancel: &'c AtomicBool,
    ) -> std::io::Result<PainterOutputContext<'c>> {
        let file = self.create_output_file(path)?;
        Ok(PainterOutputContext { 
            file,
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

    fn parallel_render_row<F>(&self, row: usize, uv_color: &F, cancel: &AtomicBool,
                              target: &dyn PainterTarget, 
                              pixel_map: &dyn PixelController) -> Vec<[f32; 4]>
    where
        F: Fn(f64, f64, &mut FastRng) -> Vec3 + Send + Sync,
    {
        // info!("Processing line: {}", row);

        let mut rng = FastRng::new();

        let pixels = 
            (0..self.width)
            .map(|column| {
                if cancel.load(Ordering::Relaxed) {
                    return [0.0, 0.0, 0.0, 1.0];
                }
                if pixel_map.calculate_pixel(column, row) {
                    return self.render_pixel(row, column, &mut rng, &uv_color)
                }

                // return a fully transparent black pixel for the parts
                // which are not calculated in this pass
                return [0.0, 0.0, 0.0, 0.0];
            })
            .collect::<Vec<_>>();

        target.register_pixels(row, &pixels);

        pixels
    }


    fn parallel_render_row_iter<'a, F>(&'a self, uv_color: F, cancel: &'a AtomicBool,
                                target: &'a dyn PainterTarget, 
                                pixel_map: &'a dyn PixelController,) -> impl IndexedParallelIterator<Item = Vec<[f32; 4]>> + 'a
    where
        F: Fn(f64, f64, &mut FastRng) -> Vec3 + Send + Sync + 'a,
    {
        (0..self.height)
            .into_par_iter()
            .map(move |row| self.parallel_render_row(row, &uv_color, cancel, target, pixel_map))
    }


    fn real_row_pixels_to_file(
        context: &mut PainterOutputContext<'_>, pixels: Vec<[f32; 4]>,
    ) -> std::io::Result<()> {

        for pixel in &pixels {
            writeln!(context.file, "{} {} {}", 
            (clamp(pixel[0] as f64, 0.0 .. 1.0) * 255.5) as u8,
            (clamp(pixel[1] as f64, 0.0 .. 1.0) * 255.5) as u8,
            (clamp(pixel[2] as f64, 0.0 .. 1.0) * 255.5) as u8)?;
        }

        if let Some(controller) = &mut context.controller {
            let command = controller.receive_command();

            if command == PainterCommand::Quit {
                context.cancel.store(true, Ordering::Relaxed);
            }
        }

        context.file.flush()
    }


    fn row_pixels_to_file(
        context: &mut PainterOutputContext<'_>, pixels: Vec<[f32; 4]>,
    ) -> std::io::Result<()> {
        Self::real_row_pixels_to_file(context, pixels).map_err(|e| {
            context.cancel.store(true, Ordering::Relaxed);
            e
        })
    }


    fn parallel_render_and_output<F>(&self, uv_color: F, path: Option<&Path>, 
                                     target: &mut dyn PainterTarget,
                                     controller: &mut dyn PainterController,
                                     pixel_map: &dyn PixelController) -> Vec<[f32; 4]>
    where
        F: Fn(f64, f64, &mut FastRng) -> Vec3 + Send + Sync,

    {
        let cancel = AtomicBool::new(false);

        info!("Starting parallel render");

        let mut result = Vec::with_capacity(1 << 15);

        let ok =
            self.parallel_render_row_iter(uv_color, &cancel, target, pixel_map)
                .seq_for_each_with(
                    || self.create_output_context(path, controller, &cancel),
                    |context, pixels| {
                        for pixel in &pixels {
                            result.push(*pixel);
                        }
                        Self::row_pixels_to_file(context, pixels)
                    },
                );

        result
    }

    fn setup_thread_pool(&self) -> std::io::Result<ThreadPool> {
        let threads = if self.threads == 0 {
            num_cpus::get() + 1
        } else {
            self.threads + 1
        };
        ThreadPoolBuilder::default()
            .num_threads(threads)
            .build()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
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

        let pool = self.setup_thread_pool();
        let mut result = Vec::new();

        if pool.is_ok() {
            let pool = pool.unwrap();
            info!("Worker thread count: {}", pool.current_num_threads());

            result = 
                pool.install(|| self.parallel_render_and_output(uv_color, path, target, controller, pixel_map));
        }

        target.register_pixels(self.height, &Vec::new());


        return result;            
    }
}
