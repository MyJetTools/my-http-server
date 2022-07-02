use my_telemetry::MyTelemetryContext;

use crate::HttpRequest;

pub struct HttpContext {
    pub request: HttpRequest,
    pub telemetry_context: MyTelemetryContext,
}

impl HttpContext {
    pub fn new(request: HttpRequest) -> Self {
        Self {
            request,
            telemetry_context: MyTelemetryContext::new(),
        }
    }
}
