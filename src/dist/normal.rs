use crate::rng::Random;
use serde::Deserialize;
use crate::dist::Distribution;
use std::f64::consts::PI;

/// Algoritmo a usarse para la generación de una distribución Normal
#[derive(Deserialize)]
pub enum Algorithm {
    BoxMuller,
    Convolution,
}

/// Distribución Normal, permite su generación y cálculo de estadísticas
#[derive(Deserialize)]
pub struct Normal {
    /// Algoritmo a utilizar para la generación
    pub algorithm: Algorithm,
    /// Media de la distribución
    pub mean: f64,
    /// Desviación estándar de la distribución
    pub sd: f64,
    /// Para el caso de Box-Müller, next() devuelve el segundo número del par
    /// generado cuando se llama por segunda vez
    pub pair: Option<f64>,
}

impl Distribution for Normal {
    fn get_expected(&self, intervals: usize, lower: f64, upper: f64) -> Vec<f64> {
        let size = (upper - lower) / intervals as f64;
        let sd = self.sd;
        let mean = self.mean;
        let mut interval_list: Vec<f64> = Vec::with_capacity(intervals);
        let mut interval = lower + (size / 2f64);

        for _ in 0..intervals {
            let pt1 = 1f64 / (sd * f64::sqrt(2f64 * PI));
            let pt2 = (-0.5 * ((interval - mean) / sd).powi(2)).exp();
            let prob = pt1 * pt2 * size;
            interval_list.push(prob);
            interval += size;
        }
        interval_list
    }

    fn get_degrees(&self, intervals: usize) -> usize {
        if intervals >= 4 { intervals - 3 }
        else { 1 }
    }

    fn get_intervals(&self, limits: crate::stats::DistributionLimits) -> crate::stats::DistributionLimits {
        limits
    }
}

impl Normal {
    /// Devuelve el siguiente número a ser generado por la distribución
    ///
    /// # Argumentos
    ///
    /// * `rand` el generador de números aleatorios a utilizar, implementa Random
    pub fn next(&mut self, rand: &mut dyn Random) -> f64 {
        // Define la variable a devolver, de tipo float de 64 bits
        let ret: f64;
        match self.algorithm {
            Algorithm::BoxMuller => match self.pair {
                Some(x) => {
                    // Si ya hay un valor generado que todavía no se devolvió
                    // (el par del generado anterior), devolverlo
                    ret = x;
                    self.pair = None;
                }
                None => {
                    // Si no, generar un par de valores nuevos por Box-Müller
                    let gen = self.get_bm(rand);
                    // Guardar el segundo para devolverlo en la próxima invocación
                    // y devolver el primero
                    self.pair = Some(gen.1);
                    ret = gen.0;
                }
            },
            Algorithm::Convolution => {
                ret = self.get_conv(rand);
            }
        }
        ret
    }

    // Funciones privadas, para uso por el generador

    /// Devuelve un par de números generados por Box-Müller
    fn get_bm(&self, rand: &mut dyn Random) -> (f64, f64) {
        let rnd1 = rand.next();
        let rnd2 = rand.next();
        let z1 = (-2f64 * (1f64 - rnd1).ln()).sqrt() * (2f64 * PI * rnd2).cos();
        let z2 = (-2f64 * (1f64 - rnd1).ln()).sqrt() * (2f64 * PI * rnd2).sin();
        let n1 = z1 * self.sd + self.mean;
        let n2 = z2 * self.sd + self.mean;
        (n1, n2)
    }

    /// Devuelve un número generado por Convolución
    fn get_conv(&self, rand: &mut dyn Random) -> f64 {
        let mut sum = 0.0;
        for _ in 0..12 {
            sum += rand.next();
        }
        sum -= 6.0;
        self.mean + self.sd * sum
    }
}
