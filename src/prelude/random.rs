use {
    rand::{
        distributions::uniform::SampleUniform, rngs::StdRng, seq::SliceRandom, thread_rng, Rng,
        RngCore, SeedableRng,
    },
    std::ops::Range,
};

use rand_xorshift::XorShiftRng;


fn gen<R: Rng>(mut rng: R) -> f64 {
    rng.gen()
}


#[must_use]
fn normal<R: Rng>(mut rng: R) -> f64 {
    rng.gen_range(0.0..=1.0)
}

#[must_use]
fn range<R: Rng, T: SampleUniform + PartialOrd>(mut rng: R, r: Range<T>) -> T {
    rng.gen_range(r)
}

fn choose<T, R: Rng, S: AsRef<[T]>>(mut rng: R, values: &S) -> &T {
    let slice = values.as_ref();
    assert!(!slice.is_empty());
    let index = rng.gen_range(0..slice.len());
    &slice[index]
}

fn shuffle<T, R: Rng, S: AsMut<[T]>>(mut rng: R, values: &mut S) {
    let slice = values.as_mut();
    slice.shuffle(&mut rng);
}

#[derive(Debug)]
pub struct Random();

impl Random {


    // Returns a random number in range [0, 1)
    pub fn gen() -> f64 {
        gen(thread_rng())
    }

    // Returns a random number in range [0, 1]
    #[must_use]
    pub fn normal() -> f64 {
        normal(thread_rng())
    }

    #[must_use]
    pub fn range<T: SampleUniform + PartialOrd>(r: Range<T>) -> T {
        range(thread_rng(), r)
    }

    pub fn choose<T, S: AsRef<[T]>>(values: &S) -> &T {
        choose(thread_rng(), values)
    }

    pub fn shuffle<T, S: AsMut<[T]>>(values: &mut S) {
        shuffle(thread_rng(), values)
    }
}

#[derive(Debug)]
pub struct SeedRandom(StdRng);

impl Default for SeedRandom {
    fn default() -> Self {
        Self::random()
    }
}

impl SeedRandom {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self(StdRng::seed_from_u64(seed))
    }

    #[must_use]
    pub fn random() -> Self {
        Self::new(rand::thread_rng().next_u64())
    }

    pub fn normal(&mut self) -> f64 {
        normal(&mut self.0)
    }

    pub fn range<T: SampleUniform + PartialOrd>(&mut self, r: Range<T>) -> T {
        range(&mut self.0, r)
    }

    pub fn choose<'i, 's, T, S: AsRef<[T]>>(&'i mut self, values: &'s S) -> &'s T {
        choose(&mut self.0, values)
    }

    pub fn shuffle<T, S: AsMut<[T]>>(&mut self, values: &mut S) {
        shuffle(&mut self.0, values)
    }
}


#[derive(Debug)]
pub struct FastRng {
    rng: XorShiftRng,
}


impl FastRng {

    pub fn new() -> FastRng {
        FastRng {
            // rng: XorShiftRng::from_rng(thread_rng()).unwrap()
            rng: XorShiftRng::seed_from_u64(thread_rng().next_u64())
        }
    }


    #[inline]
    pub fn gen(&mut self) -> f64 {
        self.rng.next_u64() as f64 / u64::MAX as f64
    }

    
    /**
     * Range exclusive end
     */ 
    #[inline]
    pub fn range(&mut self, start: f64, end: f64) -> f64 {
        start + self.gen() * (end - start)
    }

    pub fn irange(&mut self, start:usize, end: usize) -> usize {
        start + (self.rng.next_u32() as usize % (end - start))
    }

    pub fn shuffle<T, S: AsMut<[T]>>(&mut self, values: &mut S) {
        let slice = values.as_mut();
        slice.shuffle(&mut self.rng);
    }
}