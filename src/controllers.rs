use axum::extract::State;
use axum::{extract, Json};
use simrng::dist::{
    exponential, normal_box_muller, normal_convolution, poisson, uniform, ExponentialData,
    NormalData, UniformData,
};
use simrng::rng::LinearCongruentialGenerator;
use simrng::stats::{
    chi_squared_test, generate_histogram, Distribution, Exponential, HistogramData, HistogramInput,
    Normal, Poisson, TestResult, Uniform,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Generated {
    pub data: Vec<f64>,
    pub dist: Arc<Box<dyn Distribution>>,
}

unsafe impl Send for Generated {}

impl Generated {
    pub fn new(data: Vec<f64>, dist: Box<dyn Distribution>) -> Self {
        let dist = Arc::new(dist);
        Self { data, dist }
    }
}

pub async fn get_uniform(
    State(arc): State<Arc<Mutex<Generated>>>,
    data: extract::Json<UniformData>,
) {
    let seed = data.seed;
    let number = data.number;
    let lower = data.lower;
    let upper = data.upper;
    let mut rng = LinearCongruentialGenerator::with_seed(seed);
    let mut res = Vec::with_capacity(number as usize);
    for _ in 0..number {
        res.push(uniform(&mut rng, lower, upper));
    }
    let mut arc = arc.lock().await;
    *arc = Generated::new(res.clone(), Box::new(Uniform { lower, upper }));
}

pub async fn get_normal_bm(
    State(arc): State<Arc<Mutex<Generated>>>,
    data: extract::Json<NormalData>,
) {
    let seed = data.seed;
    let number = data.number;
    let mean = data.mean;
    let sd = data.sd;
    let mut rng = LinearCongruentialGenerator::with_seed(seed);
    let mut res = Vec::with_capacity(number as usize);
    for _ in 0..number / 2 {
        let rnds = normal_box_muller(&mut rng, mean, sd);
        res.push(rnds.0);
        res.push(rnds.1);
    }
    let mut arc = arc.lock().await;
    *arc = Generated::new(res.clone(), Box::new(Normal { mean, sd }));
}

pub async fn get_normal_conv(
    State(arc): State<Arc<Mutex<Generated>>>,
    data: extract::Json<NormalData>,
) {
    let seed = data.seed;
    let number = data.number;
    let mean = data.mean;
    let sd = data.sd;
    let mut rng = LinearCongruentialGenerator::with_seed(seed);
    let mut res = Vec::with_capacity(number as usize);
    for _ in 0..number {
        res.push(normal_convolution(&mut rng, mean, sd));
    }
    let mut arc = arc.lock().await;
    *arc = Generated::new(res.clone(), Box::new(Normal { mean, sd }));
}

pub async fn get_exponential(
    State(arc): State<Arc<Mutex<Generated>>>,
    data: extract::Json<ExponentialData>,
) {
    let seed = data.seed;
    let number = data.number;
    let lambda = data.lambda;
    let mut rng = LinearCongruentialGenerator::with_seed(seed);
    let mut res = Vec::with_capacity(number as usize);
    for _ in 0..number {
        res.push(exponential(&mut rng, lambda));
    }
    let mut arc = arc.lock().await;
    *arc = Generated::new(res.clone(), Box::new(Exponential { lambda }));
}

pub async fn get_poisson(
    State(arc): State<Arc<Mutex<Generated>>>,
    data: extract::Json<ExponentialData>,
) {
    let seed = data.seed;
    let number = data.number;
    let lambda = data.lambda;
    let mut rng = LinearCongruentialGenerator::with_seed(seed);
    let mut res = Vec::with_capacity(number as usize);
    for _ in 0..number {
        res.push(poisson(&mut rng, lambda));
    }
    let mut arc = arc.lock().await;
    *arc = Generated::new(res.clone(), Box::new(Poisson { lambda }));
    arc.data = res.clone();
}

pub async fn get_histogram(
    State(arc): State<Arc<Mutex<Generated>>>,
    data: extract::Json<HistogramInput>,
) -> Json<HistogramData> {
    let data = data.0;
    let arc = arc.lock().await;
    Json(generate_histogram(data, &arc.data))
}

pub async fn get_chisquared(
    State(arc): State<Arc<Mutex<Generated>>>,
    data: extract::Json<HistogramInput>,
) -> Json<TestResult> {
    let data = data.0;
    let arc = arc.lock().await;
    let dist = arc.dist.clone();
    let res = chi_squared_test(data, &arc.data, dist);
    Json(res)
}
