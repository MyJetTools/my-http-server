//! End-to-end coverage of a **compressed streamed response** against a real server: an action
//! that streams with `HttpOutput::as_stream` and asks for `with_compression(&ctx.request, level)`
//! keeps sending plain bytes, and the client gets them gzipped, chunk by chunk.
//!
//! What has to hold on the wire:
//! * the response is chunked and `Content-Encoding: gzip`, and the whole body decodes - the gzip
//!   footer is there although the producer only ever dropped its end of the channel;
//! * a chunk reaches the client decodable *before* the producer has made the next one - it is a
//!   stream, not a response held back until the end;
//! * a client that does not take gzip (no `Accept-Encoding`, or `gzip;q=0`) gets the stream as
//!   produced, and either way the response says `Vary: Accept-Encoding`;
//! * a stream the action encoded itself is not compressed a second time, and a `Content-Length`
//!   the action set is dropped once the bytes on the wire are no longer the ones it counted.

use std::collections::HashMap;
use std::io::Read;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use my_http_server::controllers::ControllersMiddleware;
use my_http_server::MyHttpServer;
use rust_extensions::{AppStates, Logger};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const PARTS: [&str; 3] = ["first-part;", "second-part;", "third-part"];

/// Lets `/stream-held` make its second chunk - set by the one test that uses that route, once it
/// has decoded the first.
static RELEASE_HELD_STREAM: tokio::sync::Notify = tokio::sync::Notify::const_new();

// ─────────────────────────────── actions ───────────────────────────────

/// Streams `PARTS`, asking for compression.
pub mod stream {
    use my_http_server::macros::*;
    use my_http_server::*;

    #[http_route(
        method: "GET",
        route: "/stream",
        controller: "Test",
        summary: "Stream",
        description: "Streams the parts, compressed when the client takes it",
        result: [
            { status_code: 200, description: "Ok" },
        ]
    )]
    pub struct StreamAction;

    async fn handle_request(
        _action: &StreamAction,
        ctx: &mut HttpContext,
    ) -> Result<HttpOkResult, HttpFailResult> {
        let (output, mut producer) = HttpOutput::as_stream(10);

        tokio::spawn(async move {
            for part in super::PARTS {
                producer.send(part.as_bytes().to_vec()).await.unwrap();
            }
        });

        output
            .with_header("Content-Type", "text/plain")
            .with_compression(&ctx.request, CompressionLevel::Balanced)
            .get_result()
    }
}

/// Streams the first part, then waits for the test to let it make the rest.
pub mod stream_held {
    use my_http_server::macros::*;
    use my_http_server::*;

    #[http_route(
        method: "GET",
        route: "/stream-held",
        controller: "Test",
        summary: "Stream held",
        description: "Streams the first part and holds the rest back",
        result: [
            { status_code: 200, description: "Ok" },
        ]
    )]
    pub struct StreamHeldAction;

    async fn handle_request(
        _action: &StreamHeldAction,
        ctx: &mut HttpContext,
    ) -> Result<HttpOkResult, HttpFailResult> {
        let (output, mut producer) = HttpOutput::as_stream(10);

        tokio::spawn(async move {
            producer
                .send(super::PARTS[0].as_bytes().to_vec())
                .await
                .unwrap();

            super::RELEASE_HELD_STREAM.notified().await;

            for part in &super::PARTS[1..] {
                producer.send(part.as_bytes().to_vec()).await.unwrap();
            }
        });

        output
            .with_compression(&ctx.request, CompressionLevel::Balanced)
            .get_result()
    }
}

/// Gzips `PARTS` itself and says so - and still asks for compression.
pub mod stream_own_encoding {
    use std::io::Write;

    use my_http_server::macros::*;
    use my_http_server::*;

    #[http_route(
        method: "GET",
        route: "/stream-own-encoding",
        controller: "Test",
        summary: "Stream own encoding",
        description: "Streams a body the action gzipped itself",
        result: [
            { status_code: 200, description: "Ok" },
        ]
    )]
    pub struct StreamOwnEncodingAction;

    async fn handle_request(
        _action: &StreamOwnEncodingAction,
        ctx: &mut HttpContext,
    ) -> Result<HttpOkResult, HttpFailResult> {
        let (output, mut producer) = HttpOutput::as_stream(10);

        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(super::PARTS.concat().as_bytes()).unwrap();
        let gzipped = encoder.finish().unwrap();

        tokio::spawn(async move {
            producer.send(gzipped).await.unwrap();
        });

        output
            .with_header("Content-Encoding", "gzip")
            .with_compression(&ctx.request, CompressionLevel::Balanced)
            .get_result()
    }
}

/// Streams `PARTS` with the `Content-Length` of what it produces.
pub mod stream_with_length {
    use my_http_server::macros::*;
    use my_http_server::*;

    #[http_route(
        method: "GET",
        route: "/stream-with-length",
        controller: "Test",
        summary: "Stream with length",
        description: "Streams the parts, announcing their length",
        result: [
            { status_code: 200, description: "Ok" },
        ]
    )]
    pub struct StreamWithLengthAction;

    async fn handle_request(
        _action: &StreamWithLengthAction,
        ctx: &mut HttpContext,
    ) -> Result<HttpOkResult, HttpFailResult> {
        let (output, mut producer) = HttpOutput::as_stream(10);

        tokio::spawn(async move {
            for part in super::PARTS {
                producer.send(part.as_bytes().to_vec()).await.unwrap();
            }
        });

        output
            .with_header("Content-Length", super::PARTS.concat().len().to_string())
            .with_compression(&ctx.request, CompressionLevel::Balanced)
            .get_result()
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

async fn start_server(h2: bool) -> u16 {
    let port = free_port();

    let mut controllers = ControllersMiddleware::new(None, None);
    controllers.register_get_action(Arc::new(stream::StreamAction));
    controllers.register_get_action(Arc::new(stream_held::StreamHeldAction));
    controllers.register_get_action(Arc::new(stream_own_encoding::StreamOwnEncodingAction));
    controllers.register_get_action(Arc::new(stream_with_length::StreamWithLengthAction));

    let mut server = MyHttpServer::new(SocketAddr::from(([127, 0, 0, 1], port)));
    server.add_middleware(Arc::new(controllers));

    let app_states = Arc::new(AppStates::create_initialized());
    if h2 {
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

struct Response {
    head: String,
    /// The payload of every chunk, in order.
    chunks: Vec<Vec<u8>>,
}

impl Response {
    fn header(&self, name: &str) -> Option<&str> {
        self.head.lines().skip(1).find_map(|line| {
            let (key, value) = line.split_once(':')?;
            key.trim().eq_ignore_ascii_case(name).then(|| value.trim())
        })
    }

    fn header_count(&self, name: &str) -> usize {
        self.head
            .lines()
            .skip(1)
            .filter(|line| {
                line.split_once(':')
                    .map_or(false, |(key, _)| key.trim().eq_ignore_ascii_case(name))
            })
            .count()
    }

    fn body(&self) -> Vec<u8> {
        self.chunks.concat()
    }
}

async fn send_get(port: u16, route: &str, accept_encoding: Option<&str>) -> TcpStream {
    let accept_encoding = match accept_encoding {
        Some(value) => format!("Accept-Encoding: {}\r\n", value),
        None => String::new(),
    };

    let request = format!(
        "GET {} HTTP/1.1\r\nHost: localhost\r\n{}Connection: close\r\n\r\n",
        route, accept_encoding
    );

    let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    stream.write_all(request.as_bytes()).await.unwrap();
    stream.flush().await.unwrap();
    stream
}

async fn get(port: u16, route: &str, accept_encoding: Option<&str>) -> Response {
    let mut stream = send_get(port, route, accept_encoding).await;

    let mut buf = Vec::new();
    tokio::time::timeout(Duration::from_secs(10), stream.read_to_end(&mut buf))
        .await
        .expect("the response never ended")
        .unwrap();

    let (head, chunks) = parse_response(&buf).expect("the response is incomplete");
    Response { head, chunks }
}

/// The head and the complete chunks in `buf`, or `None` while the head is not all there yet.
fn parse_response(buf: &[u8]) -> Option<(String, Vec<Vec<u8>>)> {
    let head_end = buf.windows(4).position(|w| w == b"\r\n\r\n")?;
    let head = String::from_utf8_lossy(&buf[..head_end]).to_string();
    Some((head, dechunk(&buf[head_end + 4..])))
}

/// The payload of every complete chunk in `body`; stops at the last chunk or at a cut-off one.
fn dechunk(mut body: &[u8]) -> Vec<Vec<u8>> {
    let mut chunks = Vec::new();

    while let Some(line_end) = body.windows(2).position(|w| w == b"\r\n") {
        let size = std::str::from_utf8(&body[..line_end]).unwrap().trim();
        let size = usize::from_str_radix(size, 16).unwrap();
        if size == 0 || body.len() < line_end + 2 + size + 2 {
            break;
        }

        let payload = &body[line_end + 2..];
        chunks.push(payload[..size].to_vec());
        body = &payload[size + 2..];
    }

    chunks
}

/// What a client decodes from `gzip` - as much as there is, even if it is cut short.
fn gunzip(gzip: &[u8]) -> (Vec<u8>, std::io::Result<usize>) {
    let mut out = Vec::new();
    let result = flate2::read::GzDecoder::new(gzip).read_to_end(&mut out);
    (out, result)
}

fn assert_gunzips_to_parts(body: &[u8]) {
    let (decoded, result) = gunzip(body);
    assert!(
        result.is_ok(),
        "the body does not decode in full: {:?}",
        result
    );
    assert_eq!(String::from_utf8_lossy(&decoded), PARTS.concat());
}

// ─────────────────────────────── tests ───────────────────────────────

#[tokio::test]
async fn a_stream_is_gzipped_when_the_client_takes_gzip() {
    let port = start_server(false).await;

    let response = get(port, "/stream", Some("gzip, deflate, br")).await;

    assert!(
        response.head.starts_with("HTTP/1.1 200"),
        "{}",
        response.head
    );
    assert_eq!(response.header("Transfer-Encoding"), Some("chunked"));
    assert_eq!(response.header("Content-Encoding"), Some("gzip"));
    assert_eq!(response.header("Vary"), Some("Accept-Encoding"));
    assert_eq!(response.header("Content-Type"), Some("text/plain"));

    // A chunk per part produced, plus the gzip footer.
    assert_eq!(response.chunks.len(), PARTS.len() + 1);
    assert_gunzips_to_parts(&response.body());
}

#[tokio::test]
async fn a_chunk_reaches_the_client_decodable_before_the_next_one_is_produced() {
    let port = start_server(false).await;

    let mut stream = send_get(port, "/stream-held", Some("gzip")).await;

    // The producer is parked after its first chunk, so whatever arrives now is all there is.
    let mut buf = Vec::new();
    let first_chunk = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let mut piece = [0u8; 1024];
            let read = stream.read(&mut piece).await.unwrap();
            assert!(read > 0, "the connection closed before the first chunk");
            buf.extend_from_slice(&piece[..read]);

            if let Some((_, chunks)) = parse_response(&buf) {
                if let Some(first) = chunks.into_iter().next() {
                    return first;
                }
            }
        }
    })
    .await
    .expect("the first chunk never arrived on its own");

    let (decoded, _) = gunzip(&first_chunk);
    assert_eq!(String::from_utf8_lossy(&decoded), PARTS[0]);

    RELEASE_HELD_STREAM.notify_one();

    tokio::time::timeout(Duration::from_secs(10), stream.read_to_end(&mut buf))
        .await
        .expect("the response never ended")
        .unwrap();

    let (head, chunks) = parse_response(&buf).unwrap();
    assert!(head.contains("content-encoding: gzip"), "{}", head);
    assert_gunzips_to_parts(&chunks.concat());
}

#[tokio::test]
async fn a_client_without_accept_encoding_gets_the_stream_as_produced() {
    let port = start_server(false).await;

    let response = get(port, "/stream", None).await;

    assert!(
        response.head.starts_with("HTTP/1.1 200"),
        "{}",
        response.head
    );
    assert_eq!(response.header("Content-Encoding"), None);
    assert_eq!(response.header("Vary"), Some("Accept-Encoding"));
    assert_eq!(String::from_utf8_lossy(&response.body()), PARTS.concat());
}

#[tokio::test]
async fn a_client_that_refuses_gzip_gets_the_stream_as_produced() {
    let port = start_server(false).await;

    let response = get(port, "/stream", Some("br, gzip;q=0")).await;

    assert_eq!(response.header("Content-Encoding"), None);
    assert_eq!(String::from_utf8_lossy(&response.body()), PARTS.concat());
}

#[tokio::test]
async fn a_stream_the_action_encoded_itself_is_not_compressed_again() {
    let port = start_server(false).await;

    let response = get(port, "/stream-own-encoding", Some("gzip")).await;

    assert_eq!(response.header_count("Content-Encoding"), 1);
    assert_eq!(response.header("Content-Encoding"), Some("gzip"));
    assert_eq!(response.header("Vary"), None);
    // One layer of gzip, not two.
    assert_gunzips_to_parts(&response.body());
}

#[tokio::test]
async fn a_content_length_set_by_the_action_is_dropped_once_compressed() {
    let port = start_server(false).await;

    let response = get(port, "/stream-with-length", Some("gzip")).await;

    assert_eq!(response.header("Content-Length"), None);
    assert_eq!(response.header("Transfer-Encoding"), Some("chunked"));
    assert_eq!(response.header("Content-Encoding"), Some("gzip"));
    assert_gunzips_to_parts(&response.body());
}

/// HTTP/2 has no chunked encoding: the compressed chunks travel as DATA frames, and the footer
/// still arrives before the end of the stream.
#[tokio::test]
async fn an_h2_stream_is_gzipped_too() {
    let port = start_server(true).await;

    let tcp = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    let (mut client, connection) = h2::client::handshake(tcp).await.unwrap();
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let request = http::Request::builder()
        .method("GET")
        .uri("http://localhost/stream")
        .header("accept-encoding", "gzip")
        .body(())
        .unwrap();

    let (response, _) = client.send_request(request, true).unwrap();
    let response = tokio::time::timeout(Duration::from_secs(10), response)
        .await
        .expect("no response")
        .unwrap();

    assert_eq!(response.status(), 200);
    assert_eq!(response.headers().get("content-encoding").unwrap(), "gzip");
    assert_eq!(response.headers().get("vary").unwrap(), "Accept-Encoding");

    let mut body = response.into_body();
    let mut received = Vec::new();
    while let Some(data) = tokio::time::timeout(Duration::from_secs(10), body.data())
        .await
        .expect("the stream never ended")
    {
        let data = data.unwrap();
        let _ = body.flow_control().release_capacity(data.len());
        received.extend_from_slice(&data);
    }

    assert_gunzips_to_parts(&received);
}
