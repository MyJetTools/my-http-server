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

/// What the request promised about its body, and how to tell whether it kept the promise.
///
/// "The body ended" is not the same as "the body is complete", and the difference can not be
/// delegated to the transport. HTTP/1 truncation does surface as a hyper error, but an HTTP/2
/// stream the client aborts with `RST_STREAM(NO_ERROR)` arrives as a perfectly ordinary end of
/// body — so a handler would be handed a half-received upload labelled complete. Both ways of
/// reading a body ([the pump](spawn_body_pump) and [read-it-whole](crate::HttpRequestBody)) check
/// this, and they check it the same way.
#[derive(Clone, Copy, Debug, Default)]
pub struct BodyExpectations {
    pub version: hyper::Version,
    /// The `Content-Length` the client announced, when it announced one.
    pub content_length: Option<u64>,
    /// How long to wait for the *next* piece of the body before giving up — an **idle** timeout,
    /// not a deadline for the whole upload, so a large but progressing transfer is never cut off.
    ///
    /// It covers only the wait for the client. Time spent parked because the handler is slow to
    /// consume (a full channel) does not count against it: that is back-pressure working, not a
    /// stalled client.
    ///
    /// `None` (the default) waits forever, which is the behaviour this server has always had.
    /// Set it via `MyHttpServer::set_body_read_timeout` — a client that opens a connection,
    /// announces a large body and then goes silent otherwise holds a pump and a connection
    /// indefinitely.
    pub read_timeout: Option<std::time::Duration>,
}

impl BodyExpectations {
    /// Why the body that just ended is NOT complete, or `None` when it is.
    ///
    /// `end_stream_seen` is hyper's own `Body::is_end_stream`, **latched** — sampled after every
    /// frame and kept once it has been true, see [`EndStreamWatch`].
    pub fn incomplete_reason(&self, delivered: u64, end_stream_seen: bool) -> Option<String> {
        // The strongest signal, and transport-independent: the client said how much it would send.
        if let Some(announced) = self.content_length {
            if delivered != announced {
                return Some(format!(
                    "Request body delivered {} bytes out of the {} announced",
                    delivered, announced
                ));
            }

            return None;
        }

        // No announced length. On HTTP/2 that is the *normal* case for a streamed upload — h2 has
        // no chunked encoding, so a body of unknown size simply omits the header — and it is
        // exactly where an aborted upload would otherwise pass for a complete one. h2 does know
        // the difference: END_STREAM is what marks a body finished, and a stream that ended
        // without it was cut short.
        //
        // Deliberately gated on HTTP/2: a *completed* HTTP/1 chunked body also reports
        // `is_end_stream == false` (hyper keeps its length as CHUNKED), so checking it there would
        // reject every chunked upload. HTTP/1 truncation is caught by hyper itself, as an error.
        //
        // The wording is careful because this is the one branch that can be wrong. A client may
        // send END_STREAM and then immediately RST_STREAM(NO_ERROR) to cancel a request it is no
        // longer waiting for; h2 overwrites the state that recorded END_STREAM with the reset, so
        // if that happens before we drain the last frame, a complete body is indistinguishable
        // from a truncated one. Latching (see [`EndStreamWatch`]) closes that window whenever we
        // observed END_STREAM first, and what remains is answered the only safe way: a body that
        // *may* be incomplete is not passed off as complete.
        if self.version == hyper::Version::HTTP_2 && !end_stream_seen {
            return Some(
                "HTTP/2 stream was reset before END_STREAM was observed — the request body may be incomplete"
                    .to_string(),
            );
        }

        None
    }
}

/// Watches hyper's `Body::is_end_stream` across a body read and **remembers** it once true.
///
/// Sampling it only after the body ended is not enough. END_STREAM is a fact about a frame that
/// already arrived, but the flag reporting it is derived from live stream state that a later
/// `RST_STREAM` erases (h2 overwrites `HalfClosedRemote` with `Closed(Cause::Error(..))`). Reading
/// it after every frame catches the truth while it is still there.
///
/// Latching can only ever turn a rejection into an acceptance, and only when END_STREAM really was
/// observed — which is precisely the statement "the client finished sending the body" — so it can
/// not let a truncated body through.
pub struct EndStreamWatch(bool);

impl EndStreamWatch {
    pub fn new() -> Self {
        Self(false)
    }

    /// Sample the flag. Call after every frame, and once more when the body ends.
    pub fn sample(&mut self, incoming: &hyper::body::Incoming) {
        self.0 |= hyper::body::Body::is_end_stream(incoming);
    }

    pub fn end_stream_seen(&self) -> bool {
        self.0
    }
}

impl Default for EndStreamWatch {
    fn default() -> Self {
        Self::new()
    }
}

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

/// Raised instead of a frame when the client went quiet for longer than the idle timeout.
pub struct BodyReadTimeout;

/// [`next_data_frame`] under an idle timeout. `None` timeout waits forever.
///
/// The timeout wraps the wait for *one* frame and is restarted for the next one, so it only ever
/// fires on a client that has stopped sending — never on a slow but progressing upload.
pub async fn next_data_frame_with_timeout(
    incoming: &mut hyper::body::Incoming,
    timeout: Option<std::time::Duration>,
) -> Result<Result<Option<bytes::Bytes>, hyper::Error>, BodyReadTimeout> {
    let Some(timeout) = timeout else {
        return Ok(next_data_frame(incoming).await);
    };

    match tokio::time::timeout(timeout, next_data_frame(incoming)).await {
        Ok(frame) => Ok(frame),
        Err(_elapsed) => Err(BodyReadTimeout),
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
    expectations: BodyExpectations,
    buffer: usize,
) -> HttpBodyAsStream {
    let (sender, stream) = HttpBodyAsStream::create(buffer, expectations.content_length);

    tokio::spawn(pump(body, sender, expectations));

    stream
}

async fn pump(
    body: HttpRequestBody,
    sender: HttpBodyStreamSender,
    expectations: BodyExpectations,
) {
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

            let mut delivered: u64 = 0;
            let mut end_stream = EndStreamWatch::new();

            loop {
                let frame = tokio::select! {
                    // Checked first, on purpose. Without this arm a pump waiting on a client that
                    // went quiet would never learn that the handler walked away (returned 403
                    // without draining, say) and would hang on that body forever, holding the
                    // connection with it. `send_chunk() -> false` only reports it on the *next*
                    // chunk, which may never come.
                    biased;

                    _ = sender.closed() => return,

                    frame = next_data_frame_with_timeout(&mut incoming, expectations.read_timeout) => frame,
                };

                let frame = match frame {
                    Ok(frame) => frame,
                    Err(BodyReadTimeout) => {
                        sender
                            .send_error(HttpParseError::BodyStream(format!(
                                "Timeout while waiting for the request body: nothing received for {:?} after {} bytes",
                                expectations.read_timeout.unwrap_or_default(),
                                delivered
                            )))
                            .await;
                        return;
                    }
                };

                match frame {
                    Ok(Some(data)) => {
                        delivered += data.len() as u64;
                        // Before the send: a full channel parks us here, and a reset arriving
                        // during that wait would erase the flag.
                        end_stream.sample(&incoming);

                        if !sender.send_chunk(data.into()).await {
                            // Reader is gone.
                            return;
                        }
                    }
                    Ok(None) => {
                        // The body ended — but was it complete? See `BodyExpectations`.
                        end_stream.sample(&incoming);

                        if let Some(reason) =
                            expectations.incomplete_reason(delivered, end_stream.end_stream_seen())
                        {
                            sender
                                .send_error(HttpParseError::BodyStream(reason))
                                .await;
                            return;
                        }

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
