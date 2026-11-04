//! End-to-end coverage of the connections counter - `get_http_connections_counter()` - through
//! each of the five accept loops: `start` and `start_h1`, on TCP and on a Unix socket, and
//! `start_h2`. A connection is counted for as long as it is open, and not once it is closed.
//!
//! An accept loop counts a connection with a guard, which it moves into the connection's task so
//! that it goes with the task, however the task ends. The other tests only wait for the counter to
//! come back to zero - which it would do at once if the guard were dropped too soon, say by
//! `let _ = connection_guard;` in the task. Here the counter is read while the connection is still
//! open.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use my_http_server::*;
use rust_extensions::{AppStates, Logger};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;

// ─────────────────────────────── harness ───────────────────────────────

/// Answers every request with `200 ok`.
struct OkMiddleware;

#[my_http_server::async_trait::async_trait]
impl HttpServerMiddleware for OkMiddleware {
    async fn handle_request(
        &self,
        _ctx: &mut HttpContext,
    ) -> Option<Result<HttpOkResult, HttpFailResult>> {
        Some(HttpOutput::as_text("ok".to_string()).into_ok_result(false))
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

/// Which `start*` method the server is started with.
#[derive(Clone, Copy)]
enum Start {
    Auto,
    H1,
    H2,
}

/// Starts `server` and hands back its counter.
fn start(mut server: MyHttpServer, start: Start) -> HttpConnectionsCounter {
    server.add_middleware(Arc::new(OkMiddleware));
    let counter = server.get_http_connections_counter();

    let app_states = Arc::new(AppStates::create_initialized());
    match start {
        Start::Auto => server.start(app_states, Arc::new(SilentLogger)),
        Start::H1 => server.start_h1(app_states, Arc::new(SilentLogger)),
        Start::H2 => server.start_h2(app_states, Arc::new(SilentLogger)),
    }

    counter
}

fn tcp_server(port: u16) -> MyHttpServer {
    MyHttpServer::new(SocketAddr::from(([127, 0, 0, 1], port)))
}

fn free_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

/// Connects as soon as the server is up. No probe connection is made first: the counter would
/// count it too.
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
        let file = format!("mhs-counter-{}-{name}.sock", std::process::id());
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

/// The counter once it has come to `expected` - or, failing that, after 5 s.
async fn settled_amount(counter: &HttpConnectionsCounter, expected: i64) -> i64 {
    let mut amount = counter.get_connections_amount();
    for _ in 0..250 {
        if amount == expected {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
        amount = counter.get_connections_amount();
    }
    amount
}

/// Serves one keep-alive `GET /` over HTTP/1.1 on `stream`, then checks that the connection is
/// counted while it is open and stops being counted once it is closed.
async fn assert_counted_while_open_h1(
    counter: &HttpConnectionsCounter,
    mut stream: impl AsyncRead + AsyncWrite + Unpin,
) {
    stream
        .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n")
        .await
        .unwrap();

    let mut buf = [0u8; 256];
    let read = tokio::time::timeout(Duration::from_secs(5), stream.read(&mut buf))
        .await
        .expect("no answer within 5s")
        .unwrap();
    let response = String::from_utf8_lossy(&buf[..read]);
    assert!(response.starts_with("HTTP/1.1 200"), "response: {response}");

    // Answered and kept alive: the server is serving this connection right now.
    assert_eq!(
        counter.get_connections_amount(),
        1,
        "the open connection is not counted"
    );

    drop(stream);
    assert_eq!(
        settled_amount(counter, 0).await,
        0,
        "the closed connection is still counted"
    );
}

// ─────────────────────────────── tests ───────────────────────────────

/// `start` on TCP: `start_http_auto`.
#[tokio::test]
async fn start_counts_a_tcp_connection_while_it_is_open() {
    let port = free_port();
    let counter = start(tcp_server(port), Start::Auto);

    let stream = connect_tcp(port).await;
    assert_counted_while_open_h1(&counter, stream).await;
}

/// `start_h1` on TCP: `start_http_1`.
#[tokio::test]
async fn start_h1_counts_a_tcp_connection_while_it_is_open() {
    let port = free_port();
    let counter = start(tcp_server(port), Start::H1);

    let stream = connect_tcp(port).await;
    assert_counted_while_open_h1(&counter, stream).await;
}

/// `start_h2`: `start_http_2`.
#[tokio::test]
async fn start_h2_counts_a_connection_while_it_is_open() {
    let port = free_port();
    let counter = start(tcp_server(port), Start::H2);

    let tcp = connect_tcp(port).await;
    let (mut client, connection) = h2::client::handshake(tcp).await.unwrap();
    let connection = tokio::spawn(async move {
        let _ = connection.await;
    });

    let request = http::Request::get("http://localhost/").body(()).unwrap();
    let (response, _) = client.send_request(request, true).unwrap();
    let response = tokio::time::timeout(Duration::from_secs(5), response)
        .await
        .expect("no answer within 5s")
        .unwrap();
    assert_eq!(response.status(), 200);

    assert_eq!(
        counter.get_connections_amount(),
        1,
        "the open connection is not counted"
    );

    // The client's connection is no longer driven, and its socket goes with it.
    drop(client);
    connection.abort();
    assert_eq!(
        settled_amount(&counter, 0).await,
        0,
        "the closed connection is still counted"
    );
}

/// `start` on a Unix socket: `start_http_auto_unix_socket`.
#[cfg(unix)]
#[tokio::test]
async fn start_counts_a_unix_socket_connection_while_it_is_open() {
    let path = UnixSocketPath::new("auto");
    let counter = start(
        MyHttpServer::new_as_unix_socket(path.0.as_str()),
        Start::Auto,
    );

    let stream = connect_unix(&path.0).await;
    assert_counted_while_open_h1(&counter, stream).await;
}

/// `start_h1` on a Unix socket: `start_http_1_unix_socket`.
#[cfg(unix)]
#[tokio::test]
async fn start_h1_counts_a_unix_socket_connection_while_it_is_open() {
    let path = UnixSocketPath::new("h1");
    let counter = start(MyHttpServer::new_as_unix_socket(path.0.as_str()), Start::H1);

    let stream = connect_unix(&path.0).await;
    assert_counted_while_open_h1(&counter, stream).await;
}
