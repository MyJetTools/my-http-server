use crate::{HttpFailResult, WebContentType};
use hyper::{body::Bytes, Response};

impl From<hyper::Error> for HttpFailResult {
    fn from(src: hyper::Error) -> Self {
        HttpFailResult {
            content_type: WebContentType::Text,
            status_code: 501,
            content: format!("{:?}", src).into_bytes(),
            write_telemetry: true,
            write_to_log: true,
            headers: Default::default(),
            #[cfg(feature = "with-telemetry")]
            add_telemetry_tags: my_telemetry::TelemetryEventTagsBuilder::new(),
        }
    }
}

impl Into<Response<Bytes>> for HttpFailResult {
    fn into(self) -> Response<Bytes> {
        Response::builder()
            .header("Content-Type", self.content_type.as_str())
            .status(self.status_code)
            .body(Bytes::from(self.content))
            .unwrap()
    }
}

impl From<std::string::FromUtf8Error> for HttpFailResult {
    fn from(src: std::string::FromUtf8Error) -> Self {
        Self::as_fatal_error(format!("{}", src))
    }
}
