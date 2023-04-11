use axum::extract::State;
use axum::{extract, Json};
use serde::Deserialize;
use simrng::dist::{Exponential, Normal, Poisson, Uniform, Distribution};
use simrng::rng::LinearCongruentialGenerator;
use simrng::stats::{
    chi_squared_test, generate_histogram, HistogramData, HistogramInput, TestResult,
};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Deserialize)]
pub enum DistributionType {
    Normal,
    Uniform,
    Exponential,
    Poisson,
}

#[derive(Deserialize)]
pub struct GenerationParameters {
    pub seed: u64,
    pub number: u64,
    pub distribution: DistributionType,
    pub data: serde_json::Value,
}

pub struct Generated {
    pub data: Vec<f64>,
    pub dist: Arc<Box<dyn Distribution + Send + Sync>>,
}

impl Generated {
    pub fn new(data: Vec<f64>, dist: Box<dyn Distribution + Send + Sync>) -> Self {
        let dist = Arc::new(dist);
        Self { data, dist }
    }
}

pub async fn get_unified(
    State(arc): State<Arc<Mutex<Generated>>>,
    data: extract::Json<GenerationParameters>,
    ) {
    let mut arc = arc.lock().await;
    arc.data = vec![];
    let mut distribution: Box<dyn Distribution + Send + Sync>;
    match data.distribution {
        DistributionType::Normal => {
            distribution = Box::new(serde_json::from_value::<Normal>(data.data.clone()).unwrap());
        }
        DistributionType::Uniform => {
            distribution = Box::new(serde_json::from_value::<Uniform>(data.data.clone()).unwrap());
        }
        DistributionType::Exponential => {
            distribution = Box::new(serde_json::from_value::<Exponential>(data.data.clone()).unwrap());
        }
        DistributionType::Poisson => {
            distribution = Box::new(serde_json::from_value::<Poisson>(data.data.clone()).unwrap());
        }
    }
    let mut rng = LinearCongruentialGenerator::with_seed(data.seed);
    let mut res = Vec::with_capacity(data.number as usize);
    for _ in 0..data.number {
        res.push(distribution.next(&mut rng));
    }
    *arc = Generated::new(res, distribution)
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
