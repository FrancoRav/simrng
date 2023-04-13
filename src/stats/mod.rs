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

#[derive(Serialize)]
pub struct Interval {
    pub lower: f64,
    pub upper: f64,
}

impl Interval {
    fn contains(&self, n: f64) -> bool {
        return n >= self.lower && n < self.upper
    }
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

    // Cantidad de hilos del CPU
    let threads: usize = std::thread::available_parallelism().unwrap().into();
    // Tamaño de cada slice del vector
    let slice_size = (nums.len() as f64 / (threads - 2) as f64).ceil() as usize;
    // Vector de frecuencias por intervalo
    let mut data_list: Vec<u64> = vec![0; intervals];

    // Vector con las frecuencias parciales
    let mut results_slice: Vec<Vec<u64>> = Vec::with_capacity(threads-2);
    // Vector de tareas a iniciar, una por hilo del CPU, dejando 2 hilos sin utilizar
    let mut tasks = Vec::with_capacity(threads - 2);

    // Copiar el contador de referencias del vector de números generados
    let nums_clone = Arc::clone(&nums);
    // Por cada tarea a iniciar, iniciarla pasando como parámetro el vector entero,
    // el índice por donde debe empezar y terminar de procesar
    for i in 0..threads-2 {
        let start = 0 + i * slice_size;
        let end = start + slice_size.min(nums.len() - start);
        let task = tokio::task::spawn(parse_intervals(nums_clone.clone(), intervals, lower, size, start, end));
        tasks.push(task);
    }

    // Obtener los resultados de las tareas una vez que terminen
    for task in tasks {
        let result = task.await.unwrap();
        results_slice.push(result);
    }

    // Guardar los resultados en la lista final
    for vec in results_slice {
        for (i, &x) in vec.iter().enumerate() {
            data_list[i] += x;
        }
    }

    // Obtener las frecuencias esperadas según la distribución
    let exp_list: Vec<f64> = dist.get_expected(intervals, lower, upper)
        .iter()
        .map(|n| n * nums.len() as f64)
        .collect();

    // Cantidad de valores generados
    let len = nums.len() as f64;
    let mut calculated = 0f64;
    // Sumatoria de (fo-fe)²/fe
    let mut parsed_exp_list: Vec<f64> = Vec::with_capacity(intervals);
    let mut parsed_obs_list: Vec<u64> = Vec::with_capacity(intervals);
    let min_count = 5f64;
    let mut new_intervals: usize = 0;
    let mut current_obs = 0;
    let mut current_exp = 0f64;
    for (obs, exp) in data_list.iter().zip(exp_list) {
        current_obs += obs;
        current_exp += exp;
        if current_exp >= min_count {
            parsed_obs_list.push(current_obs);
            parsed_exp_list.push(current_exp);
            new_intervals += 1;
            current_obs = 0;
            current_exp = 0f64;
        }
    }
    if current_exp >= min_count {
        parsed_obs_list.push(current_obs);
        parsed_exp_list.push(current_exp);
        new_intervals += 1;
    }
    else {
        let laste = parsed_exp_list.pop();
        match laste {
            Some(x) => {
                parsed_exp_list.push(x + current_exp);
                let lasto = parsed_obs_list.pop().unwrap();
                parsed_obs_list.push(lasto + current_obs);
            }
            None => {
                parsed_exp_list.push(current_exp);
                parsed_obs_list.push(current_obs);
                new_intervals += 1;
            }
        }
    }
    for (obs, exp) in parsed_obs_list.iter().zip(parsed_exp_list) {
        // Evitar errores de división por 0
        if exp == 0f64 {
            continue;
        }
        calculated += (*obs as f64 - exp).powi(2) / (exp);
    }

    // Valor crítico del test de chi cuadrado
    let expected = chi_squared_critical_value(dist.get_degrees(new_intervals) as f64, 0.05);

    // Valores a devolver
    let test = TestResult {
        calculated,
        expected,
    };
    let histogram = HistogramData {
        x: interval_list,
        y: data_list,
        lower,
        upper,
        size,
    };
    let res = StatisticsResponse {
        histogram,
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
