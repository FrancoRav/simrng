use std::f64::consts::PI;

use crate::rng::Random;

pub fn normal_box_muller(rand: &mut impl Random, sd: f64, mean: f64) -> (f64, f64) {
    let rnd1 = rand.next();
    let rnd2 = rand.next();
    let z1 = (-2f64*(1f64-rnd1).ln()).sqrt() * (2f64*PI*rnd2).cos();
    let z2 = (-2f64*(1f64-rnd1).ln()).sqrt() * (2f64*PI*rnd2).sin();
    let n1 = z1*sd + mean;
    let n2 = z2*sd + mean;
    (n1, n2)
}

pub fn normal_convolution(rand: &mut impl Random, sd: f64, mean: f64) -> f64 {
    let mut sum = 0.0;
    for _ in 0..12 {
        sum += rand.next();
    }
    sum -= 6.0;
    mean + sd * sum
}
