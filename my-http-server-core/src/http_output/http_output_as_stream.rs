use bytes::Bytes;
use http_body_util::StreamBody;

use crate::{HttpFailResult, HttpOkResult};

pub struct HttpOutputAsStream {
    rx: futures::channel::mpsc::Receiver<Result<hyper::body::Frame<Bytes>, hyper::Error>>,
}

impl HttpOutputAsStream {
    pub fn new(
        rx: futures::channel::mpsc::Receiver<Result<hyper::body::Frame<Bytes>, hyper::Error>>,
    ) -> Self {
        Self { rx }
    }
    pub fn get_result(self) -> Result<HttpOkResult, HttpFailResult> {
        use http_body_util::BodyExt;

        let rx = self.rx;

        //   let stream_body = StreamBody::new(rx);
        let stream_body = StreamBody::new(rx);
        let boxed_body = stream_body.map_err(|e: hyper::Error| e.to_string()).boxed();

        let response = hyper::Response::builder().body(boxed_body).unwrap();

        Ok(HttpOkResult {
            write_telemetry: false,
            output: super::HttpOutput::Raw(response),
        })
    }
}
