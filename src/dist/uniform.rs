use crate::{rng::Random, stats::DistributionLimits};
use serde::Deserialize;
use crate::dist::Distribution;

/// Distribución Uniforme, permite su generación y cálculo de estadísticas
#[derive(Deserialize)]
pub struct Uniform {
    /// Límite inferior de la distribución
    pub lower: f64,
    /// Límite superior de la distribución
    pub upper: f64,
}

impl Distribution for Uniform {
    fn get_expected(&self, intervals: usize, lower: f64, upper: f64) -> Vec<f64> {
        let size = (upper - lower) / intervals as f64;
        let mut interval_list: Vec<f64> = Vec::with_capacity(intervals);
        let mut interval = lower;
        for _ in 0..intervals {
            let inside_interval = {
                if interval >= self.lower && (interval + size <= self.upper) {
                    size
                } else if interval + size < self.lower {
                    0f64
                } else if interval >= self.upper {
                    0f64
                } else if interval < self.lower {
                    size - (self.lower - interval)
                } else {
                    self.upper - interval
                }
            };
            interval_list.push(1f64 / (self.upper - self.lower) * inside_interval);
            interval += size;
        }
        interval_list
    }

    fn get_degrees(&self, intervals: usize) -> usize {
        intervals - 1
    }

    fn get_intervals(&self, limits: DistributionLimits) -> DistributionLimits {
        DistributionLimits {
            lower: self.lower,
            upper: self.upper,
            intervals: limits.intervals,
        }
    }
}

impl Uniform {
    /// Devuelve el siguiente número a ser generado por la distribución
    ///
    /// # Argumentos
    ///
    /// * `rand` el generador de números aleatorios a utilizar, implementa Random
    pub fn next(&self, rand: &mut dyn Random) -> f64 {
        // a + RND * (b-a)
        self.lower + rand.next() * (self.upper - self.lower)
    }
}
