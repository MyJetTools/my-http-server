//! End-to-end coverage of a **compressed request body** against a real server over a raw TCP
//! socket: a client that says `Content-Encoding: gzip` (or `zstd`) must reach an action with the
//! body already decoded, whichever way the action takes it.
//!
//! The boundary is part of the contract and is tested too: a **streamed** body is handed on
//! exactly as it arrived, because a pass-through action wants the bytes the client sent, and
//! decoding a stream frame by frame is not something this server does.
//!
//! So is the limit on what a body may decode to: a bomb is a 413 naming the limit, a body that
//! does not decode at all is a 400, and neither touches a body that announced no encoding.

use std::collections::HashMap;
use std::io::Write;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use my_http_server::controllers::ControllersMiddleware;
use my_http_server::MyHttpServer;
use rust_extensions::{AppStates, Logger};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

// ─────────────────────────────── actions ───────────────────────────────

/// Named JSON body fields - the body has to be decoded *and* parsed as JSON.
pub mod echo_json {
    use my_http_server::macros::*;
    use my_http_server::*;

    #[derive(MyHttpInput)]
    pub struct EchoJsonHttpInput {
        #[http_body(name = "email", description = "Email")]
        pub email: String,
    }

    #[http_route(
        method: "POST",
        route: "/echo-json",
        controller: "Test",
        summary: "Echo json",
        description: "Echoes a field of the JSON body",
        input_data: "EchoJsonHttpInput",
        result: [
            { status_code: 200, description: "Ok" },
        ]
    )]
    pub struct EchoJsonAction;

    async fn handle_request(
        _action: &EchoJsonAction,
        input_data: EchoJsonHttpInput,
        _ctx: &mut HttpContext,
    ) -> Result<HttpOkResult, HttpFailResult> {
        HttpOutput::as_text(format!("email={}", input_data.email))
            .into_ok_result(true)
            .into()
    }
}

/// The whole body verbatim - no JSON parsing involved, so it shows the decoding happens before
/// anything looks at the content.
pub mod echo_raw {
    use my_http_server::macros::*;
    use my_http_server::*;

    #[derive(MyHttpInput)]
    pub struct EchoRawHttpInput {
        #[http_body_raw(description = "Body")]
        pub body: RawData,
    }

    #[http_route(
        method: "POST",
        route: "/echo-raw",
        controller: "Test",
        summary: "Echo raw",
        description: "Echoes the raw body",
        input_data: "EchoRawHttpInput",
        result: [
            { status_code: 200, description: "Ok" },
        ]
    )]
    pub struct EchoRawAction;

    async fn handle_request(
        _action: &EchoRawAction,
        input_data: EchoRawHttpInput,
        _ctx: &mut HttpContext,
    ) -> Result<HttpOkResult, HttpFailResult> {
        let body = input_data.body.as_slice();

        HttpOutput::as_text(format!(
            "raw={}:{}",
            body.len(),
            String::from_utf8_lossy(body)
        ))
        .into_ok_result(true)
        .into()
    }
}

/// A streamed body - reported by size only, which is the whole point: the size that comes back is
/// the size on the wire, compressed.
pub mod stream_through {
    use my_http_server::macros::*;
    use my_http_server::*;

    #[derive(MyHttpInput)]
    pub struct StreamHttpInput {
        #[http_body_as_stream(description = "Body")]
        pub body: HttpBodyAsStream,
    }

    #[http_route(
        method: "POST",
        route: "/stream-through",
        controller: "Test",
        summary: "Stream through",
        description: "Reports the size of the streamed body",
        input_data: "StreamHttpInput",
        result: [
            { status_code: 200, description: "Ok" },
        ]
    )]
    pub struct StreamThroughAction;

    async fn handle_request(
        _action: &StreamThroughAction,
        input_data: StreamHttpInput,
        _ctx: &mut HttpContext,
    ) -> Result<HttpOkResult, HttpFailResult> {
        let reader = input_data.body.get_body_reader()?;

        let mut total = 0usize;

        while let Some(chunk) = reader.get_next_chunk().await? {
            total += chunk.len();
        }

        HttpOutput::as_text(format!("streamed={}", total))
            .into_ok_result(true)
            .into()
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

async fn start_server() -> u16 {
    start_server_with(None).await
}

/// Same, with `set_max_decompressed_body_size` in force when a limit is given.
async fn start_server_with(max_decompressed_body_size: Option<usize>) -> u16 {
    let port = free_port();

    let mut controllers = ControllersMiddleware::new(None, None);
    controllers.register_post_action(Arc::new(echo_json::EchoJsonAction));
    controllers.register_post_action(Arc::new(echo_raw::EchoRawAction));
    controllers.register_post_action(Arc::new(stream_through::StreamThroughAction));

    let mut server = MyHttpServer::new(SocketAddr::from(([127, 0, 0, 1], port)));
    server.add_middleware(Arc::new(controllers));

    if let Some(max_decompressed_body_size) = max_decompressed_body_size {
        server.set_max_decompressed_body_size(max_decompressed_body_size);
    }

    server.start_h1(
        Arc::new(AppStates::create_initialized()),
        Arc::new(SilentLogger),
    );

    for _ in 0..100 {
        if TcpStream::connect(("127.0.0.1", port)).await.is_ok() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    port
}

/// Sends a POST whose body is `body` bytes, announcing `encoding` when one is given.
async fn post(port: u16, route: &str, encoding: Option<&str>, body: &[u8]) -> String {
    let encoding_header = match encoding {
        Some(encoding) => format!("Content-Encoding: {}\r\n", encoding),
        None => String::new(),
    };

    let mut request = format!(
        "POST {} HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\n{}Content-Length: {}\r\nConnection: close\r\n\r\n",
        route,
        encoding_header,
        body.len()
    )
    .into_bytes();
    request.extend_from_slice(body);

    let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    stream.write_all(&request).await.unwrap();
    stream.flush().await.unwrap();

    let mut buf = Vec::new();
    let _ = tokio::time::timeout(Duration::from_secs(10), stream.read_to_end(&mut buf)).await;

    String::from_utf8_lossy(&buf).to_string()
}

const BODY: &[u8] = br#"{"email":"a@b.com"}"#;

fn gzip(raw: &[u8]) -> Vec<u8> {
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    encoder.write_all(raw).unwrap();
    encoder.finish().unwrap()
}

fn zstd(raw: &[u8]) -> Vec<u8> {
    ruzstd::encoding::compress_to_vec(raw, ruzstd::encoding::CompressionLevel::Fastest)
}

/// `BODY` in standard brotli. There is no brotli encoder among the dependencies, so it is a
/// fixture, made with the `brotli` 8.0.2 crate.
const BR_BODY: &[u8] = &[
    0x0b, 0x09, 0x80, 0x7b, 0x22, 0x65, 0x6d, 0x61, 0x69, 0x6c, 0x22, 0x3a, 0x22, 0x61, 0x40, 0x62,
    0x2e, 0x63, 0x6f, 0x6d, 0x22, 0x7d, 0x03,
];

/// The same `BODY` as "Large Window Brotli" - the non-standard extension that lets a 6-byte
/// header ask the decoder for a 1 GiB window.
const BR_BODY_LARGE_WINDOW: &[u8] = &[
    0x11, 0x1e, 0x24, 0x00, 0x02, 0x7b, 0x22, 0x65, 0x6d, 0x61, 0x69, 0x6c, 0x22, 0x3a, 0x22, 0x61,
    0x40, 0x62, 0x2e, 0x63, 0x6f, 0x6d, 0x22, 0x7d, 0x03,
];

/// `deflate` on the wire is a zlib stream.
fn deflate(raw: &[u8]) -> Vec<u8> {
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::fast());
    encoder.write_all(raw).unwrap();
    encoder.finish().unwrap()
}

// ─────────────────────────────── tests ───────────────────────────────

#[tokio::test]
async fn a_gzip_body_reaches_the_action_decoded() {
    let port = start_server().await;

    let response = post(port, "/echo-json", Some("gzip"), &gzip(BODY)).await;

    assert!(response.starts_with("HTTP/1.1 200"), "response: {}", response);
    assert!(response.contains("email=a@b.com"), "response: {}", response);
}

#[tokio::test]
async fn a_zstd_body_reaches_the_action_decoded() {
    let port = start_server().await;

    let response = post(port, "/echo-json", Some("zstd"), &zstd(BODY)).await;

    assert!(response.starts_with("HTTP/1.1 200"), "response: {}", response);
    assert!(response.contains("email=a@b.com"), "response: {}", response);
}

#[tokio::test]
async fn a_deflate_body_reaches_the_action_decoded() {
    let port = start_server().await;

    let response = post(port, "/echo-json", Some("deflate"), &deflate(BODY)).await;

    assert!(response.starts_with("HTTP/1.1 200"), "response: {}", response);
    assert!(response.contains("email=a@b.com"), "response: {}", response);
}

#[tokio::test]
async fn a_br_body_reaches_the_action_decoded() {
    let port = start_server().await;

    let response = post(port, "/echo-json", Some("br"), BR_BODY).await;

    assert!(response.starts_with("HTTP/1.1 200"), "response: {}", response);
    assert!(response.contains("email=a@b.com"), "response: {}", response);
}

/// "Large Window Brotli" is not the `br` content coding, and it is refused before the decoder can
/// size a window from its header - so the same body that decodes as standard brotli is a 400.
#[tokio::test]
async fn a_large_window_brotli_body_is_a_400() {
    let port = start_server().await;

    let response = post(port, "/echo-json", Some("br"), BR_BODY_LARGE_WINDOW).await;

    assert!(response.starts_with("HTTP/1.1 400"), "response: {}", response);
}

/// `#[http_body_raw]` never parses anything - so this is decoding, not a JSON reader being clever.
#[tokio::test]
async fn a_compressed_raw_body_is_decoded_too() {
    let port = start_server().await;

    let response = post(port, "/echo-raw", Some("gzip"), &gzip(BODY)).await;

    assert!(response.starts_with("HTTP/1.1 200"), "response: {}", response);
    assert!(
        response.contains(&format!("raw={}:{}", BODY.len(), String::from_utf8_lossy(BODY))),
        "response: {}",
        response
    );
}

/// The header a client sends when it compresses nothing. It is not an encoding we fail on.
#[tokio::test]
async fn an_identity_body_is_taken_as_it_is() {
    let port = start_server().await;

    let response = post(port, "/echo-json", Some("identity"), BODY).await;

    assert!(response.starts_with("HTTP/1.1 200"), "response: {}", response);
    assert!(response.contains("email=a@b.com"), "response: {}", response);
}

/// Nothing changes for the overwhelming majority of requests, which announce no encoding at all.
#[tokio::test]
async fn a_plain_body_is_untouched() {
    let port = start_server().await;

    let response = post(port, "/echo-json", None, BODY).await;

    assert!(response.starts_with("HTTP/1.1 200"), "response: {}", response);
    assert!(response.contains("email=a@b.com"), "response: {}", response);
}

/// An encoding we can not undo is a 400 naming it - not a body handed on as if it were plain.
#[tokio::test]
async fn an_encoding_we_can_not_undo_is_a_400() {
    let port = start_server().await;

    let response = post(port, "/echo-json", Some("compress"), BODY).await;

    assert!(response.starts_with("HTTP/1.1 400"), "response: {}", response);
    assert!(response.contains("compress"), "response: {}", response);
}

/// The boundary: a streamed body is passed through as it arrived, so what the action sees is the
/// **compressed** length. Decoding it is the action's business - it is the one proxying it on.
#[tokio::test]
async fn a_streamed_body_is_handed_on_still_compressed() {
    let port = start_server().await;

    let compressed = gzip(BODY);
    assert_ne!(compressed.len(), BODY.len());

    let response = post(port, "/stream-through", Some("gzip"), &compressed).await;

    assert!(response.starts_with("HTTP/1.1 200"), "response: {}", response);
    assert!(
        response.contains(&format!("streamed={}", compressed.len())),
        "response: {}",
        response
    );
}

/// A header that lies about a body that is no encoding at all is the client's mistake: 400, not
/// the 500 of a server fault.
#[tokio::test]
async fn a_body_that_does_not_decode_is_a_400() {
    let port = start_server().await;

    let response = post(
        port,
        "/echo-json",
        Some("gzip"),
        b"this is not a compressed body",
    )
    .await;

    assert!(response.starts_with("HTTP/1.1 400"), "response: {}", response);
    assert!(response.contains("decompress"), "response: {}", response);
}

/// A limit small enough to trip over in a test, and a bomb that inflates far past it from a few
/// kilobytes on the wire.
const SMALL_LIMIT: usize = 64 * 1024;
const BOMB_SIZE: usize = 8 * 1024 * 1024;

#[tokio::test]
async fn a_decompression_bomb_is_a_413_naming_the_limit() {
    let port = start_server_with(Some(SMALL_LIMIT)).await;

    let bomb = gzip(&vec![0u8; BOMB_SIZE]);
    assert!(bomb.len() < SMALL_LIMIT);

    let response = post(port, "/echo-raw", Some("gzip"), &bomb).await;

    assert!(response.starts_with("HTTP/1.1 413"), "response: {}", response);
    assert!(
        response.contains(&SMALL_LIMIT.to_string()),
        "response: {}",
        response
    );
}

/// The limit is inclusive: a body that decodes to exactly the limit reaches the action, one byte
/// more does not.
#[tokio::test]
async fn a_body_exactly_at_the_limit_reaches_the_action() {
    let port = start_server_with(Some(SMALL_LIMIT)).await;

    let at_the_limit = vec![b'a'; SMALL_LIMIT];
    let response = post(port, "/echo-raw", Some("gzip"), &gzip(&at_the_limit)).await;

    assert!(
        response.starts_with("HTTP/1.1 200"),
        "status line: {}",
        response.lines().next().unwrap_or_default()
    );
    assert!(
        response.contains(&format!("raw={}:", SMALL_LIMIT)),
        "response starts: {}",
        &response[..response.len().min(300)]
    );

    let one_byte_more = vec![b'a'; SMALL_LIMIT + 1];
    let response = post(port, "/echo-raw", Some("gzip"), &gzip(&one_byte_more)).await;

    assert!(response.starts_with("HTTP/1.1 413"), "response: {}", response);
}

/// The limit bounds what a decoder produces, nothing else: a plain body larger than it is taken
/// as it is.
#[tokio::test]
async fn a_plain_body_is_not_held_to_the_decompression_limit() {
    let port = start_server_with(Some(SMALL_LIMIT)).await;

    let body = vec![b'a'; SMALL_LIMIT * 2];
    let response = post(port, "/echo-raw", None, &body).await;

    assert!(
        response.starts_with("HTTP/1.1 200"),
        "status line: {}",
        response.lines().next().unwrap_or_default()
    );
    assert!(
        response.contains(&format!("raw={}:", body.len())),
        "response starts: {}",
        &response[..response.len().min(300)]
    );
}

/// Nor is a streamed body, which this server never decodes: the bomb reaches the action as the
/// few kilobytes it is on the wire.
#[tokio::test]
async fn a_streamed_bomb_is_handed_on_as_it_is() {
    let port = start_server_with(Some(SMALL_LIMIT)).await;

    let bomb = gzip(&vec![0u8; BOMB_SIZE]);
    let response = post(port, "/stream-through", Some("gzip"), &bomb).await;

    assert!(response.starts_with("HTTP/1.1 200"), "response: {}", response);
    assert!(
        response.contains(&format!("streamed={}", bomb.len())),
        "response: {}",
        response
    );
}
