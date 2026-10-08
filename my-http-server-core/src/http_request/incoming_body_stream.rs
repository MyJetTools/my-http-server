//! The request body as a [`rust_extensions::AsyncBytesStream`], read straight off hyper.
//!
//! This is what the derive-generated `parse_with_body_stream` reads a model's named body fields
//! (`#[http_body]` / `#[http_form_data]`) out of. A JSON body is taken apart member by member as
//! it arrives, so it is never put together in memory whole. There is no channel and no pump: the
//! parser asks for a frame when it needs one, so the back-pressure is the parser's own.

use my_http_utils::http_input::HttpParseError;
use rust_extensions::AsyncBytesStream;

use crate::{next_data_frame_with_timeout, BodyExpectations, BodyReadTimeout, EndStreamWatch};

/// The DATA frames of a hyper body, held to the same completeness rule and the same idle timeout
/// as the other two ways of reading a body ([the pump](crate::spawn_body_pump) and
/// [read-it-whole](crate::HttpRequestBody)).
///
/// `Ok(None)` comes only out of a body that arrived in full. A body that was cut short, timed
/// out or broke in the transport is an [`HttpParseError::BodyStream`], so it is answered `400`
/// and never reaches an action looking like the whole thing.
///
/// The bytes are handed on exactly as they came, so a body that announced a `Content-Encoding`
/// is not read this way. [`HttpRequest::take_incoming_body_stream`](crate::HttpRequest::take_incoming_body_stream)
/// leaves such a body to the materialize path, which decodes it.
pub struct IncomingBodyStream {
    // `get_next` takes `&self`, and reading a frame needs `&mut Incoming` across an `.await`
    state: tokio::sync::Mutex<IncomingBodyState>,
    expectations: BodyExpectations,
}

struct IncomingBodyState {
    incoming: hyper::body::Incoming,
    delivered: u64,
    end_stream: EndStreamWatch,
    /// The body was read to its end and was complete. The stream is not asked again.
    ended: bool,
}

impl IncomingBodyStream {
    pub fn new(incoming: hyper::body::Incoming, expectations: BodyExpectations) -> Self {
        Self {
            state: tokio::sync::Mutex::new(IncomingBodyState {
                incoming,
                delivered: 0,
                end_stream: EndStreamWatch::new(),
                ended: false,
            }),
            expectations,
        }
    }

    /// Reads what is left of the body and lets it go — the bytes are not kept.
    ///
    /// A parse stops reading once it has every field the model reads, and what comes after them
    /// still has to come off the wire: until the body has ended there is no telling it was
    /// complete, and an HTTP/1 connection with a body left unread on it is not used again.
    pub async fn drain(&self) -> Result<(), HttpParseError> {
        while self.get_next().await?.is_some() {}
        Ok(())
    }
}

impl IncomingBodyState {
    async fn next_chunk(
        &mut self,
        expectations: &BodyExpectations,
    ) -> Result<Option<bytes::Bytes>, HttpParseError> {
        if self.ended {
            return Ok(None);
        }

        let frame = next_data_frame_with_timeout(&mut self.incoming, expectations.read_timeout)
            .await
            .map_err(|BodyReadTimeout| {
                HttpParseError::BodyStream(expectations.timeout_reason(self.delivered))
            })?;

        match frame {
            Ok(Some(chunk)) => {
                self.delivered += chunk.len() as u64;
                self.end_stream.sample(&self.incoming);
                Ok(Some(chunk))
            }
            Ok(None) => {
                // The body ended - but was it complete? See `BodyExpectations`.
                self.end_stream.sample(&self.incoming);

                if let Some(reason) = expectations
                    .incomplete_reason(self.delivered, self.end_stream.end_stream_seen())
                {
                    return Err(HttpParseError::BodyStream(reason));
                }

                self.ended = true;
                Ok(None)
            }
            Err(err) => Err(HttpParseError::BodyStream(format!(
                "Can not read request body chunk: {}",
                err
            ))),
        }
    }
}

#[async_trait::async_trait]
impl AsyncBytesStream<HttpParseError> for IncomingBodyStream {
    type Chunk = bytes::Bytes;

    async fn get_next(&self) -> Result<Option<bytes::Bytes>, HttpParseError> {
        let mut state = self.state.lock().await;
        state.next_chunk(&self.expectations).await
    }

    /// The `Content-Length` the client announced.
    fn get_size(&self) -> Option<usize> {
        self.expectations.content_length.map(|size| size as usize)
    }

    async fn into_vec(&self) -> Result<Vec<u8>, HttpParseError> {
        // Not the default of the trait: that one allocates the announced size up front, and the
        // announced size is whatever the client wrote into `Content-Length`
        let mut result = Vec::new();

        while let Some(chunk) = self.get_next().await? {
            result.extend_from_slice(&chunk);
        }

        Ok(result)
    }
}
