use lazy_static::lazy_static;
use prometheus::{register_histogram_vec, HistogramTimer, HistogramVec};

lazy_static! {
    pub static ref HTTP_RESPONSE_TIME_SECONDS: HistogramVec = register_histogram_vec!(
        "http_response_time_seconds",
        "HTTP response times",
        &["method", "path"]
    )
    .unwrap();
}

pub trait MyMetricsFactory {
    fn create_duration_timer(&self, method: &str, path: &str) -> HistogramTimer;

    fn get_my_metrics(&self) -> Vec<u8>;
}
