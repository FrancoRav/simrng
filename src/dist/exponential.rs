use crate::rng::Random;
use serde::Deserialize;
use crate::dist::Distribution;

/// Distribución Exponencial, permite su generación y cálculo de estadísticas
#[derive(Deserialize)]
pub struct Exponential {
    /// Lambda de la distribución
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

    fn get_degrees(&self, intervals: usize) -> usize {
        intervals - 2
    }

    fn get_intervals(&self, limits: crate::stats::DistributionLimits) -> crate::stats::DistributionLimits {
        limits
    }
}

impl Exponential {
    /// Devuelve el siguiente número a ser generado por la distribución
    ///
    /// # Argumentos
    ///
    /// * `rand` el generador de números aleatorios a utilizar, implementa Random
    pub fn next(&self, rand: &mut dyn Random) -> f64 {
        // (-1/λ) * ln(1-RND)
        -1f64 / self.lambda * f64::ln(1f64 - rand.next())
    }
}
