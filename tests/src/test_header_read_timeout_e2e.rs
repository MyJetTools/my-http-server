//! End-to-end coverage of the header read timeout - `set_header_read_timeout` - against a real
//! server over a raw socket.
//!
//! A client that sends the start of a request and then trickles the rest a byte at a time, or goes
//! quiet - slowloris - has its connection closed once the timeout is up, through each of the four
//! HTTP/1 accept loops: `start` and `start_h1`, on TCP and on a Unix socket. So does a client that
//! connects and sends nothing at all. The server goes on serving afterwards.
//!
//! The timeout covers the head of a request only: a slow body, a handler that takes its time (long
//! polling), a long streamed response and an upgraded connection all outlive it, and so does an
//! HTTP/2 connection. An idle keep-alive connection does not: it is waiting for the head of its
//! next request.
//!
//! The socket is written to by hand: an HTTP client would not send half a head.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use my_http_server::my_hyper_utils::ToMyHttpResponse;
use my_http_server::*;
use rust_extensions::{AppStates, Logger};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;

/// The header read timeout of every server here.
const TIMEOUT: Duration = Duration::from_secs(1);

/// Longer than [`TIMEOUT`], by enough that a timeout which did apply would have fired.
const LONGER_THAN_TIMEOUT: Duration = Duration::from_millis(2500);

/// A connection closed by the timeout is closed no sooner than this after it was opened - a bit
/// under [`TIMEOUT`], for the timer's granularity.
const CLOSED_NO_SOONER: Duration = Duration::from_millis(900);

/// A connection closed by the timeout is closed within this after it was opened.
const CLOSED_WITHIN: Duration = Duration::from_secs(5);

const GET: &[u8] = b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";

// ─────────────────────────────── harness ───────────────────────────────

/// - `/slow` answers after [`LONGER_THAN_TIMEOUT`] - long polling;
/// - `/echo` answers with the body of the request;
/// - `/stream` streams [`STREAM_CHUNKS`] chunks, [`STREAM_CHUNK_PAUSE`] apart;
/// - `/upgrade` switches the connection to a protocol that echoes what it gets;
/// - anything else is answered `200 ok`.
struct TestMiddleware;

const STREAM_CHUNKS: usize = 4;
const STREAM_CHUNK_PAUSE: Duration = Duration::from_millis(700);

#[my_http_server::async_trait::async_trait]
impl HttpServerMiddleware for TestMiddleware {
    async fn handle_request(
        &self,
        ctx: &mut HttpContext,
    ) -> Option<Result<HttpOkResult, HttpFailResult>> {
        match ctx.request.http_path.as_str() {
            "/slow" => {
                tokio::time::sleep(LONGER_THAN_TIMEOUT).await;
                Some(HttpOutput::as_text("slow ok".to_string()).into_ok_result(false))
            }
            "/echo" => {
                let body = match ctx.request.receive_body().await {
                    Ok(body) => body.get_body(),
                    Err(err) => return Some(Err(err)),
                };
                let text = format!("echo {}", String::from_utf8(body).unwrap());
                Some(HttpOutput::as_text(text).into_ok_result(false))
            }
            "/stream" => {
                let (output, mut producer) = HttpOutput::as_stream(STREAM_CHUNKS);
                tokio::spawn(async move {
                    for i in 0..STREAM_CHUNKS {
                        tokio::time::sleep(STREAM_CHUNK_PAUSE).await;
                        if producer.send(format!("chunk{i};").into_bytes()).await.is_err() {
                            return;
                        }
                    }
                });
                Some(output.get_result())
            }
            "/upgrade" => Some(upgrade_to_echo(ctx)),
            _ => Some(HttpOutput::as_text("ok".to_string()).into_ok_result(false)),
        }
    }
}

/// Answers `101 Switching Protocols`, and from then on echoes whatever comes over the connection.
fn upgrade_to_echo(ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
    let MyHyperHttpRequest::Incoming(mut req) = ctx.request.take_my_hyper_http_request() else {
        panic!("the request has been read already");
    };

    let on_upgrade = my_http_server::hyper::upgrade::on(&mut req);
    tokio::spawn(async move {
        let Ok(upgraded) = on_upgrade.await else {
            return;
        };
        let mut io = hyper_util::rt::TokioIo::new(upgraded);
        let mut buf = [0u8; 64];
        loop {
            match io.read(&mut buf).await {
                Ok(0) | Err(_) => return,
                Ok(read) => {
                    if io.write_all(&buf[..read]).await.is_err() {
                        return;
                    }
                }
            }
        }
    });

    let builder = my_http_server::hyper::Response::builder()
        .status(101)
        .header("connection", "upgrade")
        .header("upgrade", "echo");
    HttpOutput::Raw((builder, Vec::new()).to_my_http_response()).into_ok_result(false)
}

struct SilentLogger;

impl Logger for SilentLogger {
    fn write_info(&self, _p: String, _m: String, _c: Option<HashMap<String, String>>) {}
    fn write_warning(&self, _p: String, _m: String, _c: Option<HashMap<String, String>>) {}
    fn write_error(&self, _p: String, _m: String, _c: Option<HashMap<String, String>>) {}
    fn write_fatal_error(&self, _p: String, _m: String, _c: Option<HashMap<String, String>>) {}
    fn write_debug_info(&self, _p: String, _m: String, _c: Option<HashMap<String, String>>) {}
}

/// Which `start*` method the server is started with.
#[derive(Clone, Copy)]
enum Start {
    Auto,
    H1,
}

/// Starts `server` with `header_read_timeout`.
fn start(mut server: MyHttpServer, start: Start, header_read_timeout: Option<Duration>) {
    server.set_header_read_timeout(header_read_timeout);
    server.add_middleware(Arc::new(TestMiddleware));

    let app_states = Arc::new(AppStates::create_initialized());
    match start {
        Start::Auto => server.start(app_states, Arc::new(SilentLogger)),
        Start::H1 => server.start_h1(app_states, Arc::new(SilentLogger)),
    }
}

/// A TCP server on a free port, started with [`TIMEOUT`]. Hands back the port.
fn start_tcp(start_with: Start) -> u16 {
    let port = free_port();
    start(
        MyHttpServer::new(SocketAddr::from(([127, 0, 0, 1], port))),
        start_with,
        Some(TIMEOUT),
    );
    port
}

fn free_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

/// Connects as soon as the server is up.
async fn connect_tcp(port: u16) -> TcpStream {
    for _ in 0..100 {
        if let Ok(stream) = TcpStream::connect(("127.0.0.1", port)).await {
            return stream;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("the server never came up on port {port}");
}

/// Where a Unix socket server listens, in the temp dir. The socket file is removed on drop, a
/// failing test included.
#[cfg(unix)]
struct UnixSocketPath(String);

#[cfg(unix)]
impl UnixSocketPath {
    fn new(name: &str) -> Self {
        let file = format!("mhs-header-timeout-{}-{name}.sock", std::process::id());
        Self(
            std::env::temp_dir()
                .join(file)
                .to_str()
                .unwrap()
                .to_string(),
        )
    }
}

#[cfg(unix)]
impl Drop for UnixSocketPath {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// [`connect_tcp`], on a Unix socket.
#[cfg(unix)]
async fn connect_unix(path: &str) -> tokio::net::UnixStream {
    for _ in 0..100 {
        if let Ok(stream) = tokio::net::UnixStream::connect(path).await {
            return stream;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("the server never came up on {path}");
}

/// Reads until the server closes the connection - an end of stream or a reset - and hands back
/// what came before it. Panics if it is still open after `within`.
async fn read_until_closed(stream: &mut (impl AsyncRead + Unpin), within: Duration) -> Vec<u8> {
    let mut received = Vec::new();
    let mut buf = [0u8; 1024];

    let closed = tokio::time::timeout(within, async {
        loop {
            match stream.read(&mut buf).await {
                Ok(0) | Err(_) => return,
                Ok(read) => received.extend_from_slice(&buf[..read]),
            }
        }
    })
    .await;

    assert!(
        closed.is_ok(),
        "the connection is still open after {within:?}; received: {:?}",
        String::from_utf8_lossy(&received)
    );

    received
}

/// Reads one response off `stream`: the head, and the body by its `Content-Length` or up to the
/// last chunk. Within 5 s, or panics.
async fn read_response(stream: &mut (impl AsyncRead + Unpin)) -> String {
    let mut received = Vec::new();
    let mut buf = [0u8; 1024];
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);

    loop {
        if let Some(response) = complete_response(&received) {
            return response;
        }

        let read = tokio::time::timeout_at(deadline, stream.read(&mut buf))
            .await
            .unwrap_or_else(|_| {
                panic!(
                    "no complete response within 5s; received: {:?}",
                    String::from_utf8_lossy(&received)
                )
            })
            .unwrap();

        assert!(
            read > 0,
            "the connection was closed mid-response; received: {:?}",
            String::from_utf8_lossy(&received)
        );
        received.extend_from_slice(&buf[..read]);
    }
}

/// The response at the start of `received`, once all of it is there.
fn complete_response(received: &[u8]) -> Option<String> {
    let text = std::str::from_utf8(received).ok()?;
    let head_end = text.find("\r\n\r\n")? + 4;
    let head = text[..head_end].to_ascii_lowercase();

    if head.starts_with("http/1.1 101") {
        return Some(text[..head_end].to_string());
    }

    if let Some(length) = head
        .lines()
        .find_map(|line| line.strip_prefix("content-length:"))
    {
        let length: usize = length.trim().parse().unwrap();
        return (text.len() >= head_end + length).then(|| text[..head_end + length].to_string());
    }

    if head.contains("transfer-encoding: chunked") {
        return text.ends_with("\r\n0\r\n\r\n").then(|| text.to_string());
    }

    Some(text[..head_end].to_string())
}

/// `GET /` on `stream`, answered `200 ok`.
async fn assert_served(stream: &mut (impl AsyncRead + AsyncWrite + Unpin)) {
    stream.write_all(GET).await.unwrap();
    let response = read_response(stream).await;
    assert!(response.starts_with("HTTP/1.1 200"), "response: {response}");
    assert!(response.ends_with("ok"), "response: {response}");
}

/// The client sends the request line and a header, then trickles one more header a byte every
/// 100 ms and never ends the head. The server closes the connection once the timeout is up: the
/// timeout is for the whole head, a client still sending does not restart it. No response is sent.
async fn assert_trickled_head_is_cut(stream: impl AsyncRead + AsyncWrite + Unpin + Send + 'static) {
    let opened = Instant::now();
    let (mut reader, mut writer) = tokio::io::split(stream);

    let trickle = tokio::spawn(async move {
        writer
            .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n")
            .await?;
        loop {
            for byte in b"X-Slow: 1\r\n" {
                tokio::time::sleep(Duration::from_millis(100)).await;
                writer.write_all(&[*byte]).await?;
            }
        }
        #[allow(unreachable_code)]
        Ok::<(), std::io::Error>(())
    });

    let received = read_until_closed(&mut reader, CLOSED_WITHIN).await;
    trickle.abort();

    assert_closed_by_the_timeout(opened, &received);
}

/// The client sends half a head and goes quiet: closed once the timeout is up, no response sent.
async fn assert_half_a_head_is_cut(mut stream: impl AsyncRead + AsyncWrite + Unpin) {
    let opened = Instant::now();
    stream
        .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n")
        .await
        .unwrap();

    let received = read_until_closed(&mut stream, CLOSED_WITHIN).await;
    assert_closed_by_the_timeout(opened, &received);
}

/// The client sends the start of the HTTP/2 preface and goes quiet: closed once the timeout is up.
/// Only for `start`, whose auto builder reads the first bytes to tell HTTP/2 from HTTP/1 - to the
/// HTTP/1 parser of `start_h1` these bytes are a request line of an unknown version.
async fn assert_half_a_preface_is_cut(mut stream: impl AsyncRead + AsyncWrite + Unpin) {
    let opened = Instant::now();
    stream.write_all(b"PRI * HTTP/2.0\r\n").await.unwrap();

    let received = read_until_closed(&mut stream, CLOSED_WITHIN).await;
    assert_closed_by_the_timeout(opened, &received);
}

/// The client connects and sends nothing: closed once the timeout is up.
async fn assert_silence_is_cut(mut stream: impl AsyncRead + AsyncWrite + Unpin) {
    let opened = Instant::now();
    let received = read_until_closed(&mut stream, CLOSED_WITHIN).await;
    assert_closed_by_the_timeout(opened, &received);
}

fn assert_closed_by_the_timeout(opened: Instant, received: &[u8]) {
    let elapsed = opened.elapsed();
    assert!(
        elapsed >= CLOSED_NO_SOONER,
        "closed after {elapsed:?}, before the timeout"
    );
    assert!(
        received.is_empty(),
        "a response was sent: {:?}",
        String::from_utf8_lossy(received)
    );
}

// ─────────────────────────── the four accept loops ───────────────────────────

/// `start` on TCP: `start_http_auto`.
#[tokio::test]
async fn start_closes_a_tcp_connection_whose_head_does_not_come() {
    let port = start_tcp(Start::Auto);

    tokio::join!(
        async { assert_trickled_head_is_cut(connect_tcp(port).await).await },
        async { assert_half_a_head_is_cut(connect_tcp(port).await).await },
        async { assert_silence_is_cut(connect_tcp(port).await).await },
        async { assert_half_a_preface_is_cut(connect_tcp(port).await).await },
    );

    assert_served(&mut connect_tcp(port).await).await;
}

/// `start_h1` on TCP: `start_http_1`.
#[tokio::test]
async fn start_h1_closes_a_tcp_connection_whose_head_does_not_come() {
    let port = start_tcp(Start::H1);

    tokio::join!(
        async { assert_trickled_head_is_cut(connect_tcp(port).await).await },
        async { assert_half_a_head_is_cut(connect_tcp(port).await).await },
        async { assert_silence_is_cut(connect_tcp(port).await).await },
    );

    assert_served(&mut connect_tcp(port).await).await;
}

/// `start` on a Unix socket: `start_http_auto_unix_socket`.
#[cfg(unix)]
#[tokio::test]
async fn start_closes_a_unix_socket_connection_whose_head_does_not_come() {
    let path = UnixSocketPath::new("auto");
    start(
        MyHttpServer::new_as_unix_socket(path.0.as_str()),
        Start::Auto,
        Some(TIMEOUT),
    );

    tokio::join!(
        async { assert_trickled_head_is_cut(connect_unix(&path.0).await).await },
        async { assert_half_a_head_is_cut(connect_unix(&path.0).await).await },
        async { assert_silence_is_cut(connect_unix(&path.0).await).await },
        async { assert_half_a_preface_is_cut(connect_unix(&path.0).await).await },
    );

    assert_served(&mut connect_unix(&path.0).await).await;
}

/// `start_h1` on a Unix socket: `start_http_1_unix_socket`.
#[cfg(unix)]
#[tokio::test]
async fn start_h1_closes_a_unix_socket_connection_whose_head_does_not_come() {
    let path = UnixSocketPath::new("h1");
    start(
        MyHttpServer::new_as_unix_socket(path.0.as_str()),
        Start::H1,
        Some(TIMEOUT),
    );

    tokio::join!(
        async { assert_trickled_head_is_cut(connect_unix(&path.0).await).await },
        async { assert_half_a_head_is_cut(connect_unix(&path.0).await).await },
        async { assert_silence_is_cut(connect_unix(&path.0).await).await },
    );

    assert_served(&mut connect_unix(&path.0).await).await;
}

// ─────────────────────────────── switched off ───────────────────────────────

/// `set_header_read_timeout(None)`: the server waits for the rest of the head for as long as it
/// takes, and answers once it comes.
#[tokio::test]
async fn switched_off_the_server_waits_for_the_head() {
    for start_with in [Start::Auto, Start::H1] {
        let port = free_port();
        start(
            MyHttpServer::new(SocketAddr::from(([127, 0, 0, 1], port))),
            start_with,
            None,
        );

        let mut stream = connect_tcp(port).await;
        stream
            .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n")
            .await
            .unwrap();

        let mut buf = [0u8; 64];
        let read = tokio::time::timeout(LONGER_THAN_TIMEOUT, stream.read(&mut buf)).await;
        assert!(read.is_err(), "the server did not wait: {read:?}");

        stream.write_all(b"\r\n").await.unwrap();
        let response = read_response(&mut stream).await;
        assert!(response.starts_with("HTTP/1.1 200"), "response: {response}");
    }
}

// ─────────────────────────────── keep-alive ───────────────────────────────

/// The timeout starts over for every request: a keep-alive connection whose requests keep coming
/// is served for longer than the timeout.
#[tokio::test]
async fn a_keep_alive_connection_is_served_for_as_long_as_its_requests_keep_coming() {
    for start_with in [Start::Auto, Start::H1] {
        let port = start_tcp(start_with);
        let mut stream = connect_tcp(port).await;

        let opened = Instant::now();
        for _ in 0..4 {
            assert_served(&mut stream).await;
            tokio::time::sleep(Duration::from_millis(600)).await;
        }
        assert_served(&mut stream).await;
        assert!(opened.elapsed() > TIMEOUT * 2);
    }
}

/// A keep-alive connection waiting for its next request is waiting for the head of one: it is
/// closed once the timeout is up.
#[tokio::test]
async fn an_idle_keep_alive_connection_is_closed_once_the_timeout_is_up() {
    for start_with in [Start::Auto, Start::H1] {
        let port = start_tcp(start_with);
        let mut stream = connect_tcp(port).await;

        assert_served(&mut stream).await;
        let answered = Instant::now();

        let received = read_until_closed(&mut stream, CLOSED_WITHIN).await;
        assert_closed_by_the_timeout(answered, &received);
    }
}

// ─────────────────────────── past the head ───────────────────────────

/// A handler that takes longer than the timeout - long polling - is not cut off.
#[tokio::test]
async fn a_slow_handler_is_not_cut_off() {
    for start_with in [Start::Auto, Start::H1] {
        let port = start_tcp(start_with);
        let mut stream = connect_tcp(port).await;

        stream
            .write_all(b"GET /slow HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .await
            .unwrap();
        let response = read_response(&mut stream).await;
        assert!(response.starts_with("HTTP/1.1 200"), "response: {response}");
        assert!(response.ends_with("slow ok"), "response: {response}");
    }
}

/// A body that takes longer than the timeout to arrive is not cut off.
#[tokio::test]
async fn a_slow_body_is_not_cut_off() {
    for start_with in [Start::Auto, Start::H1] {
        let port = start_tcp(start_with);
        let mut stream = connect_tcp(port).await;

        stream
            .write_all(b"POST /echo HTTP/1.1\r\nHost: localhost\r\nContent-Length: 5\r\n\r\n")
            .await
            .unwrap();
        for byte in b"hello" {
            tokio::time::sleep(Duration::from_millis(500)).await;
            stream.write_all(&[*byte]).await.unwrap();
        }

        let response = read_response(&mut stream).await;
        assert!(response.starts_with("HTTP/1.1 200"), "response: {response}");
        assert!(response.ends_with("echo hello"), "response: {response}");
    }
}

/// A streamed response that takes longer than the timeout is not cut off.
#[tokio::test]
async fn a_long_streamed_response_is_not_cut_off() {
    for start_with in [Start::Auto, Start::H1] {
        let port = start_tcp(start_with);
        let mut stream = connect_tcp(port).await;

        stream
            .write_all(b"GET /stream HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .await
            .unwrap();
        let response = read_response(&mut stream).await;
        assert!(response.starts_with("HTTP/1.1 200"), "response: {response}");
        for i in 0..STREAM_CHUNKS {
            assert!(response.contains(&format!("chunk{i};")), "response: {response}");
        }
    }
}

/// An upgraded connection - a websocket, say - is no longer read by hyper, and the timeout is not
/// there any more: it stays open while quiet for longer than the timeout.
#[tokio::test]
async fn an_upgraded_connection_is_not_cut_off() {
    for start_with in [Start::Auto, Start::H1] {
        let port = start_tcp(start_with);
        let mut stream = connect_tcp(port).await;

        stream
            .write_all(
                b"GET /upgrade HTTP/1.1\r\nHost: localhost\r\nConnection: upgrade\r\nUpgrade: echo\r\n\r\n",
            )
            .await
            .unwrap();
        let response = read_response(&mut stream).await;
        assert!(response.starts_with("HTTP/1.1 101"), "response: {response}");

        tokio::time::sleep(LONGER_THAN_TIMEOUT).await;

        stream.write_all(b"ping").await.unwrap();
        let mut buf = [0u8; 4];
        tokio::time::timeout(Duration::from_secs(5), stream.read_exact(&mut buf))
            .await
            .expect("no echo within 5s")
            .unwrap();
        assert_eq!(&buf, b"ping");
    }
}

// ─────────────────────────────── HTTP/2 ───────────────────────────────

/// hyper has no header read timeout for HTTP/2: an HTTP/2 connection `start` took is not closed
/// while quiet for longer than the timeout, and serves a request after it.
#[tokio::test]
async fn an_http2_connection_is_not_covered() {
    let port = start_tcp(Start::Auto);

    let tcp = connect_tcp(port).await;
    let (mut client, connection) = h2::client::handshake(tcp).await.unwrap();
    let connection = tokio::spawn(async move {
        let _ = connection.await;
    });

    tokio::time::sleep(LONGER_THAN_TIMEOUT).await;

    let request = http::Request::get("http://localhost/").body(()).unwrap();
    let (response, _) = client.send_request(request, true).unwrap();
    let response = tokio::time::timeout(Duration::from_secs(5), response)
        .await
        .expect("no answer within 5s")
        .unwrap();
    assert_eq!(response.status(), 200);

    drop(client);
    connection.abort();
}
