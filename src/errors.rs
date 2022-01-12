use crate::{url_decoder::UrlDecodeError, HttpFailResult, WebContentType};
use hyper::{Body, Response};

impl Into<HttpFailResult> for hyper::Error {
    fn into(self) -> HttpFailResult {
        HttpFailResult {
            content_type: WebContentType::Text,
            status_code: 501,
            content: format!("{:?}", self).into_bytes(),
            write_telemetry: true,
        }
    }
}

impl Into<Response<Body>> for HttpFailResult {
    fn into(self) -> Response<Body> {
        Response::builder()
            .header("Content-Type", self.content_type.to_string())
            .status(self.status_code)
            .body(Body::from(self.content))
            .unwrap()
    }
}

impl From<UrlDecodeError> for HttpFailResult {
    fn from(src: UrlDecodeError) -> Self {
        Self {
            status_code: 501,
            content_type: WebContentType::Text,
            content: format!("UrlDecodeError: {}", src.msg).into_bytes(),
            write_telemetry: true,
        }
    }
}
