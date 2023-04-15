pub mod dist;
pub mod list;
pub mod rng;
pub mod stats;

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::{
        dist::Normal,
        rng::{LinearCongruentialGenerator, Random},
        stats::{full_statistics, TestResult},
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

        let normal = dist::Normal {
            mean: 182.41f64,
            sd: 7.01f64,
            algorithm: dist::Algorithm::BoxMuller,
            pair: None,
        };
        let data: Vec<f64> = normal
            .get_expected(12, 164f64, 201.00000001f64)
            .iter()
            .map(|n| f64::trunc(n * 1000f64) / 1000f64)
            .collect();
        assert_eq!(
            vec![
                0.009, 0.025, 0.054, 0.097, 0.142, 0.171, 0.170, 0.139, 0.094, 0.052, 0.024, 0.009,
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
    }

    #[test]
    fn test_random() {
        let mut random = LinearCongruentialGenerator::new(6, 8, 13, 7);
        assert_eq!(trunc_to_dec(random.next(), 4), 0.625);
        assert_eq!(trunc_to_dec(random.next(), 4), 0.000);
        assert_eq!(trunc_to_dec(random.next(), 4), 0.875);
    }

    #[test]
    fn test_chisquared() {
        let nums: Vec<f64> = vec![
            180f64, 183f64, 191f64, 177f64, 172f64, 175f64, 174f64, 176f64, 187f64, 173f64, 182f64,
            183f64, 186f64, 178f64, 196f64, 178f64, 196f64, 185f64, 190f64, 178f64, 192f64, 182f64,
            178f64, 195f64, 179f64, 183f64, 189f64, 175f64, 174f64, 183f64, 173f64, 184f64, 189f64,
            198f64, 181f64, 175f64, 172f64, 193f64, 180f64, 171f64, 194f64, 173f64, 183f64, 194f64,
            176f64, 185f64, 188f64, 175f64, 188f64, 194f64, 185f64, 189f64, 187f64, 180f64, 189f64,
            189f64, 175f64, 190f64, 176f64, 180f64, 183f64, 175f64, 178f64, 183f64, 185f64, 193f64,
            178f64, 178f64, 175f64, 185f64, 183f64, 193f64, 193f64, 190f64, 187f64, 178f64, 193f64,
            182f64, 188f64, 180f64, 175f64, 180f64, 183f64, 185f64, 185f64, 196f64, 182f64, 164f64,
            176f64, 176f64, 179f64, 182f64, 175f64, 198f64, 183f64, 175f64, 198f64, 189f64, 190f64,
            173f64, 191f64, 170f64, 182f64, 183f64, 195f64, 177f64, 179f64, 180f64, 191f64, 191f64,
            190f64, 188f64, 180f64, 178f64, 183f64, 183f64, 180f64, 178f64, 182f64, 188f64, 191f64,
            174f64, 173f64, 175f64, 180f64, 178f64, 180f64, 188f64, 195f64, 178f64, 170f64, 191f64,
            174f64, 187f64, 175f64, 175f64, 190f64, 183f64, 183f64, 179f64, 183f64, 180f64, 185f64,
            185f64, 173f64, 188f64, 185f64, 186f64, 173f64, 176f64, 183f64, 178f64, 185f64, 180f64,
            188f64, 170f64, 179f64, 188f64, 183f64, 188f64, 178f64, 188f64, 171f64, 170f64, 181f64,
            182f64, 186f64, 178f64, 173f64, 183f64, 183f64, 185f64, 175f64, 178f64, 182f64, 167f64,
            168f64, 183f64, 188f64, 181f64, 191f64, 180f64, 191f64, 196f64, 181f64, 189f64, 174f64,
            178f64, 178f64, 180f64, 185f64, 183f64, 176f64, 190f64, 191f64, 178f64, 168f64, 189f64,
            170f64, 170f64, 193f64, 193f64, 176f64, 175f64, 176f64, 182f64, 181f64, 173f64, 196f64,
            188f64, 185f64, 195f64, 194f64, 183f64, 188f64, 174f64, 183f64, 175f64, 186f64, 187f64,
            183f64, 173f64, 184f64, 191f64, 173f64, 180f64, 178f64, 183f64, 180f64, 175f64, 193f64,
            175f64, 188f64, 181f64, 180f64, 188f64, 185f64, 187f64, 183f64, 173f64, 192f64, 186f64,
            192f64, 173f64, 188f64, 177f64, 178f64, 180f64, 196f64, 201f64, 183f64, 174f64, 178f64,
            177f64, 182f64, 186f64, 175f64, 188f64, 181f64, 180f64, 182f64, 191f64, 186f64, 183f64,
            193f64, 197f64, 189f64, 185f64, 183f64, 177f64, 176f64, 183f64, 189f64, 191f64, 187f64,
            185f64, 178f64, 175f64, 175f64, 195f64, 174f64, 181f64, 178f64, 179f64, 186f64, 178f64,
            191f64, 168f64, 185f64, 191f64, 180f64, 170f64, 177f64, 180f64, 183f64, 188f64, 188f64,
            180f64, 191f64, 182f64, 178f64, 180f64, 182f64, 182f64, 177f64, 174f64, 188f64, 182f64,
            170f64, 183f64, 185f64, 184f64, 173f64, 191f64, 175f64,
        ];
        let normal = Normal {
            algorithm: dist::Algorithm::BoxMuller,
            mean: 182.41f64,
            sd: 7.01f64,
            pair: None,
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        let res = rt.block_on(full_statistics(
            stats::StatisticsInput { intervals: 12 },
            Arc::new(nums),
            Arc::new(Box::new(normal)),
        ));
        let test: TestResult = res.test;
        assert_eq!(trunc_to_dec(test.expected, 1), 14.0);
        assert_eq!(trunc_to_dec(test.calculated, 1), 10.1);
    }

    fn trunc_to_dec(num: f64, dec: i32) -> f64 {
        (num * 10f64.powi(dec)).trunc() / 10f64.powi(dec)
    }
}
