pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub mod dist;
pub mod rng;
pub mod stats;

#[cfg(test)]
mod tests {
    use stats::chi_squared_critical_value;
    use dist::Distribution;

    use super::*;

    #[test]
    fn test_expected_uniform() {
        let uniform = dist::Uniform {
            lower: 0f64,
            upper: 5f64,
        };
        let data = uniform.get_expected(5, 0f64, 5f64);
        assert_eq!(vec![0.2; 5], data);
    }

    #[test]
    fn test_expected_normal() {
        let normal = dist::Normal {
            mean: 10f64,
            sd: 2f64,
            algorithm: dist::Algorithm::BoxMuller,
            pair: None,
        };
        let data: Vec<f64> = normal
            .get_expected(8, 6f64, 14f64)
            .iter()
            .map(|n| f64::trunc(n * 1000000f64) / 1000000f64)
            .collect();
        assert_eq!(
            vec![
                0.043138f64,
                0.091324f64,
                0.150568f64,
                0.193334f64,
                0.193334f64,
                0.150568f64,
                0.091324f64,
                0.043138f64
            ],
            data
        );
    }

    #[test]
    fn test_critical_value() {
        let critical = chi_squared_critical_value(3f64, 0.05);
        assert_eq!((critical * 100f64).trunc() / 100f64, 7.81f64);
        let critical = chi_squared_critical_value(5f64, 0.05);
        assert_eq!((critical * 100f64).trunc() / 100f64, 11.07f64);
        let critical = chi_squared_critical_value(7f64, 0.05);
        assert_eq!((critical * 100f64).trunc() / 100f64, 14.06f64);
    }
}
