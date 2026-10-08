# Streamed Request Body

**Streamed body (`#[http_body_as_stream]`) — read it in chunks, never materialize it:**

Every body kind above puts the whole body in memory before the handler runs. For uploads and proxying that is not acceptable. `#[http_body_as_stream]` hands the handler a reader instead:

```rust
use my_http_server::HttpBodyAsStream;

#[derive(MyHttpInput)]
pub struct UploadInputModel {
    #[http_header(name = "X-File-Name", description = "File name")]
    pub file_name: String,

    #[http_body_as_stream(description = "File content")]
    pub body: HttpBodyAsStream,
}

async fn handle_request(
    _action: &ActionName,
    input_data: UploadInputModel,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let reader = input_data.body.get_body_reader()?;

    // Some(n) when the client sent Content-Length; None for a chunked body.
    let _expected = reader.get_content_length();

    while let Some(chunk) = reader.get_next_chunk().await? {
        // chunk: Vec<u8> — write it to a file, hash it, forward it...
    }

    HttpOutput::as_text("ok".to_string()).into_ok_result(true).into()
}
```

It works identically for `Transfer-Encoding: chunked` and for a `Content-Length` body — the server reads hyper's DATA frames either way, and the only difference is whether the length is known up front.

Things worth knowing before using it:

- **Memory.** The chunks travel through a *bounded* channel (`BODY_STREAM_DEFAULT_BUFFER`), so a request costs roughly `buffer × chunk size` no matter how big the upload is. A pump that runs into a full channel parks, and that back-pressure reaches the TCP window. `take_body_stream_with_buffer` changes the capacity.
- **A truncated upload is an error, not a short body.** If the client disappears mid-upload, `get_next_chunk()` returns `Err(HttpParseError::BodyStream(..))` — it never reports a clean end after a partial body. Do not "handle" that by treating it as end-of-data; you would be writing half a file and calling it success.

  The server checks this itself rather than trusting the transport, because "the body ended" and "the body is complete" are not the same thing. A body that ends short of the `Content-Length` the client announced is rejected, and on HTTP/2 — where an aborted upload arrives as a perfectly ordinary end of stream (`RST_STREAM(NO_ERROR)`), and where an upload of unknown size carries no length at all, since h2 has no chunked encoding — a stream that ended without `END_STREAM` is rejected too. HTTP/1 truncation is caught by hyper itself.

  The same rule covers bodies that are *not* streamed: `#[http_body_raw]`, `#[http_body]` and any middleware calling `get_body()` go through it as well, so a truncated upload never reaches an action's business logic looking like the whole thing.

  One HTTP/2 case is decided conservatively rather than correctly, because it can not be decided at all. A client may send `END_STREAM` and then immediately reset the stream to cancel a request it is no longer waiting for; h2 overwrites the state that recorded `END_STREAM` with the reset, so a complete body becomes indistinguishable from a truncated one. The server keeps the flag from the moment the frame arrives, which covers this whenever `END_STREAM` was seen first; in the remaining race the body is reported as possibly incomplete. A client that sends `Content-Length` is unaffected — the length settles it. This only matters if you read the body from a task that outlives the handler, since hyper drops the handler on a reset anyway.
- **`read_to_end(max_size)`** is there when you just want the bytes but still want a ceiling: it fails past `max_size` instead of allocating without limit.
- **Returning without reading is fine.** An action may answer 403 and never touch the reader; the pump notices the reader is gone and stops. It does not hang and does not hold the connection.
- **Nothing waits on a silent client forever — if you switch it on.** `MyHttpServer::set_body_read_timeout(duration)` stops a body read that has gone quiet. It is an **idle** timeout: it restarts for every piece of the body, so a large upload over a slow link is never cut off, and time the pump spends parked because *your handler* is slow to consume does not count against it either. It is **off by default**, keeping the behaviour the server has always had — but without it a client that announces a large body and then goes silent holds a connection, and for a streamed body a pump task, indefinitely. It applies to every way of reading a body — streamed, parsed as it arrives, and materialized.
- **The body can only be taken once.** `get_body_reader()` hands out exactly one reader; a second call fails. And like `receive_body()`, taking the stream consumes the body — a middleware that reads the body afterwards will find it gone.
- **Not for client models.** A model with a streamed body describes an incoming request only; building it into an outgoing request fails.

Swagger shows the body as `type: string, format: binary`.
