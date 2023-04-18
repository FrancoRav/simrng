pub mod uniform;
pub mod normal;
pub mod exponential;
pub mod poisson;

/// Interfaz requerida para cualquier distribución
pub trait Distribution {
    /// Devuelve el vector de frecuencias esperadas para cada intervalo
    /// requerido por el test de chi cuadrado
    ///
    /// # Argumentos
    /// * `intervals` cantidad de intervalos a usarse para la prueba
    /// * `lower` límite inferior de los intervalos a calcular
    /// * `upper` límite superior de los intervalos a calcular
    fn get_expected(&self, intervals: usize, lower: f64, upper: f64) -> Vec<f64>;
    /// Devuelve los grados de libertad de la distribución para la prueba
    /// de chi cuadrado
    ///
    /// # Argumentos
    /// * `intervals` cantidad de intervalos a usarse para la prueba
    fn get_degrees(&self, intervals: usize) -> usize;
}

