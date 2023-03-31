use axum::Json;
use simrng::dist::{exponential, normal_box_muller, normal_convolution, poisson, uniform};
use simrng::rng::LinearCongruentialGenerator;

pub async fn get_uniform(data: Json<(u64, f64, f64, u64)>) -> Json<Vec<f64>> {
    let seed = data.0 .0;
    let lower_limit = data.1;
    let upper_limit = data.2;
    let number = data.3;
    let mut rng = LinearCongruentialGenerator::new(seed);
    let mut res = Vec::new();
    for _ in 0..number {
        res.push(uniform(&mut rng, lower_limit, upper_limit));
    }
    Json(res)
}

pub async fn get_normal_bm(data: Json<(u64, f64, f64, u64)>) -> Json<Vec<f64>> {
    println!("{:?}", data);
    let seed = data.0 .0;
    let mean = data.1;
    let sd = data.2;
    let number = data.3;
    let mut rng = LinearCongruentialGenerator::new(seed);
    let mut res = Vec::new();
    for _ in 0..number / 2 {
        let rnds = normal_box_muller(&mut rng, mean, sd);
        res.push(rnds.0);
        res.push(rnds.1);
    }
    Json(res)
}

pub async fn get_normal_conv(data: Json<(u64, f64, f64, u64)>) -> Json<Vec<f64>> {
    let seed = data.0 .0;
    let mean = data.1;
    let sd = data.2;
    let number = data.3;
    let mut rng = LinearCongruentialGenerator::new(seed);
    let mut res = Vec::new();
    for _ in 0..number {
        res.push(normal_convolution(&mut rng, mean, sd));
    }
    Json(res)
}

pub async fn get_exponential(data: Json<(u64, f64, u64)>) -> Json<Vec<f64>> {
    let seed = data.0 .0;
    let lambda = data.1;
    let number = data.2;
    let mut rng = LinearCongruentialGenerator::new(seed);
    let mut res = Vec::new();
    for _ in 0..number {
        res.push(exponential(&mut rng, lambda));
    }
    Json(res)
}

pub async fn get_poisson(data: Json<(u64, f64, u64)>) -> Json<Vec<u64>> {
    let seed = data.0 .0;
    let lambda = data.1;
    let number = data.2;
    let mut rng = LinearCongruentialGenerator::new(seed);
    let mut res = Vec::new();
    for _ in 0..number {
        res.push(poisson(&mut rng, lambda));
    }
    Json(res)
}
