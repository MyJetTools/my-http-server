use crate::{HttpFailResult, WebContentType};
use hyper::{Body, Response};

impl From<hyper::Error> for HttpFailResult {
    fn from(src: hyper::Error) -> Self {
        HttpFailResult {
            content_type: WebContentType::Text,
            status_code: 501,
            content: format!("{:?}", src).into_bytes(),
            write_telemetry: true,
        }
    }
}

impl Into<Response<Body>> for HttpFailResult {
    fn into(self) -> Response<Body> {
        Response::builder()
            .header("Content-Type", self.content_type.as_str())
            .status(self.status_code)
            .body(Body::from(self.content))
            .unwrap()
    }
}

impl From<std::string::FromUtf8Error> for HttpFailResult {
    fn from(src: std::string::FromUtf8Error) -> Self {
        Self::as_fatal_error(format!("{}", src))
    }
}
