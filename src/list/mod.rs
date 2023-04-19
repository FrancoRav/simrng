use std::sync::Arc;

pub fn get_page(nums: Arc<Vec<f64>>, pagenum: usize) -> Vec<f64> {
    let page_size = 30;
    let start: usize = page_size * (pagenum-1);
    let end = (start + page_size).min(nums.len());
    match nums.get(start..end) {
        Some(nums) => nums.to_vec(),
        None => {
            vec![]
        }
    }
}
