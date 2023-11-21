#[cfg(feature = "with-telemetry")]
use my_telemetry::MyTelemetryContext;

use crate::{HttpRequest, RequestCredentials};

pub struct HttpContext {
    pub request: HttpRequest,
    #[cfg(feature = "with-telemetry")]
    pub telemetry_context: MyTelemetryContext,
    pub credentials: Option<Box<dyn RequestCredentials + Send + Sync + 'static>>,
}

impl HttpContext {
    pub fn new(request: HttpRequest) -> Self {
        Self {
            request,
            credentials: None,
            #[cfg(feature = "with-telemetry")]
            telemetry_context: MyTelemetryContext::new(),
        }
    }
}
