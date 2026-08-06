//! Filling a [`HttpBodyAsStream`] out of hyper.
//!
//! my-http-utils owns the channel and the reading half and knows nothing about transports; the
//! whole "where do the bytes come from" half lives here: pull DATA frames off
//! `hyper::body::Incoming` and pour them into the sending half.
//!
//! [`next_data_frame`] is *the* body-reading primitive of this crate — both the pump below and the
//! materialize-it-whole path ([`crate::HttpRequestBody::get_http_request_body`]) go through it, so
//! there is exactly one place that knows how a body is taken apart. It works the same for
//! `Transfer-Encoding: chunked` and for a `Content-Length` body: hyper decodes both into DATA
//! frames, and the only difference is whether the length was known up front.

use my_http_utils::http_input::{HttpBodyAsStream, HttpBodyStreamSender, HttpParseError};

use crate::HttpRequestBody;

/// The next DATA frame of a hyper body. `Ok(None)` — the body is over.
///
/// TRAILERS frames are skipped (they carry headers, not body bytes), and so are empty DATA frames:
/// an empty chunk is not an end-of-body marker, and letting one through would make
/// `Ok(Some(vec![]))` look like "almost done" to a consumer.
pub async fn next_data_frame(
    incoming: &mut hyper::body::Incoming,
) -> Result<Option<bytes::Bytes>, hyper::Error> {
    use http_body_util::BodyExt;

    loop {
        let Some(frame) = incoming.frame().await else {
            return Ok(None);
        };

        match frame?.into_data() {
            Ok(data) => {
                if data.is_empty() {
                    continue;
                }

                return Ok(Some(data));
            }
            // Trailers — no body bytes in it, keep going.
            Err(_) => continue,
        }
    }
}

/// Creates the channel and starts the background pump that fills it.
///
/// The channel is **bounded**, so the pump reads at most `buffer` chunks ahead and then parks on
/// `send().await`. That is what keeps a fast uploader from eating memory: the pressure propagates
/// back through hyper to the TCP window. It also makes the eager start harmless — a pump nobody
/// reads from stops after `buffer` chunks instead of draining the whole body.
pub fn spawn_body_pump(
    body: HttpRequestBody,
    content_length: Option<u64>,
    buffer: usize,
) -> HttpBodyAsStream {
    let (sender, stream) = HttpBodyAsStream::create(buffer, content_length);

    tokio::spawn(pump(body, sender));

    stream
}

async fn pump(body: HttpRequestBody, sender: HttpBodyStreamSender) {
    match body {
        HttpRequestBody::Incoming { mut incoming, .. } => {
            let Some(mut incoming) = incoming.take() else {
                sender
                    .send_error(HttpParseError::BodyStream(
                        "Request body was already consumed".to_string(),
                    ))
                    .await;
                return;
            };

            loop {
                let frame = tokio::select! {
                    // Checked first, on purpose. Without this arm a pump waiting on a client that
                    // went quiet would never learn that the handler walked away (returned 403
                    // without draining, say) and would hang on that body forever, holding the
                    // connection with it. `send_chunk() -> false` only reports it on the *next*
                    // chunk, which may never come.
                    biased;

                    _ = sender.closed() => return,

                    frame = next_data_frame(&mut incoming) => frame,
                };

                match frame {
                    Ok(Some(data)) => {
                        if !sender.send_chunk(data.into()).await {
                            // Reader is gone.
                            return;
                        }
                    }
                    Ok(None) => {
                        // The ONLY place `finish` may be called: every other exit out of this
                        // loop leaves the flag unset, which is what turns a half-delivered body
                        // into an error on the reading side instead of a silent truncation.
                        sender.finish();
                        return;
                    }
                    Err(err) => {
                        sender
                            .send_error(HttpParseError::BodyStream(format!(
                                "Can not read request body chunk: {}",
                                err
                            )))
                            .await;
                        return;
                    }
                }
            }
        }
        // Some middleware materialized the body before the action ran. Streaming still has to work
        // — hand it over as a single chunk rather than blowing up.
        HttpRequestBody::Full(content) => {
            let bytes = content.get_body();

            if !bytes.is_empty() && !sender.send_chunk(bytes).await {
                return;
            }

            sender.finish();
        }
    }
}
