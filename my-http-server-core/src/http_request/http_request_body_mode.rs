use my_http_utils::http_input::HttpBodyAsStream;

use crate::{
    next_data_frame_with_timeout, spawn_body_pump, BodyContentType, BodyEncryptionKeys,
    BodyExpectations, HttpFailResult, HttpRequestBodyContent,
};

use super::body_decoding::{BodyDecoding, DecodedBody};

/// The request body, kept **lazy**: it holds hyper's `Incoming` and is not turned into bytes until
/// something actually asks for them. There are three ways to ask, and all of them go through the
/// same frame-by-frame primitive ([`next_data_frame`](crate::next_data_frame)):
///
/// * materialize it whole — [`get_http_request_body`](Self::get_http_request_body) /
///   [`into_http_request_body`](Self::into_http_request_body), used for `#[http_body_raw]`, for
///   a body that announced a `Content-Encoding`, and by middleware;
/// * read it as it arrives — [`crate::IncomingBodyStream`], which the named body fields of a
///   `#[http_body]` / `#[http_form_data]` model are parsed out of;
/// * stream it — [`into_body_stream`](Self::into_body_stream), used by a
///   `#[http_body_as_stream]` model.
///
/// A body that announced a `Content-Encoding` is decoded on the first path, and only there -
/// decompressed, or decrypted with the server's [`BodyEncryptionKeys`] (`aes128gcm`). A streamed
/// body is handed on exactly as it arrived, which is what a pass-through action wants and the only
/// thing this crate can do frame by frame.
pub enum HttpRequestBody {
    Incoming {
        incoming: Option<hyper::body::Incoming>,
        content_type: BodyContentType,
        /// The `Content-Encoding` the request announced, verbatim. It is parsed when the body is
        /// materialized rather than when the request arrives: a streamed body is passed through
        /// exactly as it came, and a request whose body is never read must not be rejected for an
        /// encoding nobody was going to undo.
        content_encoding: Option<String>,
    },
    Full(HttpRequestBodyContent),
}

impl HttpRequestBody {
    /// `encryption_keys` open a body announced with `Content-Encoding: aes128gcm`; see
    /// [`MyHttpServer::set_body_encryption_keys`](crate::MyHttpServer::set_body_encryption_keys).
    pub async fn get_http_request_body(
        &mut self,
        expectations: BodyExpectations,
        encryption_keys: Option<&BodyEncryptionKeys>,
    ) -> Result<&HttpRequestBodyContent, HttpFailResult> {
        match self {
            HttpRequestBody::Incoming {
                incoming,
                content_type,
                content_encoding,
            } => {
                // Gone only if an earlier read of the body failed half-way: reading it again
                // is a bug of the handler.
                let mut take = incoming.take().expect(
                    "Body is taken by some middleware before: an earlier read of it failed",
                );
                let decoding = BodyDecoding::new(content_encoding.as_deref(), encryption_keys)?;

                let decoded = read_bytes(decoding, &mut take, expectations).await?;
                let mut body = HttpRequestBodyContent::new(decoded.bytes, content_type.clone())?;
                body.set_encryption_info(decoded.encryption_info);
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

    /// Same as [`get_http_request_body`](Self::get_http_request_body), taking the body.
    pub async fn into_http_request_body(
        self,
        expectations: BodyExpectations,
        encryption_keys: Option<&BodyEncryptionKeys>,
    ) -> Result<HttpRequestBodyContent, HttpFailResult> {
        match self {
            HttpRequestBody::Incoming {
                mut incoming,
                content_type,
                content_encoding,
            } => {
                // Gone only if an earlier read of the body failed half-way: reading it again
                // is a bug of the handler.
                let mut take = incoming.take().expect(
                    "Body is taken by some middleware before: an earlier read of it failed",
                );
                let decoding = BodyDecoding::new(content_encoding.as_deref(), encryption_keys)?;
                let decoded = read_bytes(decoding, &mut take, expectations).await?;
                let mut body = HttpRequestBodyContent::new(decoded.bytes, content_type)?;
                body.set_encryption_info(decoded.encryption_info);
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
///
/// Decoding comes last, after the completeness check: `Content-Length` counts the bytes on the
/// wire, which are the compressed or encrypted ones.
async fn read_bytes(
    decoding: BodyDecoding,
    incoming: &mut hyper::body::Incoming,
    expectations: BodyExpectations,
) -> Result<DecodedBody, HttpFailResult> {
    let mut result: Vec<u8> = Vec::new();
    let mut delivered: u64 = 0;
    let mut end_stream = crate::EndStreamWatch::new();

    loop {
        let frame = next_data_frame_with_timeout(incoming, expectations.read_timeout).await;

        let Ok(frame) = frame else {
            return Err(HttpFailResult::from((
                400u16,
                expectations.timeout_reason(delivered),
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

    // The overwhelming majority of bodies announce no encoding: they are handed on right here,
    // with no trip to the blocking pool.
    if decoding.is_none() {
        return Ok(DecodedBody::plain(result));
    }

    // Decompressing is CPU work, and with a limit in the tens of megabytes it is hundreds of
    // milliseconds of it - enough to stall every other task parked on this tokio worker.
    // Decrypting a large body is too. So it runs on the blocking pool, and this task just waits
    // for it.
    let max_decompressed_size = expectations.max_decompressed_body_size;

    tokio::task::spawn_blocking(move || decoding.decode(result, max_decompressed_size))
        .await
        .map_err(|err| {
            HttpFailResult::as_fatal_error(format!(
                "Decoding the request body did not finish: {}",
                err
            ))
        })?
}
