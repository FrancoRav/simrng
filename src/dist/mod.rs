use crate::rng::Random;
use serde::Deserialize;
use std::f64::consts::PI;

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
    fn get_degrees(&self, intervals: usize) -> u64;
}

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
    fn get_degrees(&self, intervals: usize) -> u64 {
        intervals as u64 - 3
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

    fn get_degrees(&self, intervals: usize) -> u64 {
        intervals as u64 - 1
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

    fn get_degrees(&self, intervals: usize) -> u64 {
        intervals as u64 - 2
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

/// Distribución Poisson, permite su generación y cálculo de estadísticas
#[derive(Deserialize)]
pub struct Poisson {
    /// Lambda de la distribución
    pub lambda: f64,
}

impl Distribution for Poisson {
    fn get_expected(&self, intervals: usize, lower: f64, upper: f64) -> Vec<f64> {
        let size = (upper - lower) / intervals as f64;
        let lambda = self.lambda;
        let mut interval_list: Vec<f64> = Vec::with_capacity(intervals);
        let mut interval = lower;
        for _ in 0..intervals {
            let prob = ((-lambda).exp() * lambda.powf(interval)) / factorial(interval);
            interval_list.push(prob);
            interval += size;
        }
        interval_list
    }

    fn get_degrees(&self, intervals: usize) -> u64 {
        intervals as u64 - 2
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
fn factorial(n: f64) -> f64 {
    let prod: u64 = (0..n as u64).product();
    prod as f64
}
