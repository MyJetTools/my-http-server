#[cfg(feature = "my-telemetry")]
use my_telemetry::TelemetryEvent;
#[cfg(feature = "my-telemetry")]
use rust_extensions::date_time::DateTimeAsMicroseconds;

use std::sync::Arc;

use crate::HttpServerMiddleware;

pub struct HttpServerData {
    pub middlewares: Vec<Arc<dyn HttpServerMiddleware + Send + Sync + 'static>>,
}
