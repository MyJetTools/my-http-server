//! End-to-end coverage of `#[http_body_as_stream]` against a **real** server over a **raw TCP**
//! socket — the only way to answer the questions that matter here, because they are all about how
//! hyper behaves on the wire:
//!
//! * a `Content-Length` body and a `Transfer-Encoding: chunked` body must arrive identically;
//! * an action that returns a response **without draining the body** must not hang, and must not
//!   leave the pump holding the connection;
//! * a client that aborts mid-upload must surface as an **error**, never as a clean end of body
//!   (that would be a silently truncated upload — corruption that looks like success).

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use my_http_server::controllers::ControllersMiddleware;
use my_http_server::{HttpConnectionsCounter, MyHttpServer};
use rust_extensions::{AppStates, Logger};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// What each action saw, keyed by the `X-Test-Id` the request carried. Asserting on this rather
/// than on the HTTP response is deliberate: on an aborted upload the connection may die before any
/// response is written, and the question under test is what the *handler* observed.
///
/// Keyed, not global: `cargo test` runs these in parallel, and a shared slot lets one test read
/// another's outcome — which is exactly the false failure this started out with.
pub static OUTCOMES: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());

fn set_outcome(test_id: String, value: String) {
    OUTCOMES.lock().unwrap().push((test_id, value));
}

fn take_outcome(test_id: &str) -> Option<String> {
    let mut outcomes = OUTCOMES.lock().unwrap();
    let pos = outcomes.iter().position(|(id, _)| id == test_id)?;
    Some(outcomes.remove(pos).1)
}

/// A distinct id per test, so outcomes never cross.
fn test_id(name: &str) -> String {
    format!("{}-{}", name, free_port())
}

// ─────────────────────────────── actions ───────────────────────────────
// One `handle_request` per module: `#[http_route]` generates a call to a free function of that
// name, so two actions can not share a module.

pub mod upload {
    use my_http_server::macros::*;
    use my_http_server::*;

    #[derive(MyHttpInput)]
    pub struct UploadHttpInput {
        // Proves headers still parse alongside a streamed body — and keys the recorded outcome.
        #[http_header(name = "X-Test-Id", description = "Test id")]
        pub test_id: String,

        #[http_body_as_stream(description = "File content")]
        pub body: HttpBodyAsStream,
    }

    #[http_route(
        method: "POST",
        route: "/upload",
        controller: "Test",
        summary: "Upload",
        description: "Reads the body as a stream",
        input_data: "UploadHttpInput",
        result: [
            { status_code: 200, description: "Ok" },
        ]
    )]
    pub struct UploadAction;

    async fn handle_request(
        _action: &UploadAction,
        input_data: UploadHttpInput,
        _ctx: &mut HttpContext,
    ) -> Result<HttpOkResult, HttpFailResult> {
        let UploadHttpInput { test_id, body } = input_data;

        let content_length = body.get_content_length();

        let reader = body.get_body_reader()?;

        let mut total = 0usize;
        let mut chunks = 0usize;
        let mut sum: u64 = 0;

        loop {
            match reader.get_next_chunk().await {
                Ok(Some(chunk)) => {
                    chunks += 1;
                    total += chunk.len();
                    for b in chunk.iter() {
                        sum += *b as u64;
                    }
                }
                Ok(None) => break,
                Err(err) => {
                    // Recorded before bubbling up: an aborted upload may kill the connection
                    // before the response reaches the client.
                    super::set_outcome(test_id, format!("err:{}", err));
                    return Err(err.into());
                }
            }
        }

        super::set_outcome(
            test_id,
            format!(
                "ok:total={},chunks={},sum={},content_length={:?}",
                total, chunks, sum, content_length
            ),
        );

        HttpOutput::as_text(format!("total={},sum={}", total, sum))
            .into_ok_result(true)
            .into()
    }
}

pub mod reject {
    use my_http_server::macros::*;
    use my_http_server::*;

    #[derive(MyHttpInput)]
    pub struct RejectHttpInput {
        #[http_header(name = "X-Test-Id", description = "Test id")]
        pub test_id: String,

        #[http_body_as_stream(description = "File content")]
        pub body: HttpBodyAsStream,
    }

    #[http_route(
        method: "POST",
        route: "/reject",
        controller: "Test",
        summary: "Reject",
        description: "Answers without reading the body at all",
        input_data: "RejectHttpInput",
        result: [
            { status_code: 403, description: "Forbidden" },
        ]
    )]
    pub struct RejectAction;

    async fn handle_request(
        _action: &RejectAction,
        input_data: RejectHttpInput,
        _ctx: &mut HttpContext,
    ) -> Result<HttpOkResult, HttpFailResult> {
        // Deliberately does NOT touch the body: the reader is never taken, the stream is dropped
        // with the model. This is the case that must not hang.
        super::set_outcome(input_data.test_id, "rejected".to_string());
        Err(HttpFailResult::as_forbidden(Some("nope".to_string())))
    }
}

// ─────────────────────────────── harness ───────────────────────────────

struct SilentLogger;

impl Logger for SilentLogger {
    fn write_info(&self, _p: String, _m: String, _c: Option<HashMap<String, String>>) {}
    fn write_warning(&self, _p: String, _m: String, _c: Option<HashMap<String, String>>) {}
    fn write_error(&self, _p: String, _m: String, _c: Option<HashMap<String, String>>) {}
    fn write_fatal_error(&self, _p: String, _m: String, _c: Option<HashMap<String, String>>) {}
    fn write_debug_info(&self, _p: String, _m: String, _c: Option<HashMap<String, String>>) {}
}

fn free_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    port
}

async fn start_server() -> (u16, HttpConnectionsCounter) {
    let port = free_port();

    let mut controllers = ControllersMiddleware::new(None, None);
    controllers.register_post_action(Arc::new(upload::UploadAction));
    controllers.register_post_action(Arc::new(reject::RejectAction));

    let mut server = MyHttpServer::new(SocketAddr::from(([127, 0, 0, 1], port)));
    server.add_middleware(Arc::new(controllers));

    let counter = server.get_http_connections_counter();

    server.start_h1(
        Arc::new(AppStates::create_initialized()),
        Arc::new(SilentLogger),
    );

    // The listener comes up on a spawned task — wait for it to accept.
    for _ in 0..100 {
        if TcpStream::connect(("127.0.0.1", port)).await.is_ok() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    (port, counter)
}

/// Sends raw bytes and reads the whole response back. `Connection: close` in the request makes the
/// server hang up, so this returns as soon as the exchange is over rather than on a timeout.
async fn send_raw(port: u16, request: Vec<u8>) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    stream.write_all(&request).await.unwrap();
    stream.flush().await.unwrap();

    let mut buf = Vec::new();
    let _ = tokio::time::timeout(Duration::from_secs(10), stream.read_to_end(&mut buf)).await;

    String::from_utf8_lossy(&buf).to_string()
}

fn payload(size: usize) -> Vec<u8> {
    (0..size).map(|i| (i % 251) as u8).collect()
}

fn sum_of(data: &[u8]) -> u64 {
    data.iter().map(|b| *b as u64).sum()
}

// ─────────────────────────────── tests ───────────────────────────────

/// A body announced with `Content-Length` arrives whole, and the length is visible up front.
#[tokio::test]
async fn a_content_length_body_is_streamed_in_full() {
    let (port, _) = start_server().await;
    let id = test_id("content-length");

    let body = payload(300_000);

    let mut request = format!(
        "POST /upload HTTP/1.1\r\nHost: localhost\r\nX-Test-Id: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        id,
        body.len()
    )
    .into_bytes();
    request.extend_from_slice(&body);

    let response = send_raw(port, request).await;

    assert!(response.starts_with("HTTP/1.1 200"), "response: {}", response);
    assert!(
        response.contains(&format!("total={},sum={}", body.len(), sum_of(&body))),
        "response: {}",
        response
    );

    let outcome = take_outcome(&id).unwrap();
    assert!(
        outcome.starts_with(&format!("ok:total={},", body.len())),
        "outcome: {}",
        outcome
    );
    // The length was known up front, and it really was delivered in more than one chunk — i.e.
    // this test would have caught a "read it all at once" implementation.
    assert!(
        outcome.contains(&format!("content_length=Some({})", body.len())),
        "outcome: {}",
        outcome
    );
    assert!(!outcome.contains("chunks=1,"), "not streamed: {}", outcome);
}

/// The same body sent chunked must land identically — the frame loop is what makes the two paths
/// the same, and `Content-Length` is simply absent.
#[tokio::test]
async fn a_chunked_body_is_streamed_in_full() {
    let (port, _) = start_server().await;
    let id = test_id("chunked");

    let body = payload(300_000);

    let mut request = format!(
        "POST /upload HTTP/1.1\r\nHost: localhost\r\nX-Test-Id: {}\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n",
        id
    )
    .into_bytes();

    for part in body.chunks(50_000) {
        request.extend_from_slice(format!("{:x}\r\n", part.len()).as_bytes());
        request.extend_from_slice(part);
        request.extend_from_slice(b"\r\n");
    }
    request.extend_from_slice(b"0\r\n\r\n");

    let response = send_raw(port, request).await;

    assert!(response.starts_with("HTTP/1.1 200"), "response: {}", response);
    assert!(
        response.contains(&format!("total={},sum={}", body.len(), sum_of(&body))),
        "response: {}",
        response
    );

    let outcome = take_outcome(&id).unwrap();
    assert!(
        outcome.starts_with(&format!("ok:total={},", body.len())),
        "outcome: {}",
        outcome
    );
    // Chunked carries no length up front.
    assert!(outcome.contains("content_length=None"), "outcome: {}", outcome);
    assert!(!outcome.contains("chunks=1,"), "not streamed: {}", outcome);
}

/// THE experiment. An action answers 403 while the client is still uploading and never touches the
/// body. The pump is already running at that point, so this is exactly the case where a detached
/// task could hang on `Incoming` forever, holding the connection with it.
///
/// The assertions are the two things that would actually hurt: the response must come back
/// promptly, and the connection must be released afterwards.
#[tokio::test]
async fn an_action_that_never_reads_the_body_does_not_hang() {
    let (port, counter) = start_server().await;
    let id = test_id("reject");

    let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();

    // A huge announced body, of which we send only a sliver and then go quiet — the worst case for
    // a pump that has no way of learning the reader left.
    stream
        .write_all(
            format!(
                "POST /reject HTTP/1.1\r\nHost: localhost\r\nX-Test-Id: {}\r\nContent-Length: 1000000000\r\nConnection: close\r\n\r\n",
                id
            )
            .as_bytes(),
        )
        .await
        .unwrap();
    stream.write_all(&payload(1024)).await.unwrap();
    stream.flush().await.unwrap();

    let mut buf = vec![0u8; 4096];
    let read = tokio::time::timeout(Duration::from_secs(5), stream.read(&mut buf))
        .await
        .expect("the server did not answer within 5s — the pump is hanging on the body");

    let read = read.unwrap();
    let response = String::from_utf8_lossy(&buf[..read]).to_string();

    assert!(response.starts_with("HTTP/1.1 403"), "response: {}", response);
    assert_eq!(take_outcome(&id).unwrap(), "rejected");

    drop(stream);

    // And the connection is actually let go, rather than pinned by a pump still reading a body
    // nobody wants.
    let mut released = false;
    for _ in 0..100 {
        if counter.get_connections_amount() == 0 {
            released = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(
        released,
        "connection was never released: {} still open",
        counter.get_connections_amount()
    );
}

/// A client that announces 100_000 bytes and delivers 1_024 before closing must NOT look to the
/// handler like a body that ended cleanly. Getting this wrong writes a truncated file and reports
/// success.
#[tokio::test]
async fn an_aborted_upload_is_an_error_not_a_short_body() {
    let (port, _) = start_server().await;
    let id = test_id("aborted");

    let stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    let (mut read_half, mut write_half) = stream.into_split();

    write_half
        .write_all(
            format!(
                "POST /upload HTTP/1.1\r\nHost: localhost\r\nX-Test-Id: {}\r\nContent-Length: 100000\r\nConnection: close\r\n\r\n",
                id
            )
            .as_bytes(),
        )
        .await
        .unwrap();
    write_half.write_all(&payload(1024)).await.unwrap();
    write_half.flush().await.unwrap();

    // FIN on the write half only: hyper sees the body end early, we can still read the answer.
    write_half.shutdown().await.unwrap();

    let mut buf = Vec::new();
    let _ = tokio::time::timeout(Duration::from_secs(10), read_half.read_to_end(&mut buf)).await;

    // Whatever reached the wire, the handler must have seen a failure — never `Ok(None)` after
    // 1024 bytes.
    let outcome = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if let Some(outcome) = take_outcome(&id) {
                return outcome;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("the handler never recorded an outcome");

    assert!(
        outcome.starts_with("err:"),
        "an aborted upload was reported as a successful short body: {}",
        outcome
    );
}

/// A request with no body at all: the stream must end immediately rather than block waiting for
/// bytes that will never come.
#[tokio::test]
async fn an_empty_body_ends_the_stream_at_once() {
    let (port, _) = start_server().await;
    let id = test_id("empty");

    let request = format!(
        "POST /upload HTTP/1.1\r\nHost: localhost\r\nX-Test-Id: {}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        id
    )
    .into_bytes();

    let response = send_raw(port, request).await;

    assert!(response.starts_with("HTTP/1.1 200"), "response: {}", response);

    let outcome = take_outcome(&id).unwrap();
    assert!(
        outcome.starts_with("ok:total=0,chunks=0,"),
        "outcome: {}",
        outcome
    );
}

/// A middleware may have materialized the body before the action ran. Streaming still has to work
/// then — as a single chunk — instead of blowing up or reporting an empty body.
#[tokio::test]
async fn a_body_already_materialized_by_a_middleware_still_streams() {
    use my_http_server::{BodyContentType, HttpRequestBody, HttpRequestBodyContent};

    let content =
        HttpRequestBodyContent::new(b"hello world".to_vec(), BodyContentType::Unknown).unwrap();

    let stream = HttpRequestBody::Full(content).into_body_stream(Some(11), 4);
    let reader = stream.get_body_reader().unwrap();

    assert_eq!(reader.get_content_length(), Some(11));
    assert_eq!(
        reader.get_next_chunk().await.unwrap().unwrap(),
        b"hello world".to_vec()
    );
    // And a clean end — not the "ended unexpectedly" error, i.e. `finish()` really was called on
    // this path too.
    assert!(reader.get_next_chunk().await.unwrap().is_none());
}

/// Same path, empty: no chunk at all, still a clean end.
#[tokio::test]
async fn an_already_materialized_empty_body_streams_as_a_clean_end() {
    use my_http_server::{BodyContentType, HttpRequestBody, HttpRequestBodyContent};

    let content = HttpRequestBodyContent::new(Vec::new(), BodyContentType::Empty).unwrap();

    let stream = HttpRequestBody::Full(content).into_body_stream(None, 4);
    let reader = stream.get_body_reader().unwrap();

    assert!(reader.get_next_chunk().await.unwrap().is_none());
}

/// The plumbing itself: a streaming model streams and does not materialize, and an ordinary model
/// is untouched by any of this.
#[test]
fn the_model_consts_say_which_path_the_body_takes() {
    assert!(upload::UploadHttpInput::STREAMS_BODY);
    assert!(!upload::UploadHttpInput::READS_BODY);

    assert!(!crate::test_macros_split::UpdateUserRequest::STREAMS_BODY);
    assert!(crate::test_macros_split::UpdateUserRequest::READS_BODY);
}
