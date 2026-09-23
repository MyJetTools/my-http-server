//! End-to-end coverage of a **compressed request body** against a real server over a raw TCP
//! socket: a client that says `Content-Encoding: gzip` (or `zstd`) must reach an action with the
//! body already decoded, whichever way the action takes it.
//!
//! The boundary is part of the contract and is tested too: a **streamed** body is handed on
//! exactly as it arrived, because a pass-through action wants the bytes the client sent, and
//! decoding a stream frame by frame is not something this server does.

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
    let port = free_port();

    let mut controllers = ControllersMiddleware::new(None, None);
    controllers.register_post_action(Arc::new(echo_json::EchoJsonAction));
    controllers.register_post_action(Arc::new(echo_raw::EchoRawAction));
    controllers.register_post_action(Arc::new(stream_through::StreamThroughAction));

    let mut server = MyHttpServer::new(SocketAddr::from(([127, 0, 0, 1], port)));
    server.add_middleware(Arc::new(controllers));

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
