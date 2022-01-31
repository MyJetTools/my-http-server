use prometheus::Encoder;

use super::{my_metrics_factory, MyMetricsFactory};

pub struct MyMetrics {}

impl MyMetricsFactory for MyMetrics {
    fn create_duration_timer(&self, method: &str, path: &str) -> prometheus::HistogramTimer {
        my_metrics_factory::HTTP_RESPONSE_TIME_SECONDS
            .with_label_values(&[method, path])
            .start_timer()
    }

    fn get_my_metrics(&self) -> Vec<u8> {
        let encoder = prometheus::TextEncoder::new();

        let metric_families = prometheus::gather();
        let mut buffer = std::vec![];
        encoder.encode(&metric_families, &mut buffer).unwrap();

        buffer
    }
}
