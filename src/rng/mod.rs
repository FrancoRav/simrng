pub trait Random {
    fn next(&mut self) -> f64;
}

pub struct LinearCongruentialGenerator {
    x0: u64, // seed
    m: u64,  // modulus
    a: u64,  // multiplier
    c: u64,  // increment
}

impl LinearCongruentialGenerator {
    pub fn new(x0: u64) -> Self {
        Self {
            x0,
            m: 4294967296,
            a: 1 + (4 * 712300),
            c: 1013904223,
        }
    }
}

impl Random for LinearCongruentialGenerator {
    fn next(&mut self) -> f64 {
        self.x0 = (self.a * self.x0 + self.c) % self.m;
        self.x0 as f64 / self.m as f64
    }
}
