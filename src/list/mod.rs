use std::sync::Arc;

pub fn get_page(nums: Arc<Vec<f64>>, pagenum: usize) -> Vec<f64> {
    let page_size = 30;
    let start: usize = page_size * pagenum;
    let end = (start + page_size).min(nums.len());
    nums.get(start..end).unwrap().to_vec()
}
