use serde::{Deserialize, Serialize};
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
    /// tabla de cálculo
    pub intervals: Vec<ChiInterval>,
    /// chi cuadrado calculado
    pub calculated: f64,
    /// valor crítico, chi cuadrado tabulado
    pub critical: f64,
}

#[derive(Serialize)]
pub struct Interval {
    pub lower: f64,
    pub upper: f64,
}

/// Fila de la tabla del cálculo de Chi Cuadrado
#[derive(Serialize)]
pub struct ChiInterval {
    pub lower: f64,
    pub upper: f64,
    /// frecuencia observada
    pub fo: u64,
    /// frecuencia esperada
    pub fe: f64,
    pub c: Option<f64>,
    pub c_ac: Option<f64>,
}

impl ChiInterval {
    fn set_c(&mut self, cumulative: f64) -> f64 {
        let c = (self.fo as f64 - self.fe).powi(2) / self.fe;
        self.c = Some(c);
        self.c_ac = Some(cumulative + c);
        c
    }

    /// Une el intervalo con otro, pasado como referencia
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
        .get_expected(intervals, lower, upper)
        .iter()
        .map(|n| n * nums.len() as f64)
        .collect();

    // Unir la lista de intervalos, de frecuencias esperadas y de frecuencias observadas
    // en una lista de ChiInterval
    let intervals: Vec<ChiInterval> = data_list
        .iter()
        // Iterar sobre las tres listas a la vez
        .zip(exp_list)
        .zip(interval_list)
        // por cada intervalo, crear el objeto necesario
        .map(|((fo, fe), int)| ChiInterval {
            lower: int.lower,
            upper: int.upper,
            fo: *fo,
            fe,
            c: None,
            c_ac: None,
        })
        .collect();

    let mut merged_intervals = merge_intervals(intervals);

    // Sumatoria de (fo-fe)²/fe
    let mut calculated = 0f64;
    for interval in merged_intervals.iter_mut() {
        calculated += interval.set_c(calculated);
    }

    // Valor crítico del test de chi cuadrado
    let critical =
        chi_squared_critical_value(dist.get_degrees(merged_intervals.len()), 7);

    // Valores a devolver
    let test = TestResult {
        intervals: merged_intervals,
        calculated,
        critical,
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

fn merge_intervals(intervals: Vec<ChiInterval>) -> Vec<ChiInterval> {
    // Lista de intervalos después de combinar los que tienen fe < 5
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
            }
            None => {
                merged_intervals.push(pending.take().unwrap());
            }
        }
    }
    merged_intervals
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
    if let Some(nums) = opt {
        for num in nums.iter() {
            let ind = ((num - lower) / size) as usize;
            let ind = ind.min(intervals - 1);
            data_list[ind] += 1;
        }
    }
    data_list
}

pub fn chi_squared_critical_value(df: u64, alpha: u64) -> f64 {
    let list: [[f32; 100]; 10] = [
        [
            0.0, 0.002, 0.024, 0.091, 0.21, 0.381, 0.598, 0.857, 1.152, 1.479, 1.8341, 2.2142,
            2.6173, 3.0414, 3.4835, 3.9426, 4.4167, 4.9058, 5.4079, 5.921, 6.4471, 6.9832, 7.5293,
            8.0854, 8.6495, 9.2226, 9.8037, 10.3918, 10.9869, 11.588, 12.1961, 12.8112, 13.4313,
            14.0574, 14.6885, 15.3246, 15.9657, 16.6118, 17.2629, 17.916, 18.5751, 19.2392,
            19.9063, 20.5764, 21.2515, 21.9296, 22.6107, 23.2958, 23.9839, 24.674, 25.3681,
            26.0652, 26.7653, 27.4684, 28.1735, 28.8816, 29.5927, 30.3058, 31.0209, 31.738,
            32.4591, 33.1812, 33.9063, 34.6334, 35.3625, 36.0936, 36.8267, 37.5618, 38.2989,
            39.036, 39.7771, 40.5192, 41.2643, 42.0104, 42.7575, 43.5076, 44.2587, 45.0108,
            45.7649, 46.52, 47.2771, 48.0362, 48.7963, 49.5574, 50.3205, 51.0856, 51.8507, 52.6178,
            53.3869, 54.155, 54.9261, 55.6982, 56.4723, 57.2464, 58.0225, 58.7996, 59.5777,
            60.3568, 61.1379, 61.918,
        ],
        [
            0.0, 0.02, 0.115, 0.297, 0.554, 0.872, 1.239, 1.646, 2.088, 2.558, 3.053, 3.571, 4.107,
            4.66, 5.229, 5.812, 6.408, 7.015, 7.633, 8.26, 8.897, 9.542, 10.196, 10.856, 11.524,
            12.198, 12.879, 13.565, 14.256, 14.953, 15.655, 16.362, 17.074, 17.789, 18.509, 19.233,
            19.96, 20.691, 21.426, 22.164, 22.906, 23.65, 24.398, 25.148, 25.901, 26.657, 27.416,
            28.177, 28.941, 29.707, 30.475, 31.246, 32.018, 32.793, 33.57, 34.35, 35.131, 35.913,
            36.698, 37.485, 38.273, 39.063, 39.855, 40.649, 41.444, 42.24, 43.038, 43.838, 44.639,
            45.442, 46.246, 47.051, 47.858, 48.666, 49.475, 50.286, 51.097, 51.91, 52.725, 53.54,
            54.357, 55.174, 55.993, 56.813, 57.634, 58.456, 59.279, 60.103, 60.928, 61.754, 62.581,
            63.409, 64.238, 65.068, 65.898, 66.73, 67.562, 68.396, 69.23, 70.065,
        ],
        [
            0.001, 0.051, 0.216, 0.484, 0.831, 1.237, 1.69, 2.18, 2.7, 3.247, 3.816, 4.404, 5.009,
            5.629, 6.262, 6.908, 7.564, 8.231, 8.907, 9.591, 10.283, 10.982, 11.689, 12.401, 13.12,
            13.844, 14.573, 15.308, 16.047, 16.791, 17.539, 18.291, 19.047, 19.806, 20.569, 21.336,
            22.106, 22.878, 23.654, 24.433, 25.215, 25.999, 26.785, 27.575, 28.366, 29.16, 29.956,
            30.755, 31.555, 32.357, 33.162, 33.968, 34.776, 35.586, 36.398, 37.212, 38.027, 38.844,
            39.662, 40.482, 41.303, 42.126, 42.95, 43.776, 44.603, 45.431, 46.261, 47.092, 47.924,
            48.758, 49.592, 50.428, 51.265, 52.103, 52.942, 53.782, 54.623, 55.466, 56.309, 57.153,
            57.998, 58.845, 59.692, 60.54, 61.389, 62.239, 63.089, 63.941, 64.793, 65.647, 66.501,
            67.356, 68.211, 69.068, 69.925, 70.783, 71.642, 72.501, 73.361, 74.222,
        ],
        [
            0.004, 0.103, 0.352, 0.711, 1.145, 1.635, 2.167, 2.733, 3.325, 3.94, 4.575, 5.226,
            5.892, 6.571, 7.261, 7.962, 8.672, 9.39, 10.117, 10.851, 11.591, 12.338, 13.091,
            13.848, 14.611, 15.379, 16.151, 16.928, 17.708, 18.493, 19.281, 20.072, 20.867, 21.664,
            22.465, 23.269, 24.075, 24.884, 25.695, 26.509, 27.326, 28.144, 28.965, 29.787, 30.612,
            31.439, 32.268, 33.098, 33.93, 34.764, 35.6, 36.437, 37.276, 38.116, 38.958, 39.801,
            40.646, 41.492, 42.339, 43.188, 44.038, 44.889, 45.741, 46.595, 47.45, 48.305, 49.162,
            50.02, 50.879, 51.739, 52.6, 53.462, 54.325, 55.189, 56.054, 56.92, 57.786, 58.654,
            59.522, 60.391, 61.261, 62.132, 63.004, 63.876, 64.749, 65.623, 66.498, 67.373, 68.249,
            69.126, 70.003, 70.882, 71.76, 72.64, 73.52, 74.401, 75.282, 76.164, 77.046, 77.929,
        ],
        [
            0.016, 0.211, 0.584, 1.064, 1.61, 2.204, 2.833, 3.49, 4.168, 4.865, 5.578, 6.304,
            7.042, 7.79, 8.547, 9.312, 10.085, 10.865, 11.651, 12.443, 13.24, 14.041, 14.848,
            15.659, 16.473, 17.292, 18.114, 18.939, 19.768, 20.599, 21.434, 22.271, 23.11, 23.952,
            24.797, 25.643, 26.492, 27.343, 28.196, 29.051, 29.907, 30.765, 31.625, 32.487, 33.35,
            34.215, 35.081, 35.949, 36.818, 37.689, 38.56, 39.433, 40.308, 41.183, 42.06, 42.937,
            43.816, 44.696, 45.577, 46.459, 47.342, 48.226, 49.111, 49.996, 50.883, 51.77, 52.659,
            53.548, 54.438, 55.329, 56.221, 57.113, 58.006, 58.9, 59.795, 60.69, 61.586, 62.483,
            63.38, 64.278, 65.176, 66.076, 66.976, 67.876, 68.777, 69.679, 70.581, 71.484, 72.387,
            73.291, 74.196, 75.1, 76.006, 76.912, 77.818, 78.725, 79.633, 80.541, 81.449, 82.358,
        ],
        [
            2.706, 4.605, 6.251, 7.779, 9.236, 10.645, 12.017, 13.362, 14.684, 15.987, 17.275,
            18.549, 19.812, 21.064, 22.307, 23.542, 24.769, 25.989, 27.204, 28.412, 29.615, 30.813,
            32.007, 33.196, 34.382, 35.563, 36.741, 37.916, 39.087, 40.256, 41.422, 42.585, 43.745,
            44.903, 46.059, 47.212, 48.363, 49.513, 50.66, 51.805, 52.949, 54.09, 55.23, 56.369,
            57.505, 58.641, 59.774, 60.907, 62.038, 63.167, 64.295, 65.422, 66.548, 67.673, 68.796,
            69.919, 71.04, 72.16, 73.279, 74.397, 75.514, 76.63, 77.745, 78.86, 79.973, 81.085,
            82.197, 83.308, 84.418, 85.527, 86.635, 87.743, 88.85, 89.956, 91.061, 92.166, 93.27,
            94.374, 95.476, 96.578, 97.68, 98.78, 99.88, 100.98, 102.079, 103.177, 104.275,
            105.372, 106.469, 107.565, 108.661, 109.756, 110.85, 111.944, 113.038, 114.131,
            115.223, 116.315, 117.407, 118.498,
        ],
        [
            3.841, 5.991, 7.815, 9.488, 11.07, 12.592, 14.067, 15.507, 16.919, 18.307, 19.675,
            21.026, 22.362, 23.685, 24.996, 26.296, 27.587, 28.869, 30.144, 31.41, 32.671, 33.924,
            35.172, 36.415, 37.652, 38.885, 40.113, 41.337, 42.557, 43.773, 44.985, 46.194, 47.4,
            48.602, 49.802, 50.998, 52.192, 53.384, 54.572, 55.758, 56.942, 58.124, 59.304, 60.481,
            61.656, 62.83, 64.001, 65.171, 66.339, 67.505, 68.669, 69.832, 70.993, 72.153, 73.311,
            74.468, 75.624, 76.778, 77.931, 79.082, 80.232, 81.381, 82.529, 83.675, 84.821, 85.965,
            87.108, 88.25, 89.391, 90.531, 91.67, 92.808, 93.945, 95.081, 96.217, 97.351, 98.484,
            99.617, 100.749, 101.879, 103.01, 104.139, 105.267, 106.395, 107.522, 108.648, 109.773,
            110.898, 112.022, 113.145, 114.268, 115.39, 116.511, 117.632, 118.752, 119.871, 120.99,
            122.108, 123.225, 124.342,
        ],
        [
            5.024, 7.378, 9.348, 11.143, 12.833, 14.449, 16.013, 17.535, 19.023, 20.483, 21.92,
            23.337, 24.736, 26.119, 27.488, 28.845, 30.191, 31.526, 32.852, 34.17, 35.479, 36.781,
            38.076, 39.364, 40.646, 41.923, 43.195, 44.461, 45.722, 46.979, 48.232, 49.48, 50.725,
            51.966, 53.203, 54.437, 55.668, 56.896, 58.12, 59.342, 60.561, 61.777, 62.99, 64.201,
            65.41, 66.617, 67.821, 69.023, 70.222, 71.42, 72.616, 73.81, 75.002, 76.192, 77.38,
            78.567, 79.752, 80.936, 82.117, 83.298, 84.476, 85.654, 86.83, 88.004, 89.177, 90.349,
            91.519, 92.689, 93.856, 95.023, 96.189, 97.353, 98.516, 99.678, 100.839, 101.999,
            103.158, 104.316, 105.473, 106.629, 107.783, 108.937, 110.09, 111.242, 112.393,
            113.544, 114.693, 115.841, 116.989, 118.136, 119.282, 120.427, 121.571, 122.715,
            123.858, 125.0, 126.141, 127.282, 128.422, 129.561,
        ],
        [
            6.635, 9.21, 11.345, 13.277, 15.086, 16.812, 18.475, 20.09, 21.666, 23.209, 24.725,
            26.217, 27.688, 29.141, 30.578, 32.0, 33.409, 34.805, 36.191, 37.566, 38.932, 40.289,
            41.638, 42.98, 44.314, 45.642, 46.963, 48.278, 49.588, 50.892, 52.191, 53.486, 54.776,
            56.061, 57.342, 58.619, 59.893, 61.162, 62.428, 63.691, 64.95, 66.206, 67.459, 68.71,
            69.957, 71.201, 72.443, 73.683, 74.919, 76.154, 77.386, 78.616, 79.843, 81.069, 82.292,
            83.513, 84.733, 85.95, 87.166, 88.379, 89.591, 90.802, 92.01, 93.217, 94.422, 95.626,
            96.828, 98.028, 99.228, 100.425, 101.621, 102.816, 104.01, 105.202, 106.393, 107.583,
            108.771, 109.958, 111.144, 112.329, 113.512, 114.695, 115.876, 117.057, 118.236,
            119.414, 120.591, 121.767, 122.942, 124.116, 125.289, 126.462, 127.633, 128.803,
            129.973, 131.141, 132.309, 133.476, 134.642, 135.807,
        ],
        [
            10.828, 13.816, 16.266, 18.467, 20.515, 22.458, 24.322, 26.125, 27.877, 29.588, 31.264,
            32.91, 34.528, 36.123, 37.697, 39.252, 40.79, 42.312, 43.82, 45.315, 46.797, 48.268,
            49.728, 51.179, 52.62, 54.052, 55.476, 56.892, 58.301, 59.703, 61.098, 62.487, 63.87,
            65.247, 66.619, 67.985, 69.347, 70.703, 72.055, 73.402, 74.745, 76.084, 77.419, 78.75,
            80.077, 81.4, 82.72, 84.037, 85.351, 86.661, 87.968, 89.272, 90.573, 91.872, 93.168,
            94.461, 95.751, 97.039, 98.324, 99.607, 100.888, 102.166, 103.442, 104.716, 105.988,
            107.258, 108.526, 109.791, 111.055, 112.317, 113.577, 114.835, 116.092, 117.346,
            118.599, 119.85, 121.1, 122.348, 123.594, 124.839, 126.083, 127.324, 128.565, 129.804,
            131.041, 132.277, 133.512, 134.746, 135.978, 137.208, 138.438, 139.666, 140.893,
            142.119, 143.344, 144.567, 145.789, 147.01, 148.23, 149.449,
        ],
    ];
    let df = df.min(100);
    list[alpha as usize - 1][df as usize - 1] as f64
}

