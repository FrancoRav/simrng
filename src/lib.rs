pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub mod rng;
pub mod dist;
pub mod stats;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
