use async_trait::async_trait;
use my_http_server_core::{
    HttpContext, HttpFailResult, HttpOkResult, HttpOutput, HttpServerMiddleware,
    HttpServerRequestFlow,
};
use stopwatch::Stopwatch;

pub struct MetricsMiddleware;

impl MetricsMiddleware {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl HttpServerMiddleware for MetricsMiddleware {
    async fn handle_request(
        &self,
        ctx: &mut HttpContext,
        get_next: &mut HttpServerRequestFlow,
    ) -> Result<HttpOkResult, HttpFailResult> {
        let path = ctx.request.http_path.as_str().to_string();

        if path == "/metrics" {
            let report = prometheus::TextEncoder::new()
                .encode_to_string(&prometheus::default_registry().gather());

            match report {
                Ok(report) => {
                    let response = HttpOutput::as_text(report).into_ok_result(false);
                    return response;
                }
                Err(err) => {
                    let response =
                        HttpOutput::as_text(err.to_string()).into_fail_result(502, false);
                    return response;
                }
            }
        }

        let mut sw = Stopwatch::start_new();
        let result = get_next.next(ctx).await;
        sw.stop();

        let duration = sw.elapsed();
        let method = ctx.request.method.as_str().to_string();
        let path = ctx.request.http_path.as_str().to_string();
        let common_labels = &[("method", method.clone()), ("path", path.clone())];

        let mut has_to_write_metrics = false;

        if let Err(result) = &result {
            if result.status_code == 404 {
                has_to_write_metrics = true
            } else {
                let failed_labels = &[
                    ("method", method.clone()),
                    ("path", path.clone()),
                    ("status_code", result.status_code.to_string()),
                ];

                metrics::counter!("failed_request_count", failed_labels).increment(1);
                metrics::counter!("failed_request_duration_sum", failed_labels).increment(duration.as_millis() as u64);
                metrics::histogram!("failed_request_duration_nanos", failed_labels).record(duration.as_millis() as f64);
            }
        }

        if has_to_write_metrics {
            return result;
        }

        metrics::histogram!("request_duration_nanos", common_labels).record(duration.as_millis() as f64);
        metrics::counter!("request_duration_sum", common_labels).increment(duration.as_millis() as u64);
        metrics::counter!("request_count", common_labels).increment(1);

        return result;
    }
}
