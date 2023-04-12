use axum::extract::State;
use axum::{extract, Json};
use serde::Deserialize;
use simrng::dist::{Distribution, Exponential, Normal, Poisson, Uniform};
use simrng::rng::LinearCongruentialGenerator;
use simrng::stats::{
    chi_squared_test, full_statistics, generate_histogram, HistogramData, StatisticsInput,
    StatisticsResponse, TestResult,
};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Tipo de distribución: parámetro para la generación de números
#[derive(Deserialize)]
pub enum DistributionType {
    Normal,
    Uniform,
    Exponential,
    Poisson,
}

/// Parámetros para la generación de valores
#[derive(Deserialize)]
pub struct GenerationParameters {
    /// Semilla a partir de la cual se genera la distribución
    pub seed: u64,
    /// Cantidad de valores a generar
    pub number: u64,
    /// Tipo de distribución a generar
    pub distribution: DistributionType,
    /// Parámetros para la distribución, de tipo Distribution
    pub data: serde_json::Value,
}

/// Últimos datos generados, con los parámetros de su distribución
pub struct Generated {
    /// Vector de números generados
    pub data: Vec<f64>,
    /// Parámetros de la distribución
    pub dist: Arc<Box<dyn Distribution + Send + Sync>>,
}

impl Generated {
    pub fn new(data: Vec<f64>, dist: Box<dyn Distribution + Send + Sync>) -> Self {
        let dist = Arc::new(dist);
        Self { data, dist }
    }
}

/// Método handler de las peticiones de generación de valores
///
/// # Argumentos
///
/// * `State(arc)` Un wrapper state al Arc que contiene el RwLock del estado
/// * `data` Datos en Json recibidos del front end
pub async fn get_unified(
    State(arc): State<Arc<RwLock<Generated>>>,
    data: extract::Json<GenerationParameters>,
) {
    // Asegurarse de que ningún otro hilo pueda acceder al estado
    let mut arc = arc.write().await;
    arc.data = vec![];
    // Crear una instancia de generador de números aleatorios, con la semilla
    // de los parámetros de la generación
    let mut rng = LinearCongruentialGenerator::with_seed(data.seed);
    // Crear el vector en el que se guardan los datos, con capacidad
    // suficiente para la cantidad de valores a generar
    let mut res = Vec::with_capacity(data.number as usize);
    // Distribución a almacenar posteriormente en el estado
    let dist: Box<dyn Distribution + Send + Sync>;
    // Según la distribución, llamar al método correcto
    // No se usa método de interfaz por rendimiento al usar dynamic dispatch
    match data.distribution {
        DistributionType::Normal => {
            let mut distribution = serde_json::from_value::<Normal>(data.data.clone()).unwrap();
            for _ in 0..data.number {
                res.push(distribution.next(&mut rng));
            }
            dist = Box::new(distribution);
        }
        DistributionType::Uniform => {
            let distribution = serde_json::from_value::<Uniform>(data.data.clone()).unwrap();
            for _ in 0..data.number {
                res.push(distribution.next(&mut rng));
            }
            dist = Box::new(distribution);
        }
        DistributionType::Exponential => {
            let distribution = serde_json::from_value::<Exponential>(data.data.clone()).unwrap();
            for _ in 0..data.number {
                res.push(distribution.next(&mut rng));
            }
            dist = Box::new(distribution);
        }
        DistributionType::Poisson => {
            let distribution = serde_json::from_value::<Poisson>(data.data.clone()).unwrap();
            for _ in 0..data.number {
                res.push(distribution.next(&mut rng));
            }
            dist = Box::new(distribution);
        }
    }
    // Guardar el vector generado y la distribución utilizada
    *arc = Generated::new(res, dist);
}

/// Método handler de la petición de generación de histograma
pub async fn get_histogram(
    State(arc): State<Arc<RwLock<Generated>>>,
    data: extract::Json<StatisticsInput>,
) -> Json<HistogramData> {
    let data = data.0;
    let arc = arc.read().await;
    Json(generate_histogram(data, &arc.data))
}

/// Método handler de la petición de test de chi cuadrado
pub async fn get_chisquared(
    State(arc): State<Arc<RwLock<Generated>>>,
    data: extract::Json<StatisticsInput>,
) -> Json<TestResult> {
    let data = data.0;
    let arc = arc.read().await;
    let dist = arc.dist.clone();
    let res = chi_squared_test(data, &arc.data, dist);
    Json(res)
}

/// Método handler de las peticiones de cálculo de estadísticas
/// Devuelve Json con histogram, de tipo HistogramData, y test, de tipo
/// TestResult
///
/// # Argumentos
///
/// * `State(arc)` Un wrapper state al Arc que contiene el RwLock del estado
/// * `data` Datos en Json recibidos del front end
pub async fn get_statistics(
    State(arc): State<Arc<RwLock<Generated>>>,
    data: extract::Json<StatisticsInput>,
) -> Json<StatisticsResponse> {
    // Extraer el Input del body Json
    let data = data.0;
    // Bloquear el estado para lectura
    let arc = arc.read().await;
    // Clonar la distribución (se podría pasar una referencia?)
    let dist = arc.dist.clone();
    // Guardar la respuesta del método y devolverla como Json
    let res = full_statistics(data, &arc.data, dist);
    Json(res)
}
