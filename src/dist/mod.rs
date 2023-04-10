use crate::rng::Random;
use serde::Deserialize;
use std::f64::consts::{E, PI};

#[derive(Deserialize)]
pub struct UniformData {
    pub seed: u64,
    pub number: u64,
    pub lower: f64,
    pub upper: f64,
}

#[derive(Deserialize)]
pub struct NormalData {
    pub seed: u64,
    pub number: u64,
    pub mean: f64,
    pub sd: f64,
}

#[derive(Deserialize)]
pub struct ExponentialData {
    pub seed: u64,
    pub number: u64,
    pub lambda: f64,
}

pub fn uniform(rand: &mut impl Random, lower: f64, upper: f64) -> f64 {
    lower + rand.next() * (upper - lower)
}

pub fn normal_box_muller(rand: &mut impl Random, mean: f64, sd: f64) -> (f64, f64) {
    let rnd1 = rand.next();
    let rnd2 = rand.next();
    let z1 = (-2f64 * (1f64 - rnd1).ln()).sqrt() * (2f64 * PI * rnd2).cos();
    let z2 = (-2f64 * (1f64 - rnd1).ln()).sqrt() * (2f64 * PI * rnd2).sin();
    let n1 = z1 * sd + mean;
    let n2 = z2 * sd + mean;
    (n1, n2)
}

pub fn normal_convolution(rand: &mut impl Random, mean: f64, sd: f64) -> f64 {
    let mut sum = 0.0;
    for _ in 0..12 {
        sum += rand.next();
    }
    sum -= 6.0;
    mean + sd * sum
}

pub fn exponential(rand: &mut impl Random, lambda: f64) -> f64 {
    -1f64 / lambda * f64::ln(1f64 - rand.next())
}

pub fn poisson(rand: &mut impl Random, lambda: f64) -> f64 {
    let mut p: f64 = 1f64;
    let mut x: i64 = -1;
    let a = E.powf(-lambda);
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
