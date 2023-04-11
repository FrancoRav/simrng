use crate::rng::Random;
use serde::Deserialize;
use std::f64::consts::{E, PI};

pub trait Distribution {
    fn get_expected(&self, intervals: usize, lower: f64, upper: f64) -> Vec<f64>;
    fn get_degrees(&self, intervals: usize) -> u64;
}

#[derive(Deserialize)]
pub enum Algorithm {
    BoxMuller,
    Convolution,
}

#[derive(Deserialize)]
pub struct Normal {
    pub algorithm: Algorithm,
    pub mean: f64,
    pub sd: f64,
    pub pair: Option<f64>,
}

impl Distribution for Normal {
    fn get_expected(&self, intervals: usize, lower: f64, upper: f64) -> Vec<f64> {
        let size = (upper - lower) / intervals as f64;
        let sd = self.sd;
        let mean = self.mean;
        let mut interval_list: Vec<f64> = Vec::with_capacity(intervals);
        let mut interval = lower + (size / 2f64);

        for _ in 0..intervals {
            let pt1 = 1f64 / (sd * f64::sqrt(2f64 * PI));
            let pt2 = (-0.5 * ((interval - mean) / sd).powi(2)).exp();
            let prob = pt1 * pt2 * size;
            interval_list.push(prob);
            interval += size;
        }
        interval_list
    }
    fn get_degrees(&self, intervals: usize) -> u64 {
        intervals as u64 - 3
    }
}

impl Normal {
    pub fn next(&mut self, rand: &mut dyn Random) -> f64 {
        let ret: f64;
        match self.algorithm {
            Algorithm::BoxMuller => match self.pair {
                Some(x) => {
                    ret = x;
                    self.pair = None;
                }
                None => {
                    let gen = self.get_bm(rand);
                    self.pair = Some(gen.1);
                    ret = gen.0;
                }
            },
            Algorithm::Convolution => {
                ret = self.get_conv(rand);
            }
        }
        ret
    }
}

impl Normal {
    pub fn get_bm(&self, rand: &mut dyn Random) -> (f64, f64) {
        let rnd1 = rand.next();
        let rnd2 = rand.next();
        let z1 = (-2f64 * (1f64 - rnd1).ln()).sqrt() * (2f64 * PI * rnd2).cos();
        let z2 = (-2f64 * (1f64 - rnd1).ln()).sqrt() * (2f64 * PI * rnd2).sin();
        let n1 = z1 * self.sd + self.mean;
        let n2 = z2 * self.sd + self.mean;
        (n1, n2)
    }

    pub fn get_conv(&self, rand: &mut dyn Random) -> f64 {
        let mut sum = 0.0;
        for _ in 0..12 {
            sum += rand.next();
        }
        sum -= 6.0;
        self.mean + self.sd * sum
    }
}

#[derive(Deserialize)]
pub struct Uniform {
    pub lower: f64,
    pub upper: f64,
}

impl Distribution for Uniform {
    fn get_expected(&self, intervals: usize, lower: f64, upper: f64) -> Vec<f64> {
        let size = (upper - lower) / intervals as f64;
        let mut interval_list: Vec<f64> = Vec::with_capacity(intervals);
        let mut interval = lower;
        for _ in 0..intervals {
            let inside_interval = {
                if interval >= self.lower && (interval + size <= self.upper) {
                    size
                } else if interval + size < self.lower {
                    0f64
                } else if interval >= self.upper {
                    0f64
                } else if interval < self.lower {
                    size - (self.lower - interval)
                } else {
                    self.upper - interval
                }
            };
            interval_list.push(1f64 / (self.upper - self.lower) * inside_interval);
            interval += size;
        }
        interval_list
    }

    fn get_degrees(&self, intervals: usize) -> u64 {
        intervals as u64 - 1
    }
}

impl Uniform {
    pub fn next(&self, rand: &mut dyn Random) -> f64 {
        self.lower + rand.next() * (self.upper - self.lower)
    }
}

#[derive(Deserialize)]
pub struct Exponential {
    pub lambda: f64,
}

impl Distribution for Exponential {
    fn get_expected(&self, intervals: usize, lower: f64, upper: f64) -> Vec<f64> {
        let size = (upper - lower) / intervals as f64;
        let lambda = self.lambda;
        let mut interval_list: Vec<f64> = Vec::with_capacity(intervals);
        let mut interval = lower + (size / 2f64);
        for _ in 0..intervals {
            let prob = (-lambda * interval).exp() * lambda;
            interval_list.push(prob);
            interval += size;
        }
        interval_list
    }

    fn get_degrees(&self, intervals: usize) -> u64 {
        intervals as u64 - 2
    }
}

impl Exponential {
    pub fn next(&self, rand: &mut dyn Random) -> f64 {
        -1f64 / self.lambda * f64::ln(1f64 - rand.next())
    }
}

#[derive(Deserialize)]
pub struct Poisson {
    pub lambda: f64,
}

impl Distribution for Poisson {
    fn get_expected(&self, intervals: usize, lower: f64, upper: f64) -> Vec<f64> {
        let size = (upper - lower) / intervals as f64;
        let lambda = self.lambda;
        let mut interval_list: Vec<f64> = Vec::with_capacity(intervals);
        let mut interval = lower;
        for _ in 0..intervals {
            let prob = ((-lambda).exp() * lambda.powf(interval)) / factorial(interval);
            interval_list.push(prob);
            interval += size;
        }
        interval_list
    }

    fn get_degrees(&self, intervals: usize) -> u64 {
        intervals as u64 - 2
    }
}

impl Poisson {
    pub fn next(&self, rand: &mut dyn Random) -> f64 {
        let mut p: f64 = 1f64;
        let mut x: i64 = -1;
        let a = E.powf(-self.lambda);
        loop {
            let u = rand.next();
            p *= u;
            x += 1;
            if p < a {
                break;
            }
        }
        x as f64
    }
}

fn factorial(n: f64) -> f64 {
    let prod: u64 = (0..n as u64).product();
    prod as f64
}
