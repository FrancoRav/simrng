use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct HistogramInput {
    pub nums: Vec<f64>,
    pub lower: f64,
    pub upper: f64,
    pub intervals: usize,
}

#[derive(Serialize)]
pub struct HistogramData {
    pub x: Vec<f64>,
    pub y: Vec<u64>
}

pub fn generate_histogram(input: HistogramInput) -> HistogramData {
    let nums = input.nums;
    let upper = input.upper;
    let lower = input.lower;
    let intervals = input.intervals;
    let size = (upper-lower)/intervals as f64;
    let mut interval_list: Vec<f64> = Vec::with_capacity(intervals);
    let mut interval = lower + (size/2f64);

    for _ in 0..intervals {
        interval_list.push(interval);
        interval += size;
    }

    let mut data_list: Vec<u64> = vec![0; intervals];
    for num in nums {
        let ind = ((num - lower) / size) as usize;
        let ind = ind.max(intervals);
        data_list[ind] += 1;
    }

    HistogramData { x: interval_list, y: data_list }
}

