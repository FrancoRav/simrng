pub mod dist;
pub mod rng;
pub mod stats;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        dist::Normal,
        rng::{LinearCongruentialGenerator, Random},
    };
    use dist::Distribution;
    use stats::chi_squared_critical_value;

    #[test]
    fn test_expected_uniform() {
        let uniform = dist::Uniform {
            lower: 0f64,
            upper: 5f64,
        };
        let data = uniform.get_expected(5, 0f64, 5f64);
        assert_eq!(vec![0.2; 5], data);
        let uniform = dist::Uniform {
            lower: 0f64,
            upper: 0.5,
        };
        let data = uniform.get_expected(4, 0f64, 1f64);
        assert_eq!(vec![0.5, 0.5, 0f64, 0f64], data);
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

    #[test]
    fn test_normal_bm() {
        let mut normal = Normal {
            mean: 10f64,
            sd: 2f64,
            algorithm: dist::Algorithm::BoxMuller,
            pair: None,
        };
        let mut random = LinearCongruentialGenerator::new(6, 8, 13, 7);
        assert_eq!(trunc_to_dec(normal.next(&mut random), 4), 12.8011);
        assert_eq!(trunc_to_dec(normal.next(&mut random), 4), 10.0);
    }

    #[test]
    fn test_normal_conv() {
        let mut normal = Normal {
            mean: 10f64,
            sd: 2f64,
            algorithm: dist::Algorithm::Convolution,
            pair: None,
        };
        let mut random = LinearCongruentialGenerator::new(6, 8, 13, 7);
        assert_eq!(trunc_to_dec(normal.next(&mut random), 4), 8.5);
        // 0.625 0.000 0.875 0.250 0.125 0.500 0.375 0.750 0.625 0.000 0.875 0.250
    }

    #[test]
    fn test_random() {
        let mut random = LinearCongruentialGenerator::new(6, 8, 13, 7);
        assert_eq!(trunc_to_dec(random.next(), 4), 0.625);
        assert_eq!(trunc_to_dec(random.next(), 4), 0.000);
        assert_eq!(trunc_to_dec(random.next(), 4), 0.875);
    }

    fn trunc_to_dec(num: f64, dec: i32) -> f64 {
        (num * 10f64.powi(dec)).trunc() / 10f64.powi(dec)
    }
}
