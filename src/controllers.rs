use axum::Json;
use simrng::dist::{exponential, normal_box_muller, poisson, uniform};
use simrng::{dist::normal_convolution, rng::LinearCongruentialGenerator};

pub async fn get_uniform(data: Json<(u64,f64,f64,u64)>) -> Json<Vec<f64>> {
    let seed = data.0.0;
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
