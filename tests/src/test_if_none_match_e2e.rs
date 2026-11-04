//! End-to-end coverage of the ETag helpers of `my-http-server-core` the way a middleware with a
//! static-content cache of its own uses them: `strong_etag` / `weak_etag` for the header,
//! `HttpRequest::if_none_match_matches` for the precondition and `as_not_modified` for the answer.
//! The request goes through hyper for real, so a header sent as several lines arrives the way it
//! does in production.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use my_http_server::*;
use rust_extensions::{AppStates, Logger};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const HASH: &str = "n4bQgYhMfWWaL+qgxVrQFaO/TxsrC4Is0V1sFbDwCgg";

/// `/strong` is served with a strong ETag, `/weak` with a weak one - the same hash.
struct EtagMiddleware;

#[my_http_server::async_trait::async_trait]
impl HttpServerMiddleware for EtagMiddleware {
    async fn handle_request(
        &self,
        ctx: &mut HttpContext,
    ) -> Option<Result<HttpOkResult, HttpFailResult>> {
        let etag = match ctx.request.http_path.as_str() {
            "/strong" => strong_etag(HASH),
            "/weak" => weak_etag(HASH),
            _ => return None,
        };

        if ctx.request.if_none_match_matches(&etag) {
            return Some(
                HttpOutput::as_not_modified()
                    .add_header("ETag", etag)
                    .add_header("Cache-Control", "no-cache")
                    .into_ok_result(false),
            );
        }

        Some(
            HttpOutput::from_builder()
                .add_header("ETag", etag)
                .add_header("Cache-Control", "no-cache")
                .set_content_type(WebContentType::Text)
                .set_content(b"content".to_vec())
                .into_ok_result(false),
        )
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

async fn start_server() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let mut server = MyHttpServer::new(SocketAddr::from(([127, 0, 0, 1], port)));
    server.add_middleware(Arc::new(EtagMiddleware));
    server.start(
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

/// Sends a GET with `headers` - a name given twice goes as two lines - and returns the response
/// head.
async fn get(port: u16, target: &str, headers: &[(&str, &str)]) -> String {
    let mut request = format!("GET {target} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n");
    for (name, value) in headers {
        request.push_str(&format!("{name}: {value}\r\n"));
    }
    request.push_str("\r\n");

    let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    stream.write_all(request.as_bytes()).await.unwrap();
    stream.flush().await.unwrap();

    let mut buf = Vec::new();
    let _ = tokio::time::timeout(Duration::from_secs(10), stream.read_to_end(&mut buf)).await;

    let response = String::from_utf8_lossy(&buf).to_string();
    match response.find("\r\n\r\n") {
        Some(head_end) => response[..head_end].to_string(),
        None => response,
    }
}

fn status(head: &str) -> &str {
    head.split_whitespace().nth(1).unwrap_or_default()
}

fn header<'h>(head: &'h str, name: &str) -> Option<&'h str> {
    head.lines().skip(1).find_map(|line| {
        let (header_name, value) = line.split_once(':')?;
        header_name
            .trim()
            .eq_ignore_ascii_case(name)
            .then(|| value.trim())
    })
}

#[tokio::test]
async fn the_etag_helpers_answer_a_conditional_request() {
    let port = start_server().await;

    let ok = get(port, "/strong", &[]).await;
    assert_eq!(status(&ok), "200", "{ok}");
    assert_eq!(header(&ok, "etag"), Some(format!("\"{HASH}\"").as_str()));

    let weak_ok = get(port, "/weak", &[]).await;
    assert_eq!(
        header(&weak_ok, "etag"),
        Some(format!("W/\"{HASH}\"").as_str())
    );

    let quoted = format!("\"{HASH}\"");
    let weak = format!("W/\"{HASH}\"");
    let listed = format!("\"other\", {quoted}");

    for target in ["/strong", "/weak"] {
        for headers in [
            vec![("If-None-Match", quoted.as_str())],
            vec![("If-None-Match", weak.as_str())],
            vec![("If-None-Match", listed.as_str())],
            vec![("If-None-Match", "*")],
            vec![("If-None-Match", HASH)],
            vec![
                ("If-None-Match", "\"other\""),
                ("If-None-Match", quoted.as_str()),
            ],
        ] {
            let head = get(port, target, &headers).await;
            assert_eq!(status(&head), "304", "{target} {headers:?}: {head}");
            assert!(header(&head, "etag").is_some(), "{head}");
            assert_eq!(header(&head, "cache-control"), Some("no-cache"), "{head}");
            assert_eq!(header(&head, "content-type"), None, "{head}");
        }

        for headers in [
            vec![],
            vec![("If-None-Match", "\"other\"")],
            vec![
                ("If-None-Match", "\"other\""),
                ("If-None-Match", "\"another\""),
            ],
            vec![("If-None-Match", "garbage here")],
        ] {
            let head = get(port, target, &headers).await;
            assert_eq!(status(&head), "200", "{target} {headers:?}: {head}");
            assert_eq!(
                header(&head, "content-type"),
                Some("text/plain; charset=utf-8")
            );
        }
    }
}
