use rand::{RngCore, Rng};

/// Interfaz de generador de números aleatorios
pub trait Random {
    /// Siguiente número a ser generado por el generador
    fn next(&mut self) -> f64;
}

/// Generador congruencial lineal, implementa interfaz Random
pub struct LinearCongruentialGenerator {
    /// Semilla del generador, x0
    x0: u64,
    /// Módulo
    m: u64,
    /// Multiplicador
    a: u64,
    /// Incremento
    c: u64,
}

impl LinearCongruentialGenerator {
    /// Constructor sólo con la semilla, utilizando valores aceptables para m, a y c
    pub fn with_seed(x0: u64) -> Self {
        Self {
            x0,
            m: 4294967296,
            a: 1 + (4 * 712300),
            c: 1013904223,
        }
    }

    /// Constructor completo
    pub fn new(x0: u64, m: u64, a: u64, c: u64) -> Self {
        Self { x0, m, a, c }
    }
}

impl Random for LinearCongruentialGenerator {
    fn next(&mut self) -> f64 {
        self.x0 = (self.a * self.x0 + self.c) % self.m;
        self.x0 as f64 / self.m as f64
    }
}

impl<T: Rng> Random for T {
    fn next(&mut self) -> f64 {
        self.gen_range(0.0..1.0)
    }
}
