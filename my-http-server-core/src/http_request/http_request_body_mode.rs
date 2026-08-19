use my_http_utils::http_input::HttpBodyAsStream;

use crate::{
    next_data_frame_with_timeout, spawn_body_pump, BodyContentType, BodyExpectations,
    ContentEncoding, HttpFailResult, HttpRequestBodyContent,
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
        expectations: BodyExpectations,
    ) -> Result<&HttpRequestBodyContent, HttpFailResult> {
        match self {
            HttpRequestBody::Incoming {
                incoming,
                content_type,
            } => {
                let mut take = incoming.take().unwrap();

                let bytes = read_bytes(ContentEncoding::None, &mut take, expectations).await?;
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

    pub async fn into_http_request_body(
        self,
        expectations: BodyExpectations,
    ) -> Result<HttpRequestBodyContent, HttpFailResult> {
        match self {
            HttpRequestBody::Incoming {
                mut incoming,
                content_type,
            } => {
                let mut take = incoming.take().unwrap();
                let bytes = read_bytes(ContentEncoding::None, &mut take, expectations).await?;
                let body = HttpRequestBodyContent::new(bytes, content_type)?;
                return Ok(body);
            }
            HttpRequestBody::Full(http_request_body) => return Ok(http_request_body),
        }
    }

    /// Turns the body into a stream of chunks: creates the channel and starts the pump that fills
    /// it. `buffer` is the bounded channel's capacity — the back-pressure knob.
    pub fn into_body_stream(
        self,
        expectations: BodyExpectations,
        buffer: usize,
    ) -> HttpBodyAsStream {
        spawn_body_pump(self, expectations, buffer)
    }
}

/// Materializes a whole body through the same frame loop the stream pump uses — and holds it to
/// the same completeness rule. Both matter: a truncated upload must not reach a `#[http_body_raw]`
/// action, or a middleware that reads the body, looking like the whole thing.
async fn read_bytes(
    body_compression: ContentEncoding,
    incoming: &mut hyper::body::Incoming,
    expectations: BodyExpectations,
) -> Result<Vec<u8>, HttpFailResult> {
    let mut result: Vec<u8> = Vec::new();
    let mut delivered: u64 = 0;
    let mut end_stream = crate::EndStreamWatch::new();

    loop {
        let frame = next_data_frame_with_timeout(incoming, expectations.read_timeout).await;

        let Ok(frame) = frame else {
            return Err(HttpFailResult::from((
                400u16,
                format!(
                    "Timeout while waiting for the request body: nothing received for {:?} after {} bytes",
                    expectations.read_timeout.unwrap_or_default(),
                    delivered
                ),
            )));
        };

        let Some(chunk) = frame? else {
            break;
        };

        delivered += chunk.len() as u64;
        end_stream.sample(incoming);

        if result.is_empty() {
            // The common single-frame case moves the buffer instead of copying it.
            result = chunk.into();
        } else {
            result.extend_from_slice(&chunk);
        }
    }

    end_stream.sample(incoming);

    if let Some(reason) = expectations.incomplete_reason(delivered, end_stream.end_stream_seen()) {
        return Err(HttpFailResult::from((400u16, reason)));
    }

    body_compression.decompress_if_needed(result.into())
}
