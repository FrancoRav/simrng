use crate::{dist::Distribution, stats::DistributionLimits};
use serde::Deserialize;
use crate::rng::Random;

/// Distribución Poisson, permite su generación y cálculo de estadísticas
#[derive(Deserialize)]
pub struct Poisson {
    /// Lambda de la distribución
    pub lambda: f64,
}

impl Distribution for Poisson {
    fn get_expected(&self, intervals: usize, lower: f64, _: f64) -> Vec<f64> {
        let lambda = self.lambda;
        let mut interval_list: Vec<f64> = Vec::with_capacity(intervals);
        let mut interval = lower as u64;
        for _ in 0..intervals {
            let prob = ((-lambda).exp() * lambda.powi(interval as i32)) / factorial(interval);
            interval_list.push(prob);
            interval += 1;
        }
        interval_list
    }

    fn get_degrees(&self, intervals: usize) -> usize {
        if intervals >= 3 { intervals - 2 }
        else { 1 }
    }

    fn get_intervals(&self, limits: DistributionLimits) -> DistributionLimits {
        let lower = limits.lower.trunc() as f64;
        let upper = limits.upper.trunc() as f64 + 1.0;
        DistributionLimits {
            lower,
            upper,
            intervals: (upper - lower) as usize,
        }
    }
}

impl Poisson {
    /// Devuelve el siguiente número a ser generado por la distribución
    ///
    /// # Argumentos
    ///
    /// * `rand` el generador de números aleatorios a utilizar, implementa Random
    pub fn next(&self, rand: &mut dyn Random) -> f64 {
        let mut p: f64 = 1f64;
        let mut x: i64 = -1;
        let a = f64::exp(-self.lambda);
        loop {
            let u = rand.next();
            p *= u;
            x += 1;
            if p < a {
                break;
            }
        }
        x as f64
    }
}

// Función privada, requerida por get_expected() de Poisson
fn factorial(n: u64) -> f64 {
    let prod: u64 = (1..n as u64).product();
    prod as f64
}
