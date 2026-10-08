//! End-to-end coverage of a model with **named body fields** (`#[http_body]`), whose body is read
//! off the wire as it arrives and parsed with `parse_with_body_stream` — against a real server
//! over a raw TCP socket, because what is under test is how the body comes off hyper:
//!
//! * a body that arrives in pieces, or chunked, parses as one sent at once;
//! * the body is read to its end before the action runs, even past the fields it reads — a body
//!   cut short, or a client gone quiet, never reaches the action, and the connection is fit to be
//!   used again;
//! * a body a middleware has already materialized is parsed from what it read.
//!
//! A body that announced a `Content-Encoding` is materialized and decoded instead - that is
//! covered in `test_content_encoding_e2e`.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use my_http_server::controllers::ControllersMiddleware;
use my_http_server::{
    HttpContext, HttpFailResult, HttpOkResult, HttpServerMiddleware, MyHttpServer,
};
use rust_extensions::{AppStates, Logger};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// What the action saw, keyed by the `X-Test-Id` the request carried — the question under test
/// is whether the action ran at all, and on what.
pub static OUTCOMES: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());

fn set_outcome(test_id: String, value: String) {
    OUTCOMES.lock().unwrap().push((test_id, value));
}

fn take_outcome(test_id: &str) -> Option<String> {
    let mut outcomes = OUTCOMES.lock().unwrap();
    let pos = outcomes.iter().position(|(id, _)| id == test_id)?;
    Some(outcomes.remove(pos).1)
}

fn test_id(name: &str) -> String {
    format!("{}-{}", name, free_port())
}

// ─────────────────────────────── actions ───────────────────────────────

pub mod echo_email {
    use my_http_server::macros::*;
    use my_http_server::*;

    #[derive(MyHttpInput)]
    pub struct EchoEmailHttpInput {
        #[http_header(name = "X-Test-Id", description = "Test id")]
        pub test_id: String,

        #[http_body(name = "email", description = "Email")]
        pub email: String,

        #[http_body(name = "note", description = "Note")]
        pub note: Option<String>,
    }

    #[http_route(
        method: "POST",
        route: "/echo-email",
        controller: "Test",
        summary: "Echo email",
        description: "Echoes the named fields of the body",
        input_data: "EchoEmailHttpInput",
        result: [
            { status_code: 200, description: "Ok" },
        ]
    )]
    pub struct EchoEmailAction;

    async fn handle_request(
        _action: &EchoEmailAction,
        input_data: EchoEmailHttpInput,
        _ctx: &mut HttpContext,
    ) -> Result<HttpOkResult, HttpFailResult> {
        let result = format!("email={},note={:?}", input_data.email, input_data.note);

        super::set_outcome(input_data.test_id, result.clone());

        HttpOutput::as_text(result).into_ok_result(true).into()
    }
}

/// Reads the body before the controllers get to it, as a logging or signing middleware would.
struct MaterializeBodyMiddleware;

#[my_http_server::async_trait::async_trait]
impl HttpServerMiddleware for MaterializeBodyMiddleware {
    async fn handle_request(
        &self,
        ctx: &mut HttpContext,
    ) -> Option<Result<HttpOkResult, HttpFailResult>> {
        if let Err(err) = ctx.request.get_body().await {
            return Some(Err(err));
        }

        None
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

#[derive(Default)]
struct ServerSetup {
    h2: bool,
    body_read_timeout: Option<Duration>,
    materialize_in_middleware: bool,
}

async fn start_server(setup: ServerSetup) -> u16 {
    let port = free_port();

    let mut controllers = ControllersMiddleware::new(None, None);
    controllers.register_post_action(Arc::new(echo_email::EchoEmailAction));

    let mut server = MyHttpServer::new(SocketAddr::from(([127, 0, 0, 1], port)));

    if setup.materialize_in_middleware {
        server.add_middleware(Arc::new(MaterializeBodyMiddleware));
    }

    server.add_middleware(Arc::new(controllers));

    if let Some(body_read_timeout) = setup.body_read_timeout {
        server.set_body_read_timeout(body_read_timeout);
    }

    let app_states = Arc::new(AppStates::create_initialized());

    if setup.h2 {
        server.start_h2(app_states, Arc::new(SilentLogger));
    } else {
        server.start_h1(app_states, Arc::new(SilentLogger));
    }

    for _ in 0..100 {
        if TcpStream::connect(("127.0.0.1", port)).await.is_ok() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    port
}

fn request_head(id: &str, content_type: &str, framing: &str) -> String {
    format!(
        "POST /echo-email HTTP/1.1\r\nHost: localhost\r\nX-Test-Id: {}\r\nContent-Type: {}\r\n{}\r\nConnection: close\r\n\r\n",
        id, content_type, framing
    )
}

/// Reads the response until the server hangs up, or for 10 seconds at most.
async fn read_response(stream: &mut (impl AsyncReadExt + Unpin)) -> String {
    let mut buf = Vec::new();
    let _ = tokio::time::timeout(Duration::from_secs(10), stream.read_to_end(&mut buf)).await;
    String::from_utf8_lossy(&buf).to_string()
}

/// Sends the head and then the body piece by piece, with a pause between the pieces, so the
/// server gets the body in several reads.
async fn send_in_pieces(port: u16, head: String, pieces: &[&[u8]]) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    stream.write_all(head.as_bytes()).await.unwrap();

    for piece in pieces {
        stream.write_all(piece).await.unwrap();
        stream.flush().await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    read_response(&mut stream).await
}

fn json_body() -> &'static [u8] {
    br#"{"pad":"some text the model never reads","email":"a@b.c","note":"hi"}"#
}

// ─────────────────────────────── tests ───────────────────────────────

#[tokio::test]
async fn a_json_body_that_arrives_in_pieces_is_parsed() {
    let port = start_server(ServerSetup::default()).await;
    let id = test_id("pieces");

    let body = json_body();
    let head = request_head(
        &id,
        "application/json",
        &format!("Content-Length: {}", body.len()),
    );

    // Cut mid-key and mid-value, so a member is put together out of several reads
    let response = send_in_pieces(
        port,
        head,
        &[&body[..5], &body[5..30], &body[30..47], &body[47..]],
    )
    .await;

    assert!(
        response.starts_with("HTTP/1.1 200"),
        "response: {}",
        response
    );
    assert_eq!(
        take_outcome(&id).as_deref(),
        Some(r#"email=a@b.c,note=Some("hi")"#)
    );
}

#[tokio::test]
async fn a_chunked_json_body_is_parsed() {
    let port = start_server(ServerSetup::default()).await;
    let id = test_id("chunked");

    let body = json_body();
    let head = request_head(&id, "application/json", "Transfer-Encoding: chunked");

    let mut chunks = Vec::new();
    for piece in body.chunks(9) {
        chunks.push(format!("{:x}\r\n", piece.len()).into_bytes());
        chunks.push(piece.to_vec());
        chunks.push(b"\r\n".to_vec());
    }
    chunks.push(b"0\r\n\r\n".to_vec());

    let pieces: Vec<&[u8]> = chunks.iter().map(|chunk| chunk.as_slice()).collect();
    let response = send_in_pieces(port, head, &pieces).await;

    assert!(
        response.starts_with("HTTP/1.1 200"),
        "response: {}",
        response
    );
    assert_eq!(
        take_outcome(&id).as_deref(),
        Some(r#"email=a@b.c,note=Some("hi")"#)
    );
}

/// The parse stops reading once it has the fields the model reads, but the body is still read to
/// its end before the action runs: here the fields come first and the body then breaks off, and
/// the action must not run on it.
#[tokio::test]
async fn a_body_cut_short_after_the_fields_never_reaches_the_action() {
    let port = start_server(ServerSetup::default()).await;
    let id = test_id("cut-after-fields");

    let head = request_head(&id, "application/json", "Content-Length: 100000");

    let stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    let (mut read_half, mut write_half) = stream.into_split();

    write_half.write_all(head.as_bytes()).await.unwrap();
    write_half
        .write_all(br#"{"note":"hi","email":"a@b.c","pad":"never finished"#)
        .await
        .unwrap();
    write_half.flush().await.unwrap();
    write_half.shutdown().await.unwrap();

    let response = read_response(&mut read_half).await;

    assert!(
        take_outcome(&id).is_none(),
        "a body cut short after the fields reached the action"
    );
    assert!(
        response.is_empty() || response.starts_with("HTTP/1.1 400"),
        "response: {}",
        response
    );
}

/// Two requests on one HTTP/1 connection, the first with more body after its fields, sent late.
/// A body left unread on the connection would have it closed after the first answer.
#[tokio::test]
async fn the_connection_is_used_again_after_a_body_read_past_its_fields() {
    let port = start_server(ServerSetup::default()).await;
    let first_id = test_id("keep-alive-1");
    let second_id = test_id("keep-alive-2");

    let fields: &[u8] = br#"{"email":"a@b.c","note":"hi","pad":""#;
    let rest: &[u8] = br#"xyz"}"#;

    let first_head = format!(
        "POST /echo-email HTTP/1.1\r\nHost: localhost\r\nX-Test-Id: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
        first_id,
        fields.len() + rest.len()
    );

    let second_body = json_body();
    let second_request = format!(
        "{}{}",
        request_head(
            &second_id,
            "application/json",
            &format!("Content-Length: {}", second_body.len()),
        ),
        String::from_utf8_lossy(second_body)
    );

    let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    stream.write_all(first_head.as_bytes()).await.unwrap();
    stream.write_all(fields).await.unwrap();
    stream.flush().await.unwrap();

    // Long enough for the fields to be parsed before the rest of the body is there. Writes may
    // fail from here on if the server has hung up - the responses tell what happened.
    tokio::time::sleep(Duration::from_millis(300)).await;
    let _ = stream.write_all(rest).await;
    let _ = stream.write_all(second_request.as_bytes()).await;
    let _ = stream.flush().await;

    let response = read_response(&mut stream).await;

    assert_eq!(
        response.matches("HTTP/1.1 200").count(),
        2,
        "response: {}",
        response
    );
    assert_eq!(
        take_outcome(&first_id).as_deref(),
        Some(r#"email=a@b.c,note=Some("hi")"#)
    );
    assert_eq!(
        take_outcome(&second_id).as_deref(),
        Some(r#"email=a@b.c,note=Some("hi")"#)
    );
}

/// The fields the model reads are past the point where the body breaks off. The action must not
/// run, and must not be told a required field is missing either: the body was not complete.
#[tokio::test]
async fn a_body_cut_short_never_reaches_the_action() {
    let port = start_server(ServerSetup::default()).await;
    let id = test_id("cut-short");

    let head = request_head(&id, "application/json", "Content-Length: 100000");

    let stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    let (mut read_half, mut write_half) = stream.into_split();

    write_half.write_all(head.as_bytes()).await.unwrap();
    write_half
        .write_all(format!(r#"{{"pad":"{}"#, "x".repeat(1024)).as_bytes())
        .await
        .unwrap();
    write_half.flush().await.unwrap();

    // FIN on the write half only: hyper sees the body end early, we can still read the answer
    write_half.shutdown().await.unwrap();

    let response = read_response(&mut read_half).await;

    assert!(
        take_outcome(&id).is_none(),
        "a body cut short reached the action"
    );
    assert!(
        response.is_empty() || response.starts_with("HTTP/1.1 400"),
        "response: {}",
        response
    );
    assert!(
        !response.contains("email"),
        "a body cut short was answered as a missing field: {}",
        response
    );
}

#[tokio::test]
async fn a_client_gone_quiet_mid_body_is_cut_off() {
    let port = start_server(ServerSetup {
        body_read_timeout: Some(Duration::from_millis(400)),
        ..Default::default()
    })
    .await;
    let id = test_id("stall");

    let head = request_head(&id, "application/json", "Content-Length: 100000000");

    let started = std::time::Instant::now();

    let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    stream.write_all(head.as_bytes()).await.unwrap();
    stream.write_all(br#"{"pad":"#).await.unwrap();
    stream.flush().await.unwrap();

    let response = read_response(&mut stream).await;

    assert!(
        started.elapsed() < Duration::from_secs(5),
        "the client gone quiet was not cut off promptly: {:?}",
        started.elapsed()
    );
    assert!(
        take_outcome(&id).is_none(),
        "a stalled body reached the action"
    );
    assert!(
        response.starts_with("HTTP/1.1 400"),
        "response: {}",
        response
    );
    assert!(response.contains("Timeout"), "response: {}", response);
}

/// A url-encoded body can only be parsed whole: it is read to its end off the stream first.
#[tokio::test]
async fn a_url_encoded_body_is_parsed() {
    let port = start_server(ServerSetup::default()).await;
    let id = test_id("url-encoded");

    let body: &[u8] = b"pad=xyz&email=a%40b.c&note=hi";
    let head = request_head(
        &id,
        "application/x-www-form-urlencoded",
        &format!("Content-Length: {}", body.len()),
    );

    let response = send_in_pieces(port, head, &[&body[..10], &body[10..]]).await;

    assert!(
        response.starts_with("HTTP/1.1 200"),
        "response: {}",
        response
    );
    assert_eq!(
        take_outcome(&id).as_deref(),
        Some(r#"email=a@b.c,note=Some("hi")"#)
    );
}

#[tokio::test]
async fn a_malformed_json_body_never_reaches_the_action() {
    let port = start_server(ServerSetup::default()).await;
    let id = test_id("malformed");

    let body: &[u8] = br#"{"pad":tru,"email":"a@b.c"}"#;
    let head = request_head(
        &id,
        "application/json",
        &format!("Content-Length: {}", body.len()),
    );

    let response = send_in_pieces(port, head, &[body]).await;

    assert!(
        take_outcome(&id).is_none(),
        "a malformed body reached the action"
    );
    assert!(
        response.starts_with("HTTP/1.1 412"),
        "response: {}",
        response
    );
}

/// A body a middleware has read is not on the wire any more: it is parsed from what the
/// middleware read.
#[tokio::test]
async fn a_body_a_middleware_materialized_is_parsed_from_it() {
    let port = start_server(ServerSetup {
        materialize_in_middleware: true,
        ..Default::default()
    })
    .await;
    let id = test_id("materialized");

    let body = json_body();
    let head = request_head(
        &id,
        "application/json",
        &format!("Content-Length: {}", body.len()),
    );

    let response = send_in_pieces(port, head, &[&body[..20], &body[20..]]).await;

    assert!(
        response.starts_with("HTTP/1.1 200"),
        "response: {}",
        response
    );
    assert_eq!(
        take_outcome(&id).as_deref(),
        Some(r#"email=a@b.c,note=Some("hi")"#)
    );
}

/// Sends one HTTP/2 request with a url-encoded body and no `Content-Length` — the normal shape of
/// an h2 body of unknown size. `reset` aborts it with `RST_STREAM(NO_ERROR)` after the first
/// piece instead of ending the stream, which hyper hands on as an ordinary end of body.
async fn h2_send(port: u16, id: &str, reset: bool) -> Option<u16> {
    let tcp = TcpStream::connect(("127.0.0.1", port)).await.unwrap();

    let (mut client, connection) = h2::client::handshake(tcp).await.unwrap();
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let request = http::Request::builder()
        .method("POST")
        .uri("http://localhost/echo-email")
        .header("x-test-id", id)
        .header("content-type", "application/x-www-form-urlencoded")
        .body(())
        .unwrap();

    let (response, mut body) = client.send_request(request, false).unwrap();

    body.reserve_capacity(64);
    body.send_data(bytes::Bytes::from_static(b"pad=xyz&"), false)
        .unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;

    if reset {
        body.send_reset(h2::Reason::NO_ERROR);
    } else {
        body.send_data(bytes::Bytes::from_static(b"email=a%40b.c&note=hi"), true)
            .unwrap();
    }

    match tokio::time::timeout(Duration::from_secs(5), response).await {
        Ok(Ok(response)) => Some(response.status().as_u16()),
        _ => None,
    }
}

/// With no `Content-Length` there is nothing to hold the body to but h2's END_STREAM - and a body
/// that really did end with it must be read to its end and parsed, not rejected.
#[tokio::test]
async fn an_h2_body_with_no_content_length_is_parsed() {
    let port = start_server(ServerSetup {
        h2: true,
        ..Default::default()
    })
    .await;
    let id = test_id("h2-ok");

    let status = h2_send(port, &id, false).await;

    assert_eq!(status, Some(200));
    assert_eq!(
        take_outcome(&id).as_deref(),
        Some(r#"email=a@b.c,note=Some("hi")"#)
    );
}

#[tokio::test]
async fn an_h2_body_reset_mid_way_never_reaches_the_action() {
    let port = start_server(ServerSetup {
        h2: true,
        ..Default::default()
    })
    .await;
    let id = test_id("h2-reset");

    h2_send(port, &id, true).await;

    assert!(
        take_outcome(&id).is_none(),
        "a body reset mid-way reached the action"
    );
}

#[test]
fn the_model_consts_send_the_body_to_the_stream() {
    const { assert!(echo_email::EchoEmailHttpInput::READS_BODY) };
    const { assert!(!echo_email::EchoEmailHttpInput::READS_BODY_RAW) };
    const { assert!(!echo_email::EchoEmailHttpInput::STREAMS_BODY) };
}
