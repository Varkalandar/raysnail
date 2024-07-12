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
        sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};


#[derive(Debug, PartialEq)]
pub enum PainterCommand {
    None,
    Quit,
}


pub trait PainterTarget : Send + Sync {
    fn register_pixels(&self, _y:usize, _pixels: &Vec<[u8; 4]>) {
    }
}

pub trait PainterController : Send {
    fn receive_command(&self) -> PainterCommand {
        PainterCommand::None
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
                let color = &self.colors[index].i();
                writeln!(
                    &mut file,
                    "{r} {g} {b}",
                    r = color.r,
                    g = color.g,
                    b = color.b
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

        // before stratification
        /*
        if self.samples == 1 {
            let u = (column as f64) / self.width as f64;
            let v = ((self.height - 1 - row) as f64) / self.height as f64;
            (u, v)
        } else {
            let u = (column as f64 + Random::normal()) / self.width as f64;
            let v = ((self.height - 1 - row) as f64 + Random::normal()) / self.height as f64;
            (u, v)
        }
        */

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
        Ok(PainterOutputContext { file, cancel, controller: Some(Box::new(controller)) })
    }


    fn render_pixel<F>(&self, row: usize, column: usize, rng: &mut FastRng, uv_color: &F) -> [u8; 4]
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
                let mut last_color = Vec3::new(0.5, 0.5, 0.5);

                let xo = x + (s_i as f64 + rng.gen()) / self.sqrt_spp as f64;
                let yo = y + (s_j as f64 + rng.gen()) / self.sqrt_spp as f64;
                
                let uv = self.calculate_uv(xo, yo);
                
                // info!("Subpixel {}, {}", xo, yo);

                let mut color = uv_color(uv[0], uv[1], rng);
                let diff = (&color - &last_color).length_squared();

                if diff > 10.0 {
                    let limit = (diff.sqrt() as usize).min(100);
                    let mut counter = 1;
                    
                    println!("Oversampling pixel {} times due to big color diff", limit);
                    while counter < limit {
                        color = color + uv_color(uv[0], uv[1], rng);
                        counter += 1;
                    }
                    
                    color = color * (1.0 / limit as f64);
                }

                color_vec = color_vec + &color;
                // last_color = color;
            }
        }

        let color = color_vec.into_color(self.samples, self.gamma);
        let int_color = color.i();

        [int_color.r, int_color.g, int_color.b, 255]
    }

    fn parallel_render_row<F>(&self, row: usize, uv_color: &F, cancel: &AtomicBool,
                              target: &dyn PainterTarget
    ) -> Vec<[u8; 4]>
    where
        F: Fn(f64, f64, &mut FastRng) -> Vec3 + Send + Sync,
    {
        info!("Processing line: {}", row);

        let mut rng = FastRng::new();

        let pixels = 
            (0..self.width)
            .map(|column| {
                if cancel.load(Ordering::Relaxed) {
                    return [0, 0, 0, 255];
                }
                self.render_pixel(row, column, &mut rng, &uv_color)
            })
            .collect::<Vec<_>>();

        target.register_pixels(row, &pixels);
/*
        if command == PainterCommand::Quit {
            cancel.store(true, Ordering::Relaxed);
        }
*/
        pixels
    }

    /*
    fn seq_render_row<F>(&self, row: usize, uv_color: &F) -> Vec<[u8; 4]>
    where
        F: Fn(f64, f64) -> Vec3 + Send + Sync,
    {
        (0..self.width)
            .map(|column| self.render_pixel(row, column, &uv_color))
            .collect::<Vec<_>>()
    }
    */

    fn parallel_render_row_iter<'c, F>(&'c self, uv_color: F, cancel: &'c AtomicBool,
                                target: &'c dyn PainterTarget,
    ) -> impl IndexedParallelIterator<Item = Vec<[u8; 4]>> + 'c
    where
        F: Fn(f64, f64, &mut FastRng) -> Vec3 + Send + Sync + 'c,
    {
        (0..self.height)
            .into_par_iter()
            .map(move |row| self.parallel_render_row(row, &uv_color, cancel, target))
    }

    /*
    fn seq_render_row_iter<'c, F>(
        &'c self, uv_color: F,
    ) -> impl Iterator<Item = Vec<[u8; 4]>> + 'c
    where
        F: Fn(f64, f64) -> Vec3 + Send + Sync + 'c,
    {
        (0..self.height).map(move |row| self.seq_render_row(row, &uv_color))
    }
    */

    fn real_row_pixels_to_file(
        context: &mut PainterOutputContext<'_>, pixels: Vec<[u8; 4]>,
    ) -> std::io::Result<()> {

        for pixel in &pixels {
            writeln!(context.file, "{} {} {}", pixel[0], pixel[1], pixel[2])?;
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
        context: &mut PainterOutputContext<'_>, pixels: Vec<[u8; 4]>,
    ) -> std::io::Result<()> {
        Self::real_row_pixels_to_file(context, pixels).map_err(|e| {
            context.cancel.store(true, Ordering::Relaxed);
            e
        })
    }

    fn parallel_render_and_output<F>(&self, uv_color: F, path: Option<&Path>, 
                                     target: &mut dyn PainterTarget,
                                     controller: &mut dyn PainterController) -> std::io::Result<()>
    where
        F: Fn(f64, f64, &mut FastRng) -> Vec3 + Send + Sync,
    {
        let cancel = AtomicBool::new(false);
        let finished_row = AtomicUsize::new(0);

        info!("Starting parallel render");

        self.parallel_render_row_iter(uv_color, &cancel, target)
            .seq_for_each_with(
                || self.create_output_context(path, controller, &cancel),
                |context, pixels| Self::row_pixels_to_file(context, pixels),
            )
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
                      target: &mut dyn PainterTarget, controller: &mut dyn PainterController,
                      uv_color: F) -> std::io::Result<()>
    where
        P: AsRef<Path>,
        F: Fn(f64, f64, &mut FastRng) -> Vec3 + Send + Sync,
    {
        let path = match path {
            Some(ref path) => Some(path.as_ref()),
            None => None,
        };

        let pool = self.setup_thread_pool()?;

        info!("Worker thread count: {}", pool.current_num_threads());

        pool.install(|| self.parallel_render_and_output(uv_color, path, target, controller))
    }
}
