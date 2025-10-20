use bytes::Bytes;
use http_body_util::StreamBody;
use rust_extensions::StrOrString;

use crate::{HttpFailResult, HttpOkResult};

pub struct HttpOutputAsStream {
    rx: futures::channel::mpsc::Receiver<Result<hyper::body::Frame<Bytes>, hyper::Error>>,
    headers: Vec<(String, String)>,
}

impl HttpOutputAsStream {
    pub fn new(
        rx: futures::channel::mpsc::Receiver<Result<hyper::body::Frame<Bytes>, hyper::Error>>,
    ) -> Self {
        Self {
            rx,
            headers: Vec::new(),
        }
    }

    pub fn with_header<'k, 'v>(
        mut self,
        key: impl Into<StrOrString<'k>>,
        value: impl Into<StrOrString<'v>>,
    ) -> Self {
        let key = key.into();
        let value = value.into();

        self.headers.push((key.to_string(), value.to_string()));
        self
    }
    pub fn get_result(self) -> Result<HttpOkResult, HttpFailResult> {
        use http_body_util::BodyExt;

        let rx = self.rx;

        //   let stream_body = StreamBody::new(rx);
        let stream_body = StreamBody::new(rx);
        let boxed_body = stream_body.map_err(|e: hyper::Error| e.to_string()).boxed();

        let mut builder = hyper::Response::builder();

        for header in self.headers {
            builder = builder.header(header.0, header.1);
        }

        let response = builder.body(boxed_body).unwrap();

        Ok(HttpOkResult {
            write_telemetry: false,
            #[cfg(feature = "with-telemetry")]
            add_telemetry_tags: my_telemetry::TelemetryEventTagsBuilder::new(),
            output: super::HttpOutput::Raw(response),
        })
    }
}
