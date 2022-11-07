#[cfg(feature = "my-telemetry")]
use my_telemetry::TelemetryEvent;
#[cfg(feature = "my-telemetry")]
use rust_extensions::date_time::DateTimeAsMicroseconds;

use std::sync::Arc;

use crate::HttpServerMiddleware;

use crate::RequestCredentials;

pub struct HttpServerData<TRequestCredentials: RequestCredentials + Send + Sync + 'static> {
    pub middlewares: Vec<
        Arc<
            dyn HttpServerMiddleware<TRequestCredentials = TRequestCredentials>
                + Send
                + Sync
                + 'static,
        >,
    >,
}
