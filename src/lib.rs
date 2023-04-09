pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub mod rng;
pub mod dist;
pub mod stats;

#[cfg(test)]
mod tests {
    use crate::stats::Distribution;

    use super::*;

    #[test]
    fn test_expected_uniform() {
        let uniform = stats::Uniform { lower: 0f64, upper: 5f64 };
        let data = uniform.get_expected(5, 0f64, 5f64);
        assert_eq!(vec![0.2; 5], data);
    }

    #[test]
    fn test_expected_normal() {
        let normal = stats::Normal { mean: 10f64, sd: 2f64 };
        let data: Vec<f64> = normal.get_expected(8, 6f64, 14f64).iter().map(|n| f64::trunc(n*1000000f64)/1000000f64).collect();
        assert_eq!(vec![0.043138f64, 0.091324f64, 0.150568f64, 0.193334f64, 0.193334f64, 0.150568f64, 0.091324f64, 0.043138f64], data);
    }
}

