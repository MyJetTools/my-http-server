#[cfg(feature = "with-telemetry")]
use my_telemetry::MyTelemetryContext;
#[cfg(feature = "with-telemetry")]
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{HttpRequest, RequestCredentials};

pub struct HttpContext {
    pub request: HttpRequest,
    #[cfg(feature = "with-telemetry")]
    pub telemetry_context: MyTelemetryContext,
    pub process_name: Option<String>,
    pub credentials: Option<Box<dyn RequestCredentials + Send + Sync + 'static>>,
}

impl HttpContext {
    pub fn new(request: HttpRequest) -> Self {
        Self {
            request,
            credentials: None,
            #[cfg(feature = "with-telemetry")]
            telemetry_context: MyTelemetryContext::Single(
                DateTimeAsMicroseconds::now().unix_microseconds,
            ),
            process_name: None,
        }
    }
}
