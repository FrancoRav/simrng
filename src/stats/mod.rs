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

pub struct ChiInterval {
    pub lower: f64,
    pub upper: f64,
    pub fo: u64,
    pub fe: f64,
    pub c: Option<f64>,
}

impl ChiInterval {
    fn get_c(&self) -> f64 {
        (self.fo as f64 - self.fe).powi(2) / self.fe
    }

    fn merge(&mut self, other: &ChiInterval) {
        self.lower = self.lower.min(other.lower);
        self.upper = self.upper.max(other.upper);
        self.fo += other.fo;
        self.fe += other.fe;
    }
}

/// Respuesta del método full_statistics()
#[derive(Serialize)]
pub struct StatisticsResponse {
    pub histogram: HistogramData,
    pub test: TestResult,
}

/// Método que recibe la última distribución generada, la cantidad de intervalos
/// y devuelve la respuesta con el test de chi-cuadrado y los datos del histograma
pub async fn full_statistics(
    input: StatisticsInput,
    nums: Arc<Vec<f64>>,
    dist: Arc<Box<dyn Distribution + Send + Sync>>,
) -> StatisticsResponse {
    // Tomar el límite inferior y superior de la distribución
    let lower = nums
        .iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(&0f64)
        .floor();
    let upper = nums
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(&0f64)
        .ceil();
    // Tomar la cantidad de intervalos y el tamaño de cada uno
    let intervals = input.intervals;
    let size = (upper - lower) / intervals as f64;

    // Crear listas necesarias
    let mut interval_list: Vec<Interval> = Vec::with_capacity(intervals);
    let mut classmark_list: Vec<f64> = Vec::with_capacity(intervals);
    let mut interval_min = lower;

    // Agregar intervalos y marcas de clases a las listas
    for _ in 0..intervals {
        interval_list.push(Interval {
            lower: interval_min,
            upper: interval_min + size,
        });
        classmark_list.push(interval_min + size / 2f64);
        interval_min += size;
    }

    // Cantidad de hilos del CPU
    let threads: usize = std::thread::available_parallelism().unwrap().into();
    // Tamaño de cada slice del vector
    let slice_size = (nums.len() as f64 / (threads - 2) as f64).ceil() as usize;
    // Vector de frecuencias por intervalo
    let mut data_list: Vec<u64> = vec![0; intervals];

    // Vector con las frecuencias parciales
    let mut results_slice: Vec<Vec<u64>> = Vec::with_capacity(threads - 2);
    // Vector de tareas a iniciar, una por hilo del CPU, dejando 2 hilos sin utilizar
    let mut tasks = Vec::with_capacity(threads - 2);

    // Copiar el contador de referencias del vector de números generados
    let nums_clone = Arc::clone(&nums);
    // Por cada tarea a iniciar, iniciarla pasando como parámetro el vector entero,
    // el índice por donde debe empezar y terminar de procesar
    for i in 0..threads - 2 {
        let start = 0 + i * slice_size;
        let end = start + slice_size.min(nums.len() - start);
        let task = tokio::task::spawn(parse_intervals(
            nums_clone.clone(),
            intervals,
            lower,
            size,
            start,
            end,
        ));
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
    let exp_list: Vec<f64> = dist
        .get_expected(intervals, lower, upper);
    let exp_list: Vec<f64> = exp_list
        .iter()
        .map(|n| n * nums.len() as f64)
        .collect();

    let intervals: Vec<ChiInterval> = data_list
        .iter()
        .zip(exp_list)
        .zip(interval_list)
        .map(|((fo, fe), int)| ChiInterval {
            lower: int.lower,
            upper: int.upper,
            fo: *fo,
            fe,
            c: None,
        })
        .collect();

    let mut merged_intervals: Vec<ChiInterval> = Vec::with_capacity(intervals.len());
    let mut pending: Option<ChiInterval> = None;
    for mut interval in intervals {
        if let Some(int) = &mut pending {
            interval.merge(&int);
            pending = None;
        }
        if interval.fe >= 5f64 {
            merged_intervals.push(interval);
        } else {
            pending = Some(interval);
        }
    }

    if let Some(int) = &mut pending {
        match merged_intervals.last_mut() {
            Some(interval) => {
                interval.merge(&int);
            },
            None => {
                merged_intervals.push(pending.take().unwrap());
            }
        }
    }

    // Sumatoria de (fo-fe)²/fe
    let mut calculated = 0f64;
    for interval in merged_intervals.iter() {
        calculated += interval.get_c();
    }

    // Valor crítico del test de chi cuadrado
    let expected = chi_squared_critical_value(dist.get_degrees(merged_intervals.len()) as f64, 0.05);

    // Valores a devolver
    let test = TestResult {
        calculated,
        expected,
    };
    let histogram = HistogramData {
        x: classmark_list,
        y: data_list,
        lower,
        upper,
        size,
    };
    let res = StatisticsResponse { histogram, test };
    res
}

async fn parse_intervals(
    nums: Arc<Vec<f64>>,
    intervals: usize,
    lower: f64,
    size: f64,
    start: usize,
    end: usize,
) -> Vec<u64> {
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

pub fn chi_squared_critical_value(df: f64, alpha: f64) -> f64 {
    let z = 1.0 - alpha;
    let a = df / 2.0;
    let x = 2.0 * gamma_inverse(z, a);
    x
}

// Funciones auxiliares privadas, cálculo matemático del valor crítico
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
