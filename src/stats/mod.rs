use std::f64::consts::PI;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct HistogramInput {
    pub lower: f64,
    pub upper: f64,
    pub intervals: usize,
}

#[derive(Serialize)]
pub struct HistogramData {
    pub x: Vec<f64>,
    pub y: Vec<u64>,
}

pub struct TestResult {
    pub calculated: f64,
    pub expected: f64,
}

pub trait Distribution {
    fn get_expected(&self, intervals: usize, lower: f64, upper: f64) -> Vec<f64>;
}

pub struct Normal {
    pub mean: f64,
    pub sd: f64,
}

impl Distribution for Normal {
    fn get_expected(&self, intervals: usize, lower: f64, upper: f64) -> Vec<f64> {
        let size = (upper - lower) / intervals as f64;
        let sd = self.sd;
        let mean = self.mean;
        let mut interval_list: Vec<f64> = Vec::with_capacity(intervals);
        let mut interval = lower + (size / 2f64);

        for _ in 0..intervals {
            let pt1 = 1f64/(sd*f64::sqrt(2f64*PI));
            let pt2 = (-0.5*((interval-mean)/sd).powi(2)).exp();
            let prob = pt1*pt2*size;
            interval_list.push(prob);
            interval += size;
        }
        interval_list
    }
}

pub struct Uniform {
    pub lower: f64,
    pub upper: f64,
}

impl Distribution for Uniform {
    fn get_expected(&self, intervals: usize, _: f64, _: f64) -> Vec<f64> {
        vec![1f64/intervals as f64; intervals]
    }
}

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
            let prob = (-lambda*interval).exp()*lambda;
            interval_list.push(prob);
            interval += size;
        }
        interval_list
    }
}

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
            let prob = ((-lambda).exp()*lambda.powf(interval))/factorial(interval);
            interval_list.push(prob);
            interval += size;
        }
        interval_list
    }
}

fn factorial(n: f64) -> f64 {
    let prod: u64 = (0..n as u64).product();
    prod as f64
}

pub fn generate_histogram(input: HistogramInput, nums: &Vec<f64>) -> HistogramData {
    let upper = input.upper;
    let lower = input.lower;
    let intervals = input.intervals;
    let size = (upper - lower) / intervals as f64;
    let mut interval_list: Vec<f64> = Vec::with_capacity(intervals);
    let mut interval = lower + (size / 2f64);

    for _ in 0..intervals {
        interval_list.push(interval);
        interval += size;
    }

    let mut data_list: Vec<u64> = vec![0; intervals];
    for num in nums.iter() {
        let ind = ((num - lower) / size) as usize;
        let ind = ind.min(intervals - 1);
        data_list[ind] += 1;
    }

    HistogramData {
        x: interval_list,
        y: data_list,
    }
}
