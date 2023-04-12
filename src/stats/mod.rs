use serde::{Deserialize, Serialize};
use special_fun::cephes_double;
use std::sync::Arc;

use crate::dist::Distribution;

/// Datos necesarios para calcular estadísticas
#[derive(Deserialize)]
pub struct StatisticsInput {
    /// Cantidad de intervalos a utilizar para los cálculos
    pub intervals: usize,
}

/// Datos a devolver para la generación del histograma
#[derive(Serialize)]
pub struct HistogramData {
    pub x: Vec<f64>,
    pub y: Vec<u64>,
    pub lower: f64,
    pub upper: f64,
    pub size: f64,
}

/// Datos a devolver como resultado del test de chi cuadrado
#[derive(Serialize)]
pub struct TestResult {
    pub calculated: f64,
    pub expected: f64,
}

/// Respuesta del método full_statistics()
#[derive(Serialize)]
pub struct StatisticsResponse {
    pub histogram: HistogramData,
    pub test: TestResult,
}

pub fn generate_histogram(input: StatisticsInput, nums: &Vec<f64>) -> HistogramData {
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

/// Método que recibe la última distribución generada, la cantidad de intervalos
/// y devuelve la respuesta con el test de chi-cuadrado y los datos del histograma
pub async fn full_statistics(
    input: StatisticsInput,
    nums: Arc<Vec<f64>>,
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
    let threads: usize = std::thread::available_parallelism().unwrap().into();
    dbg!(threads);
    let slice_size = (nums.len() as f64 / (threads - 2) as f64).ceil() as usize;
    let mut results_slice: Vec<Vec<u64>> = Vec::with_capacity(threads-2);

    let mut tasks = Vec::with_capacity(threads - 2);

    let nums_clone = Arc::clone(&nums);
    for i in 0..threads-2 {
        let start = 0 + i * slice_size;
        let end = start + slice_size.min(nums.len() - start);
        let task = tokio::task::spawn(parse_intervals(nums_clone.clone(), intervals, lower, size, start, end));
        tasks.push(task);
    }

    for task in tasks {
        let result = task.await.unwrap();
        results_slice.push(result);
    }

    for vec in results_slice {
        for (i, &x) in vec.iter().enumerate() {
            data_list[i] += x;
        }
    }
    /*for num in nums.iter() {
        let ind = ((num - lower) / size) as usize;
        let ind = ind.min(intervals - 1);
        data_list[ind] += 1;
    }*/

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

async fn parse_intervals(nums: Arc<Vec<f64>>, intervals: usize, lower: f64, size: f64, start: usize, end: usize) -> Vec<u64> {
    let mut data_list: Vec<u64> = vec![0; intervals];
    let opt = nums.get(start..end);
    match opt {
        Some(nums) => {
            for num in nums.iter() {
                let ind = ((num - lower) / size) as usize;
                let ind = ind.min(intervals - 1);
                data_list[ind] += 1;
            }
        }
        None => {}
    }
    data_list
}

pub fn chi_squared_test(
    input: StatisticsInput,
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
