#[cfg(feature = "my-telemetry")]
use my_telemetry::MyTelemetryContext;
#[cfg(feature = "my-telemetry")]
use my_telemetry::TelemetryEventTagsBuilder;

use crate::HttpRequest;

use crate::RequestCredentials;

pub struct HttpContext {
    pub request: HttpRequest,
    #[cfg(feature = "my-telemetry")]
    pub telemetry_context: MyTelemetryContext,
    #[cfg(feature = "my-telemetry")]
    pub telemetry_tags: TelemetryEventTagsBuilder,
    pub credentials: Option<Box<dyn RequestCredentials + Send + Sync + 'static>>,
}

impl HttpContext {
    pub fn new(request: HttpRequest) -> Self {
        Self {
            request,
            credentials: None,
            #[cfg(feature = "my-telemetry")]
            telemetry_context: MyTelemetryContext::new(),
            #[cfg(feature = "my-telemetry")]
            telemetry_tags: TelemetryEventTagsBuilder::new(),
        }
    }
}
