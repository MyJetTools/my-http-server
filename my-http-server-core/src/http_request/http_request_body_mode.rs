use my_http_utils::http_input::HttpBodyAsStream;

use crate::{
    next_data_frame, spawn_body_pump, BodyContentType, ContentEncoding, HttpFailResult,
    HttpRequestBodyContent,
};

/// The request body, kept **lazy**: it holds hyper's `Incoming` and is not turned into bytes until
/// something actually asks for them. There are two ways to ask, and both go through the same
/// frame-by-frame primitive ([`next_data_frame`]):
///
/// * materialize it whole — [`get_http_request_body`](Self::get_http_request_body) /
///   [`into_http_request_body`](Self::into_http_request_body), used for deserialization,
///   `#[http_body_raw]` and middleware;
/// * stream it — [`into_body_stream`](Self::into_body_stream), used by a
///   `#[http_body_as_stream]` model.
pub enum HttpRequestBody {
    Incoming {
        incoming: Option<hyper::body::Incoming>,
        content_type: BodyContentType,
    },
    Full(HttpRequestBodyContent),
}

impl HttpRequestBody {
    pub async fn get_http_request_body(
        &mut self,
    ) -> Result<&HttpRequestBodyContent, HttpFailResult> {
        match self {
            HttpRequestBody::Incoming {
                incoming,
                content_type,
            } => {
                let mut take = incoming.take().unwrap();

                let bytes = read_bytes(ContentEncoding::None, &mut take).await?;
                let body = HttpRequestBodyContent::new(bytes, content_type.clone())?;
                *self = HttpRequestBody::Full(body);
            }
            HttpRequestBody::Full(http_request_body) => return Ok(http_request_body),
        }

        match self {
            HttpRequestBody::Incoming { .. } => {
                panic!("We should never be here")
            }
            HttpRequestBody::Full(http_request_body) => Ok(http_request_body),
        }
    }

    pub async fn into_http_request_body(self) -> Result<HttpRequestBodyContent, HttpFailResult> {
        match self {
            HttpRequestBody::Incoming {
                mut incoming,
                content_type,
            } => {
                let mut take = incoming.take().unwrap();
                let bytes = read_bytes(ContentEncoding::None, &mut take).await?;
                let body = HttpRequestBodyContent::new(bytes, content_type)?;
                return Ok(body);
            }
            HttpRequestBody::Full(http_request_body) => return Ok(http_request_body),
        }
    }

    /// Turns the body into a stream of chunks: creates the channel and starts the pump that fills
    /// it. `content_length` is the `Content-Length` header when present (`None` for a chunked
    /// body); `buffer` is the bounded channel's capacity — the back-pressure knob.
    pub fn into_body_stream(self, content_length: Option<u64>, buffer: usize) -> HttpBodyAsStream {
        spawn_body_pump(self, content_length, buffer)
    }
}

/// Materializes a whole body through the same frame loop the stream pump uses, so the two paths
/// cannot drift apart in how they take a body off the wire.
async fn read_bytes(
    body_compression: ContentEncoding,
    incoming: &mut hyper::body::Incoming,
) -> Result<Vec<u8>, HttpFailResult> {
    let mut result: Vec<u8> = Vec::new();

    while let Some(chunk) = next_data_frame(incoming).await? {
        if result.is_empty() {
            // The common single-frame case moves the buffer instead of copying it.
            result = chunk.into();
        } else {
            result.extend_from_slice(&chunk);
        }
    }

    body_compression.decompress_if_needed(result.into())
}
