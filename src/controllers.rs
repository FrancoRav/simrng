use axum::extract::State;
use axum::{extract, Json};
use simrng::dist::{
    exponential, normal_box_muller, normal_convolution, poisson, uniform, ExponentialData,
    NormalData, UniformData,
};
use simrng::rng::LinearCongruentialGenerator;
use simrng::stats::{generate_histogram, HistogramData, HistogramInput, Distribution};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Generated<D: Distribution> {
    pub data: Vec<f64>,
    pub dist: D,
}

impl<D: Distribution> Generated<D> {
    fn new(data: Vec<f64>, dist: D) -> Self {
        Generated { data, dist }
    }
}

pub async fn get_uniform<D: Distribution>(
    State(arc): State<Arc<Mutex<Generated<D>>>>,
    data: extract::Json<UniformData>,
) -> Json<Vec<f64>> {
    let seed = data.seed;
    let number = data.number;
    let lower_limit = data.lower;
    let upper_limit = data.upper;
    let mut rng = LinearCongruentialGenerator::with_seed(seed);
    let mut res = Vec::new();
    for _ in 0..number {
        res.push(uniform(&mut rng, lower_limit, upper_limit));
    }
    let mut arc = arc.lock().await;
    arc.data = res.clone();
    Json(res)
}

pub async fn get_normal_bm<D: Distribution>(
    State(arc): State<Arc<Mutex<Generated<D>>>>,
    data: extract::Json<NormalData>,
) -> Json<Vec<f64>> {
    let seed = data.seed;
    let number = data.number;
    let mean = data.mean;
    let sd = data.sd;
    let mut rng = LinearCongruentialGenerator::with_seed(seed);
    let mut res = Vec::new();
    for _ in 0..number / 2 {
        let rnds = normal_box_muller(&mut rng, mean, sd);
        res.push(rnds.0);
        res.push(rnds.1);
    }
    let mut arc = arc.lock().await;
    arc.data = res.clone();
    Json(res)
}

pub async fn get_normal_conv<D: Distribution>(
    State(arc): State<Arc<Mutex<Generated<D>>>>,
    data: extract::Json<NormalData>,
) -> Json<Vec<f64>> {
    let seed = data.seed;
    let number = data.number;
    let mean = data.mean;
    let sd = data.sd;
    let mut rng = LinearCongruentialGenerator::with_seed(seed);
    let mut res = Vec::new();
    for _ in 0..number {
        res.push(normal_convolution(&mut rng, mean, sd));
    }
    let mut arc = arc.lock().await;
    arc.data = res.clone();
    Json(res)
}

pub async fn get_exponential<D: Distribution>(
    State(arc): State<Arc<Mutex<Generated<D>>>>,
    data: extract::Json<ExponentialData>,
) -> Json<Vec<f64>> {
    let seed = data.seed;
    let number = data.number;
    let lambda = data.lambda;
    let mut rng = LinearCongruentialGenerator::with_seed(seed);
    let mut res = Vec::new();
    for _ in 0..number {
        res.push(exponential(&mut rng, lambda));
    }
    let mut arc = arc.lock().await;
    arc.data = res.clone();
    Json(res)
}

pub async fn get_poisson<D: Distribution>(
    State(arc): State<Arc<Mutex<Generated<D>>>>,
    data: extract::Json<ExponentialData>,
) -> Json<Vec<f64>> {
    let seed = data.seed;
    let number = data.number;
    let lambda = data.lambda;
    let mut rng = LinearCongruentialGenerator::with_seed(seed);
    let mut res = Vec::new();
    for _ in 0..number {
        res.push(poisson(&mut rng, lambda));
    }
    let mut arc = arc.lock().await;
    arc.data = res.clone();
    Json(res)
}

pub async fn get_histogram<D: Distribution>(
    State(arc): State<Arc<Mutex<Generated<D>>>>,
    data: extract::Json<HistogramInput>,
) -> Json<HistogramData> {
    let data = data.0;
    let arc = arc.lock().await;
    Json(generate_histogram(data, &arc.data))
}
