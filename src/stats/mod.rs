use axum::Json;

pub struct Response {
    x: Vec<f64>,
    y: Vec<u64>
}

pub fn generate_histogram(nums: Vec<f64>, lower: f64, upper: f64, intervals: usize) -> Json<Response> {
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

    Json(Response { x: interval_list, y: data_list })
}


