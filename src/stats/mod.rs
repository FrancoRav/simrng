use serde::{Deserialize, Serialize};
use special_fun::cephes_double;
use std::sync::Arc;

use crate::dist::Distribution;

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

#[derive(Serialize)]
pub struct StatisticsResponse {
    pub histogram: HistogramData,
    pub test: TestResult,
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

pub fn full_statistics(
    input: HistogramInput,
    nums: &Vec<f64>,
    dist: Arc<Box<dyn Distribution + Send + Sync>>,
) -> StatisticsResponse {
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

    let exp_list: Vec<f64> = dist.get_expected(intervals, lower, upper);

    let len = nums.len() as f64;
    let mut calculated = 0f64;
    for (obs, exp) in data_list.iter().zip(exp_list) {
        if exp == 0f64 {
            continue;
        }
        calculated += (*obs as f64 - exp * len).powi(2) / (exp * len);
    }

    let expected = chi_squared_critical_value(dist.get_degrees(intervals) as f64, 0.05);

    let test = TestResult {
        calculated,
        expected,
    };
    let hist = HistogramData {
        x: interval_list,
        y: data_list,
        lower,
        upper,
        size,
    };
    let res = StatisticsResponse {
        histogram: hist,
        test,
    };
    res
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
        if exp == 0f64 {
            continue;
        }
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
