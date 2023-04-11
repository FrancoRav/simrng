use serde::{Deserialize, Serialize};
use special_fun::cephes_double;
use std::{f64::consts::PI, sync::Arc};

#[derive(Deserialize)]
pub struct HistogramInput {
    pub intervals: usize,
}

#[derive(Serialize)]
pub struct HistogramData {
    pub x: Vec<f64>,
    pub y: Vec<u64>,
    pub lower: f64,
    pub upper: f64,
    pub size: f64,
}

#[derive(Serialize)]
pub struct TestResult {
    pub calculated: f64,
    pub expected: f64,
}

pub trait Distribution {
    fn get_expected(&self, intervals: usize, lower: f64, upper: f64) -> Vec<f64>;
    fn get_degrees(&self, intervals: usize) -> u64;
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
                if interval >= self.lower && (interval + size <= self.upper) {size}
                else if interval + size < self.lower {0f64}
                else if interval >= self.upper {0f64}
                else if interval < self.lower {size-(self.lower-interval)}
                else {self.upper-interval}
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

fn factorial(n: f64) -> f64 {
    let prod: u64 = (0..n as u64).product();
    prod as f64
}

pub fn generate_histogram(input: HistogramInput, nums: &Vec<f64>) -> HistogramData {
    let lower = nums
        .iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap()
        .floor();
    let upper = nums
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap()
        .ceil();
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
        lower,
        upper,
        size,
    }
}

pub fn chi_squared_test(
    input: HistogramInput,
    nums: &Vec<f64>,
    dist: Arc<Box<dyn Distribution + Send + Sync>>,
) -> TestResult {
    let lower = nums
        .iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap()
        .floor();
    let upper = nums
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap()
        .ceil();
    let intervals = input.intervals;
    let size = (upper - lower) / intervals as f64;

    let mut data_list: Vec<u64> = vec![0; intervals];
    for num in nums.iter() {
        let ind = ((num - lower) / size) as usize;
        let ind = ind.min(intervals - 1);
        data_list[ind] += 1;
    }

    let exp_list: Vec<f64> = dist.get_expected(intervals, lower, upper);

    let len = nums.len() as f64;
    let mut calculated = 0f64;
    for (obs, exp) in data_list.iter().zip(exp_list) {
        if exp == 0f64 {continue}
        calculated += (*obs as f64 - exp * len).powi(2) / (exp * len);
    }

    let expected = chi_squared_critical_value(dist.get_degrees(intervals) as f64, 0.05);

    let res = TestResult {
        calculated,
        expected,
    };
    res
}

fn gamma_incomplete_upper(a: f64, x: f64) -> f64 {
    let epsilon = 1e-8;
    let mut s = 0.0;
    let mut term: f64 = 1.0;
    let mut k = 0;
    while term.abs() > epsilon {
        term = (x.powf(a + k as f64) * (-x).exp()) / cephes_double::gamma(a + k as f64 + 1.0);
        s += term;
        k += 1;
    }
    s
}

fn gamma_inverse(z: f64, a: f64) -> f64 {
    let epsilon = 1e-8;
    let mut x = a + 1.0;
    let mut prev_x = 0.0;
    while (x - prev_x).abs() > epsilon {
        prev_x = x;
        let f = gamma_incomplete_upper(a, x) - z;
        let df = x.powf(a - 1.0) * (-x).exp();
        x -= f / df;
    }
    x
}

pub fn chi_squared_critical_value(df: f64, alpha: f64) -> f64 {
    let z = 1.0 - alpha;
    let a = df / 2.0;
    let x = 2.0 * gamma_inverse(z, a);
    x
}
