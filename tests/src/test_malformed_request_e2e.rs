//! End-to-end coverage of requests the server can not take apart: a target which is not a path
//! (`OPTIONS *`, `CONNECT host:port`), a header which does not read as text, a multipart content
//! type without a boundary. Each one used to panic in `handle_requests` before any middleware ran,
//! killing the connection's task: the client was left without an answer. Now each is answered with
//! the error the library already makes for it - 400 or 412 - and the server goes on serving.
//!
//! The socket is written to by hand, so the request line and the raw bytes of a header are exactly
//! what gets to the wire.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use my_http_server::*;
use rust_extensions::{AppStates, Logger};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

// ─────────────────────────────── harness ───────────────────────────────

/// Answers every request it gets with `200 ok <method> <path>`.
struct EchoMiddleware;

#[my_http_server::async_trait::async_trait]
impl HttpServerMiddleware for EchoMiddleware {
    async fn handle_request(
        &self,
        ctx: &mut HttpContext,
    ) -> Option<Result<HttpOkResult, HttpFailResult>> {
        let text = format!(
            "ok {} {}",
            ctx.request.method,
            ctx.request.http_path.as_str()
        );
        Some(HttpOutput::as_text(text).into_ok_result(false))
    }
}

struct SilentLogger;

impl Logger for SilentLogger {
    fn write_info(&self, _p: String, _m: String, _c: Option<HashMap<String, String>>) {}
    fn write_warning(&self, _p: String, _m: String, _c: Option<HashMap<String, String>>) {}
    fn write_error(&self, _p: String, _m: String, _c: Option<HashMap<String, String>>) {}
    fn write_fatal_error(&self, _p: String, _m: String, _c: Option<HashMap<String, String>>) {}
    fn write_debug_info(&self, _p: String, _m: String, _c: Option<HashMap<String, String>>) {}
}

/// The two accept loops a service gets: `start` (auto h1/h2) and `start_h1`.
#[derive(Clone, Copy, Debug)]
enum Mode {
    Auto,
    H1,
}

async fn start_server(mode: Mode) -> (u16, HttpConnectionsCounter) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let mut server = MyHttpServer::new(SocketAddr::from(([127, 0, 0, 1], port)));
    server.add_middleware(Arc::new(EchoMiddleware));
    let connections = server.get_http_connections_counter();

    let app_states = Arc::new(AppStates::create_initialized());
    match mode {
        Mode::Auto => server.start(app_states, Arc::new(SilentLogger)),
        Mode::H1 => server.start_h1(app_states, Arc::new(SilentLogger)),
    }

    for _ in 0..100 {
        if TcpStream::connect(("127.0.0.1", port)).await.is_ok() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    (port, connections)
}

struct Response {
    status: u16,
    body: String,
}

/// Writes `request` to a new connection exactly as it is and reads the response to it. `None`
/// when the connection ends without a whole response - a dropped connection.
async fn send(port: u16, request: &[u8]) -> Option<Response> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    stream.write_all(request).await.unwrap();
    stream.flush().await.unwrap();

    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];

    loop {
        if let Some(head_end) = buf.windows(4).position(|window| window == b"\r\n\r\n") {
            let head = String::from_utf8_lossy(&buf[..head_end]).to_string();

            let content_length: usize = head
                .lines()
                .skip(1)
                .filter_map(|line| line.split_once(':'))
                .find(|(name, _)| name.trim().eq_ignore_ascii_case("content-length"))
                .and_then(|(_, value)| value.trim().parse().ok())
                .unwrap_or(0);

            let body_start = head_end + 4;
            if buf.len() >= body_start + content_length {
                let status = head.split_whitespace().nth(1)?.parse().ok()?;
                let body = &buf[body_start..body_start + content_length];

                return Some(Response {
                    status,
                    body: String::from_utf8_lossy(body).to_string(),
                });
            }
        }

        let read = tokio::time::timeout(Duration::from_secs(10), stream.read(&mut chunk))
            .await
            .ok()?
            .ok()?;

        if read == 0 {
            return None;
        }

        buf.extend_from_slice(&chunk[..read]);
    }
}

async fn assert_get_root_is_served(port: u16, after: &str) {
    let response = send(
        port,
        b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    )
    .await
    .unwrap_or_else(|| panic!("GET / after {after}: the connection was dropped"));

    assert_eq!(response.status, 200, "GET / after {after}");
    assert_eq!(response.body, "ok GET /", "GET / after {after}");
}

// ─────────────────────────────── tests ───────────────────────────────

/// The rows of the table each of which used to drop the connection, with the status each is
/// answered with now.
fn malformed_requests() -> Vec<(&'static str, &'static [u8], u16)> {
    vec![
        (
            "OPTIONS *",
            b"OPTIONS * HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
            400,
        ),
        (
            "GET *",
            b"GET * HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
            400,
        ),
        (
            "CONNECT example.com:443",
            b"CONNECT example.com:443 HTTP/1.1\r\nHost: example.com:443\r\nConnection: close\r\n\r\n",
            400,
        ),
        (
            "multipart/form-data without a boundary",
            b"GET / HTTP/1.1\r\nHost: localhost\r\nContent-Type: multipart/form-data\r\nConnection: close\r\n\r\n",
            412,
        ),
        (
            "0xFF in Content-Type",
            b"GET / HTTP/1.1\r\nHost: localhost\r\nContent-Type: text/\xff\r\nConnection: close\r\n\r\n",
            400,
        ),
        (
            "0xFF in Content-Encoding",
            b"GET / HTTP/1.1\r\nHost: localhost\r\nContent-Encoding: g\xffzip\r\nConnection: close\r\n\r\n",
            400,
        ),
    ]
}

#[tokio::test]
async fn a_malformed_request_is_answered_and_the_server_goes_on() {
    for mode in [Mode::Auto, Mode::H1] {
        let (port, connections) = start_server(mode).await;

        for (name, request, expected_status) in malformed_requests() {
            let response = send(port, request)
                .await
                .unwrap_or_else(|| panic!("{mode:?} {name}: the connection was dropped"));

            assert_eq!(response.status, expected_status, "{mode:?} {name}");

            // The answer is the library's own, not a parse error of hyper's (which has no body).
            if expected_status == 400 {
                assert!(
                    response.body.starts_with("Validation error"),
                    "{mode:?} {name}: {}",
                    response.body
                );
            }

            assert_get_root_is_served(port, &format!("{mode:?} {name}")).await;
        }

        // No connection's task died along the way: every one of them was let go of properly.
        let mut amount = connections.get_connections_amount();
        for _ in 0..250 {
            if amount == 0 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
            amount = connections.get_connections_amount();
        }
        assert_eq!(amount, 0, "{mode:?}: connections still counted");
    }
}

/// The rows of the table which were fine already stay as they were - an absolute-form target
/// among them.
#[tokio::test]
async fn well_formed_requests_are_served_as_before() {
    let requests: [(&str, &[u8], &str); 4] = [
        (
            "GET http://x",
            b"GET http://x HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n",
            "ok GET /",
        ),
        (
            "GET http://x/some/path?q=1",
            b"GET http://x/some/path?q=1 HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n",
            "ok GET /some/path",
        ),
        (
            "OPTIONS /path",
            b"OPTIONS /path HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
            "ok OPTIONS /path",
        ),
        (
            "0xFF in User-Agent",
            b"GET / HTTP/1.1\r\nHost: localhost\r\nUser-Agent: curl/\xff\r\nConnection: close\r\n\r\n",
            "ok GET /",
        ),
    ];

    for mode in [Mode::Auto, Mode::H1] {
        let (port, _) = start_server(mode).await;

        for (name, request, expected_body) in requests {
            let response = send(port, request)
                .await
                .unwrap_or_else(|| panic!("{mode:?} {name}: the connection was dropped"));

            assert_eq!(response.status, 200, "{mode:?} {name}");
            assert_eq!(response.body, expected_body, "{mode:?} {name}");
        }
    }
}
