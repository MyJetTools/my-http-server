use bytes::Bytes;
use http_body_util::StreamBody;
use rust_extensions::StrOrString;

use crate::{CompressionLevel, HttpFailResult, HttpOkResult, HttpRequest};

use super::stream_compression::{accepts_gzip, GzipFrames};

/// What [`HttpOutputAsStream::with_compression`] settled on.
#[derive(Clone, Copy)]
enum StreamCompression {
    NotRequested,
    /// `gzip` is the level to compress with, or `None` when the client does not take gzip.
    Negotiated {
        gzip: Option<CompressionLevel>,
    },
}

pub struct HttpOutputAsStream {
    rx: futures::channel::mpsc::Receiver<Result<hyper::body::Frame<Bytes>, hyper::Error>>,
    headers: Vec<(String, String)>,
    compression: StreamCompression,
}

impl HttpOutputAsStream {
    pub fn new(
        rx: futures::channel::mpsc::Receiver<Result<hyper::body::Frame<Bytes>, hyper::Error>>,
    ) -> Self {
        Self {
            rx,
            headers: Vec::new(),
            compression: StreamCompression::NotRequested,
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

    /// Gzips the stream on the fly when the client's `Accept-Encoding` takes gzip; otherwise it
    /// goes out as produced. The producer does not change: it keeps sending plain bytes, and
    /// every chunk is compressed and flushed as it passes, so the client can decode each one as
    /// soon as it arrives. The gzip footer is written when the producer is dropped.
    ///
    /// `level` trades the CPU spent on every chunk against the bytes on the wire.
    ///
    /// There is no size threshold and no "was it worth it" check, unlike
    /// [`HttpResultBuilder::with_compression`](crate::HttpResultBuilder::with_compression): the
    /// headers leave before the first chunk exists. Every flush costs a few bytes, so a stream of
    /// tiny chunks (a heartbeat of a few bytes) can come out larger than it went in. Chunks are
    /// compressed on the connection's task, so keep them reasonably sized - up to tens of KiB.
    ///
    /// The response gets `Vary: Accept-Encoding`, and a `Content-Length` set with
    /// [`with_header`](Self::with_header) is dropped once the stream is compressed - it counts
    /// the bytes produced, not the ones on the wire. A stream that already carries its own
    /// `Content-Encoding` is left exactly as it is.
    pub fn with_compression(mut self, request: &HttpRequest, level: CompressionLevel) -> Self {
        let accept_encoding = request
            .data
            .headers()
            .get_all(http::header::ACCEPT_ENCODING)
            .iter()
            .filter_map(|value| value.to_str().ok());

        self.compression = StreamCompression::Negotiated {
            gzip: accepts_gzip(accept_encoding).then_some(level),
        };
        self
    }

    pub fn get_result(self) -> Result<HttpOkResult, HttpFailResult> {
        use http_body_util::BodyExt;

        let mut headers = self.headers;
        let mut gzip = None;

        if let StreamCompression::Negotiated { gzip: accepted } = self.compression {
            // The action encoded the stream itself: it is the one describing it.
            if !has_header(&headers, "content-encoding") {
                add_vary_accept_encoding(&mut headers);

                if accepted.is_some() {
                    headers.retain(|(key, _)| !key.eq_ignore_ascii_case("content-length"));
                    headers.push(("Content-Encoding".to_string(), "gzip".to_string()));
                    gzip = accepted;
                }
            }
        }

        let boxed_body = match gzip {
            Some(level) => StreamBody::new(GzipFrames::new(self.rx, level)).boxed(),
            None => StreamBody::new(self.rx)
                .map_err(|e: hyper::Error| e.to_string())
                .boxed(),
        };

        let mut builder = hyper::Response::builder();

        for header in headers {
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

fn has_header(headers: &[(String, String)], name: &str) -> bool {
    headers
        .iter()
        .any(|(key, _)| key.eq_ignore_ascii_case(name))
}

/// The response differs by `Accept-Encoding` whichever way the negotiation went, and a cache has
/// to know it - unless the action already said so, or said `Vary: *`.
fn add_vary_accept_encoding(headers: &mut Vec<(String, String)>) {
    let already_varies = headers
        .iter()
        .filter(|(key, _)| key.eq_ignore_ascii_case("vary"))
        .flat_map(|(_, value)| value.split(','))
        .any(|field| {
            let field = field.trim();
            field == "*" || field.eq_ignore_ascii_case("accept-encoding")
        });

    if !already_varies {
        headers.push(("Vary".to_string(), "Accept-Encoding".to_string()));
    }
}
