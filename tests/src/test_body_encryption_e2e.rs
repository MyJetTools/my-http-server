//! End-to-end coverage of **encrypted bodies** (`Content-Encoding: aes128gcm`, RFC 8188) against
//! a real server over a raw TCP socket.
//!
//! A request body announced as encrypted reaches the action decrypted, opened by whichever of the
//! server's `BodyEncryptionKeys` fits, whichever way the action takes the body. One that can not be
//! opened never reaches it: `415` with `Accept-Encoding` while the server has no key, `400` when
//! no key opens it. Only the header makes a body encrypted: one without it is taken as it is,
//! and a streamed body is handed on as it arrived, like a compressed one.
//!
//! The request tells the action how the body came: `get_encryption_info` gives the key that opened
//! it and its salt. The other way is the action's: `HttpResultBuilder::encode`, with a key it
//! takes by id or with the key the request came with.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use my_http_server::controllers::ControllersMiddleware;
use my_http_server::{
    aes128gcm, BodyEncryptionKey, BodyEncryptionKeys, HttpContext, HttpFailResult, HttpOkResult,
    HttpOutput, HttpServerMiddleware, MyHttpServer,
};
use rust_extensions::{AppStates, Logger};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

// ─────────────────────────────── actions ───────────────────────────────

/// Named JSON body fields - the body has to be decrypted *and* parsed as JSON.
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
        HttpOutput::as_text(format!("email={}", input_data.email)).into_ok_result(true)
    }
}

/// The whole body verbatim: what the action got, byte for byte.
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
    }
}

/// A streamed body, reported by size only: the size on the wire, still encrypted.
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

        HttpOutput::as_text(format!("streamed={}", total)).into_ok_result(true)
    }
}

/// Reads from the request how the body came encrypted, and answers encrypted with the key the
/// client used. It takes encrypted bodies only: a plain one is refused.
pub mod answer_with_request_key {
    use my_http_server::macros::*;
    use my_http_server::*;

    #[derive(MyHttpInput)]
    pub struct AnswerHttpInput {
        #[http_body(name = "email", description = "Email")]
        pub email: String,
    }

    #[http_route(
        method: "POST",
        route: "/answer-with-request-key",
        controller: "Test",
        summary: "Answer with the request key",
        description: "Answers encrypted with the key the request came encrypted with",
        input_data: "AnswerHttpInput",
        result: [
            { status_code: 200, description: "Ok" },
        ]
    )]
    pub struct AnswerWithRequestKeyAction;

    async fn handle_request(
        _action: &AnswerWithRequestKeyAction,
        input_data: AnswerHttpInput,
        ctx: &mut HttpContext,
    ) -> Result<HttpOkResult, HttpFailResult> {
        let Some(info) = ctx.request.get_encryption_info() else {
            return Err(HttpFailResult::from((
                415u16,
                "The body has to come encrypted".to_string(),
            )));
        };

        HttpOutput::as_text(format!(
            "email={} key={} rs={} salt={}",
            input_data.email,
            info.key.get_id(),
            info.record_size,
            super::hex(&info.salt)
        ))
        .encode(&info.key)
        .into_ok_result(true)
    }
}

/// `GET /encoded-reply`: answers encrypted with the key it takes from the list by id - the way an
/// action that serves my-reverse-proxy does.
struct EncodedReplyMiddleware {
    keys: Arc<BodyEncryptionKeys>,
}

#[my_http_server::async_trait::async_trait]
impl HttpServerMiddleware for EncodedReplyMiddleware {
    async fn handle_request(
        &self,
        ctx: &mut HttpContext,
    ) -> Option<Result<HttpOkResult, HttpFailResult>> {
        if ctx.request.http_path.as_str() != "/encoded-reply" {
            return None;
        }

        let key = self.keys.get_by_id(REPLY_KEY_ID)?;

        Some(
            HttpOutput::as_text(REPLY)
                .encode(&key)
                .into_ok_result(false),
        )
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

/// A server with `keys` handed to `set_body_encryption_keys`, or none at all.
async fn start_server(keys: Option<Arc<BodyEncryptionKeys>>) -> u16 {
    let port = free_port();

    let mut controllers = ControllersMiddleware::new(None, None);
    controllers.register_post_action(Arc::new(echo_json::EchoJsonAction));
    controllers.register_post_action(Arc::new(echo_raw::EchoRawAction));
    controllers.register_post_action(Arc::new(stream_through::StreamThroughAction));
    controllers.register_post_action(Arc::new(
        answer_with_request_key::AnswerWithRequestKeyAction,
    ));

    let mut server = MyHttpServer::new(SocketAddr::from(([127, 0, 0, 1], port)));

    if let Some(keys) = keys {
        server.add_middleware(Arc::new(EncodedReplyMiddleware { keys: keys.clone() }));
        server.set_body_encryption_keys(keys);
    }

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

/// The head of a response as text, and its body as bytes.
struct Response {
    head: String,
    body: Vec<u8>,
}

impl Response {
    fn status_is(&self, status: u16) -> bool {
        self.head.starts_with(&format!("HTTP/1.1 {}", status))
    }

    /// The value of the header `name`, matched case-insensitively.
    fn header(&self, name: &str) -> Option<&str> {
        self.head.lines().skip(1).find_map(|line| {
            let (key, value) = line.split_once(':')?;
            key.trim().eq_ignore_ascii_case(name).then(|| value.trim())
        })
    }

    fn body_text(&self) -> String {
        String::from_utf8_lossy(&self.body).to_string()
    }
}

impl std::fmt::Debug for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n\n{}", self.head, self.body_text())
    }
}

async fn send(port: u16, request: &[u8]) -> Response {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    stream.write_all(request).await.unwrap();
    stream.flush().await.unwrap();

    let mut buf = Vec::new();
    let _ = tokio::time::timeout(Duration::from_secs(10), stream.read_to_end(&mut buf)).await;

    let split = buf
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .unwrap_or_else(|| panic!("no response head in: {:?}", String::from_utf8_lossy(&buf)));

    Response {
        head: String::from_utf8_lossy(&buf[..split]).to_string(),
        body: buf[split + 4..].to_vec(),
    }
}

/// A POST whose body is `body`, announcing `encoding` when one is given.
async fn post(port: u16, route: &str, encoding: Option<&str>, body: &[u8]) -> Response {
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

    send(port, &request).await
}

const BODY: &[u8] = br#"{"email":"a@b.com"}"#;

const REPLY_KEY_ID: &str = "reply";
const REPLY: &str = "ip_white_lists:\n  office:\n  - 10.0.0.1\n";

fn key(id: &str, byte: u8) -> BodyEncryptionKey {
    BodyEncryptionKey::new(id, [byte; BodyEncryptionKey::LEN])
}

/// `plain` encrypted the way a client does it, with the key made of `byte`.
fn encrypted(byte: u8, plain: &[u8]) -> Vec<u8> {
    aes128gcm::encrypt(&[byte; BodyEncryptionKey::LEN], plain)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{:02x}", byte)).collect()
}

/// A server holding an old and a new key, the way it does while a key is being rotated, and the
/// key it answers with.
fn rotating_keys() -> Arc<BodyEncryptionKeys> {
    Arc::new(BodyEncryptionKeys::new(vec![
        key("old", 1),
        key("new", 2),
        key(REPLY_KEY_ID, 9),
    ]))
}

// ─────────────────────────────── tests ───────────────────────────────

/// Encrypted with the second key of the list: each key is tried until one opens it.
#[tokio::test]
async fn an_encrypted_json_body_reaches_the_action_decrypted() {
    let port = start_server(Some(rotating_keys())).await;

    let response = post(port, "/echo-json", Some("aes128gcm"), &encrypted(2, BODY)).await;

    assert!(response.status_is(200), "{:?}", response);
    assert_eq!(response.body_text(), "email=a@b.com");
}

/// `#[http_body_raw]` parses nothing - so this is decryption, not a JSON reader being clever.
#[tokio::test]
async fn an_encrypted_raw_body_is_decrypted_too() {
    let port = start_server(Some(rotating_keys())).await;

    let response = post(port, "/echo-raw", Some("AES128GCM"), &encrypted(1, BODY)).await;

    assert!(response.status_is(200), "{:?}", response);
    assert_eq!(
        response.body_text(),
        format!("raw={}:{}", BODY.len(), String::from_utf8_lossy(BODY))
    );
}

/// The keys can arrive after the start - in my-reverse-proxy they come with `/api/initialize`.
/// Until then there is no key, and the body is a `415`.
#[tokio::test]
async fn keys_set_after_the_start_are_used_from_the_next_request_on() {
    let keys = Arc::new(BodyEncryptionKeys::default());
    let port = start_server(Some(keys.clone())).await;

    let body = encrypted(5, BODY);

    let response = post(port, "/echo-json", Some("aes128gcm"), &body).await;
    assert!(response.status_is(415), "{:?}", response);

    keys.set(vec![key("late", 5)]);

    let response = post(port, "/echo-json", Some("aes128gcm"), &body).await;
    assert!(response.status_is(200), "{:?}", response);
    assert_eq!(response.body_text(), "email=a@b.com");
}

/// A server that never got keys does not take the coding at all: RFC 9110 §15.5.16 - a `415`
/// carrying `Accept-Encoding` with the codings it does take.
#[tokio::test]
async fn without_keys_an_encrypted_body_is_a_415_naming_the_codings_taken() {
    let port = start_server(None).await;

    let response = post(port, "/echo-json", Some("aes128gcm"), &encrypted(1, BODY)).await;

    assert!(response.status_is(415), "{:?}", response);
    assert_eq!(
        response.header("Accept-Encoding"),
        Some("gzip, deflate, br, zstd"),
        "{:?}",
        response
    );
    assert!(response.body_text().contains("aes128gcm"), "{:?}", response);
}

#[tokio::test]
async fn a_body_no_key_opens_is_a_400() {
    let port = start_server(Some(rotating_keys())).await;

    let response = post(port, "/echo-json", Some("aes128gcm"), &encrypted(7, BODY)).await;

    assert!(response.status_is(400), "{:?}", response);
    assert!(
        response.body_text().contains("no key opens it"),
        "{:?}",
        response
    );
}

/// The right key, a broken body: refused whole, never handed on in part.
#[tokio::test]
async fn a_damaged_body_is_a_400() {
    let port = start_server(Some(rotating_keys())).await;

    let mut body = encrypted(2, BODY);
    let last = body.len() - 1;
    body[last] ^= 0x01;

    let response = post(port, "/echo-raw", Some("aes128gcm"), &body).await;

    assert!(response.status_is(400), "{:?}", response);
}

/// Bytes that were never encrypted, sent under the header: the header is never taken back.
#[tokio::test]
async fn a_plain_body_announced_as_encrypted_is_a_400() {
    let port = start_server(Some(rotating_keys())).await;

    let response = post(port, "/echo-json", Some("aes128gcm"), BODY).await;

    assert!(response.status_is(400), "{:?}", response);
    assert!(response.body_text().contains("decrypt"), "{:?}", response);
}

/// Nothing changes for a body that announces no encoding, keys or not.
#[tokio::test]
async fn a_plain_body_is_untouched() {
    let port = start_server(Some(rotating_keys())).await;

    let response = post(port, "/echo-json", None, BODY).await;

    assert!(response.status_is(200), "{:?}", response);
    assert_eq!(response.body_text(), "email=a@b.com");
}

/// Only the header makes a body encrypted: encrypted bytes sent without it are the body, and the
/// action gets them as they are - the server does not guess.
#[tokio::test]
async fn an_encrypted_body_without_the_header_is_taken_as_it_is() {
    let port = start_server(Some(rotating_keys())).await;

    let body = encrypted(1, BODY);

    let response = post(port, "/echo-raw", None, &body).await;

    assert!(response.status_is(200), "{:?}", response);
    assert!(
        response
            .body_text()
            .starts_with(&format!("raw={}:", body.len())),
        "{:?}",
        response
    );
}

/// The boundary, the same as for a compressed body: a streamed body is passed through as it
/// arrived, so what the action sees is the encrypted length.
#[tokio::test]
async fn a_streamed_body_is_handed_on_still_encrypted() {
    let port = start_server(Some(rotating_keys())).await;

    let body = encrypted(1, BODY);
    assert_ne!(body.len(), BODY.len());

    let response = post(port, "/stream-through", Some("aes128gcm"), &body).await;

    assert!(response.status_is(200), "{:?}", response);
    assert_eq!(response.body_text(), format!("streamed={}", body.len()));
}

/// The action answers encrypted with the key it took by id: the body opens with that key and no
/// other, and says what it is encrypted with.
#[tokio::test]
async fn an_action_answers_encrypted_with_the_key_it_takes_by_id() {
    let keys = rotating_keys();
    let port = start_server(Some(keys.clone())).await;

    let response = send(
        port,
        b"GET /encoded-reply HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    )
    .await;

    assert!(response.status_is(200), "{:?}", response);
    assert_eq!(response.header("Content-Encoding"), Some("aes128gcm"));
    assert_eq!(response.header("Cache-Control"), Some("no-store"));
    assert!(
        response
            .header("Content-Type")
            .is_some_and(|value| value.starts_with("text/plain")),
        "{:?}",
        response
    );

    assert_ne!(response.body, REPLY.as_bytes());
    assert_eq!(
        aes128gcm::decrypt(&[9u8; BodyEncryptionKey::LEN], &response.body).unwrap(),
        REPLY.as_bytes()
    );
    assert!(aes128gcm::decrypt(&[1u8; BodyEncryptionKey::LEN], &response.body).is_err());

    // The list opens it as well: the reply key is one of its keys.
    assert_eq!(keys.decrypt(&response.body).unwrap(), REPLY.as_bytes());
}

/// The action reads from the request which key opened the body, its record size and its salt -
/// the first 16 bytes of the body the client sent - and answers with that same key.
#[tokio::test]
async fn an_action_reads_the_encryption_info_and_answers_with_the_key_the_client_used() {
    let port = start_server(Some(rotating_keys())).await;

    let body = encrypted(1, BODY);

    let response = post(port, "/answer-with-request-key", Some("aes128gcm"), &body).await;

    assert!(response.status_is(200), "{:?}", response);
    assert_eq!(response.header("Content-Encoding"), Some("aes128gcm"));

    let answer = aes128gcm::decrypt(&[1u8; BodyEncryptionKey::LEN], &response.body).unwrap();

    assert_eq!(
        String::from_utf8(answer).unwrap(),
        format!("email=a@b.com key=old rs=4096 salt={}", hex(&body[..16]))
    );
}

/// The info is there only for a body that came encrypted, so an action that takes encrypted
/// bodies only can refuse a plain one.
#[tokio::test]
async fn a_plain_body_comes_without_encryption_info() {
    let port = start_server(Some(rotating_keys())).await;

    let response = post(port, "/answer-with-request-key", None, BODY).await;

    assert!(response.status_is(415), "{:?}", response);
}
