use crate::*;

pub struct HttpOkResult {
    pub write_telemetry: bool,
    #[cfg(feature = "with-telemetry")]
    pub add_telemetry_tags: my_telemetry::TelemetryEventTagsBuilder,
    pub output: HttpOutput,
}

impl HttpOkResult {
    pub fn get_status_code(&self) -> u16 {
        self.output.get_status_code()
    }
}

impl Into<HttpOkResult> for String {
    fn into(self) -> HttpOkResult {
        HttpOkResult {
            write_telemetry: true,
            #[cfg(feature = "with-telemetry")]
            add_telemetry_tags: my_telemetry::TelemetryEventTagsBuilder::new(),
            output: HttpOutput::Content {
                status_code: 200,
                headers: None,
                content_type: Some(WebContentType::Text),
                content: self.into_bytes(),
                set_cookies: None,
            },
        }
    }
}

impl Into<Result<HttpOkResult, HttpFailResult>> for HttpOkResult {
    fn into(self) -> Result<HttpOkResult, HttpFailResult> {
        Ok(self)
    }
}
