//! End-to-end coverage of **protobuf in the body** against a real server over a raw TCP socket:
//! an `application/x-protobuf` request reaches a `#[http_body_raw]` field byte for byte - the
//! content type is not one the server parses, so the bytes are handed on as they are - and a
//! response built with `WebContentType::Protobuf` goes back with that content type, gzipped at
//! the level the action asked for.

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

// ─────────────────────────────── actions ───────────────────────────────

/// Echoes the protobuf body back as protobuf, compressed past 1 KiB.
pub mod echo_protobuf {
    use my_http_server::macros::*;
    use my_http_server::*;

    #[derive(MyHttpInput)]
    pub struct EchoProtobufHttpInput {
        #[http_body_raw(description = "Protobuf message")]
        pub body: RawData,
    }

    #[http_route(
        method: "POST",
        route: "/echo-protobuf",
        controller: "Test",
        summary: "Echo protobuf",
        description: "Echoes the protobuf body",
        input_data: "EchoProtobufHttpInput",
        result: [
            { status_code: 200, description: "Ok" },
        ]
    )]
    pub struct EchoProtobufAction;

    async fn handle_request(
        _action: &EchoProtobufAction,
        input_data: EchoProtobufHttpInput,
        _ctx: &mut HttpContext,
    ) -> Result<HttpOkResult, HttpFailResult> {
        HttpOutput::from_builder()
            .set_content(input_data.body.into_vec())
            .set_content_type(WebContentType::Protobuf)
            .with_compression(1024, CompressionLevel::Fast)
            .into_ok_result(true)
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

async fn start_server() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let mut controllers = ControllersMiddleware::new(None, None);
    controllers.register_post_action(Arc::new(echo_protobuf::EchoProtobufAction));

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

/// Posts `body` as protobuf; returns the response head and its body bytes.
async fn post_protobuf(port: u16, body: &[u8]) -> (String, Vec<u8>) {
    let mut request = format!(
        "POST /echo-protobuf HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/x-protobuf\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .into_bytes();
    request.extend_from_slice(body);

    let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    stream.write_all(&request).await.unwrap();
    stream.flush().await.unwrap();

    let mut buf = Vec::new();
    tokio::time::timeout(Duration::from_secs(10), stream.read_to_end(&mut buf))
        .await
        .expect("the response never ended")
        .unwrap();

    let head_end = buf.windows(4).position(|w| w == b"\r\n\r\n").unwrap();
    let head = String::from_utf8_lossy(&buf[..head_end]).to_string();
    (head, buf[head_end + 4..].to_vec())
}

fn header<'h>(head: &'h str, name: &str) -> Option<&'h str> {
    head.lines().skip(1).find_map(|line| {
        let (key, value) = line.split_once(':')?;
        key.trim().eq_ignore_ascii_case(name).then(|| value.trim())
    })
}

/// A protobuf message, encoded by hand: `1: varint 150`, `2: string "testing"`,
/// `3: bytes [00 ff 7b 0d 0a]` - zeros, high bytes, a `{` and a CRLF, none of which may be touched
/// on the way in or out.
const MESSAGE: &[u8] = &[
    0x08, 0x96, 0x01, 0x12, 0x07, b't', b'e', b's', b't', b'i', b'n', b'g', 0x1a, 0x05, 0x00, 0xff,
    0x7b, 0x0d, 0x0a,
];

// ─────────────────────────────── tests ───────────────────────────────

#[tokio::test]
async fn a_protobuf_body_goes_in_and_out_byte_for_byte() {
    let port = start_server().await;

    let (head, body) = post_protobuf(port, MESSAGE).await;

    assert!(head.starts_with("HTTP/1.1 200"), "{}", head);
    assert_eq!(
        header(&head, "Content-Type"),
        Some("application/x-protobuf")
    );
    // Below the 1 KiB threshold: not worth compressing.
    assert_eq!(header(&head, "Content-Encoding"), None);
    assert_eq!(body, MESSAGE);
}

#[tokio::test]
async fn a_large_protobuf_response_is_gzipped() {
    let port = start_server().await;

    // A repeated field of the same message: what a list of records looks like on the wire.
    let message: Vec<u8> = (0..500)
        .flat_map(|_| {
            let mut field = vec![0x22, MESSAGE.len() as u8];
            field.extend_from_slice(MESSAGE);
            field
        })
        .collect();

    let (head, body) = post_protobuf(port, &message).await;

    assert!(head.starts_with("HTTP/1.1 200"), "{}", head);
    assert_eq!(
        header(&head, "Content-Type"),
        Some("application/x-protobuf")
    );
    assert_eq!(header(&head, "Content-Encoding"), Some("gzip"));
    assert!(body.len() < message.len());

    let mut decoded = Vec::new();
    flate2::read::GzDecoder::new(body.as_slice())
        .read_to_end(&mut decoded)
        .unwrap();
    assert_eq!(decoded, message);
}
