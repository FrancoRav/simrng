use axum::{Json, extract};
use simrng::dist::{exponential, normal_box_muller, normal_convolution, poisson, uniform};
use simrng::rng::LinearCongruentialGenerator;
use serde::{Deserialize};

#[derive(Deserialize)]
pub struct UniformData {
    seed: u64,
    number: u64,
    lower: f64,
    upper: f64,
}

pub async fn get_uniform(data: extract::Json<UniformData>) -> Json<Vec<f64>> {
    let seed = data.seed;
    let number = data.number;
    let lower_limit = data.lower;
    let upper_limit = data.upper;
    let mut rng = LinearCongruentialGenerator::new(seed);
    let mut res = Vec::new();
    for _ in 0..number {
        res.push(uniform(&mut rng, lower_limit, upper_limit));
    }
    Json(res)
}

#[derive(Deserialize, Debug)]
pub struct NormalData {
    seed: u64,
    number: u64,
    mean: f64,
    sd: f64,
}

pub async fn get_normal_bm(data: extract::Json<NormalData>) -> Json<Vec<f64>> {
    let seed = data.seed;
    let number = data.number;
    let mean = data.mean;
    let sd = data.sd;
    let mut rng = LinearCongruentialGenerator::new(seed);
    let mut res = Vec::new();
    for _ in 0..number / 2 {
        let rnds = normal_box_muller(&mut rng, mean, sd);
        res.push(rnds.0);
        res.push(rnds.1);
    }
    Json(res)
}

pub async fn get_normal_conv(data: extract::Json<NormalData>) -> Json<Vec<f64>> {
    let seed = data.seed;
    let number = data.number;
    let mean = data.mean;
    let sd = data.sd;
    let mut rng = LinearCongruentialGenerator::new(seed);
    let mut res = Vec::new();
    for _ in 0..number {
        res.push(normal_convolution(&mut rng, mean, sd));
    }
    Json(res)
}

#[derive(Deserialize)]
pub struct ExponentialData {
    seed: u64,
    number: u64,
    lambda: f64,
}

pub async fn get_exponential(data: extract::Json<ExponentialData>) -> Json<Vec<f64>> {
    let seed = data.seed;
    let number = data.number;
    let lambda = data.lambda;
    let mut rng = LinearCongruentialGenerator::new(seed);
    let mut res = Vec::new();
    for _ in 0..number {
        res.push(exponential(&mut rng, lambda));
    }
    Json(res)
}

pub async fn get_poisson(data: extract::Json<ExponentialData>) -> Json<Vec<u64>> {
    let seed = data.seed;
    let number = data.number;
    let lambda = data.lambda;
    let mut rng = LinearCongruentialGenerator::new(seed);
    let mut res = Vec::new();
    for _ in 0..number {
        res.push(poisson(&mut rng, lambda));
    }
    Json(res)
}
