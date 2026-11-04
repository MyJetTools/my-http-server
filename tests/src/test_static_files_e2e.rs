//! End-to-end coverage of `StaticFilesMiddleware` against a real server over a raw TCP socket:
//! it serves the files of its folders and **nothing out of them**.
//!
//! The middleware glues the request path to the folder, and the file system follows a `..` - so
//! a path with a parent segment is answered 403 before any file is looked at, however the
//! segment is spelled.
//!
//! The socket is written to by hand because an HTTP client would take the `..` out of the path
//! before sending it: what a test sends here is what gets to the wire.
//!
//! The ETag half: the `ETag` is quoted - weak for a compressed variant - and `If-None-Match` is
//! answered `304` for every spelling of it a client may send, the unquoted one of 0.9.0 included.
//! A `304` carries the caching headers of the `200` and no `Content-Type`.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use my_http_server::{FilesMapping, MyHttpServer, StaticFilesMiddleware};
use rust_extensions::{AppStates, Logger};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

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

const INDEX_HTML: &str = "<html>index</html>";
const SITE_CSS: &str = "body { margin: 0 }";
const APP_JS: &str = "console.log('app')";
const DOTS_TXT: &str = "a name with two dots in it";
const SECRET: &str = "-----BEGIN OPENSSH PRIVATE KEY-----";
const WORKER_MJS: &str = "export const answer = 42;";

/// Large and compressible enough to be stored gzipped once file caching is on.
fn big_js() -> String {
    "console.log('a line of JavaScript which compresses well');\n".repeat(1_000)
}

/// Two folders next to a file which is in neither of them:
///
/// ```text
/// <dir>/secret.txt
/// <dir>/root/index.html
/// <dir>/root/css/site.css
/// <dir>/root/a..b.txt
/// <dir>/root/..a.txt
/// <dir>/root/big.js
/// <dir>/root/worker.mjs
/// <dir>/mapped/app.js
/// ```
struct Site {
    dir: PathBuf,
}

impl Site {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

        let dir = std::env::temp_dir().join(format!(
            "static-files-e2e-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));

        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("root/css")).unwrap();
        std::fs::create_dir_all(dir.join("mapped")).unwrap();

        std::fs::write(dir.join("secret.txt"), SECRET).unwrap();
        std::fs::write(dir.join("root/index.html"), INDEX_HTML).unwrap();
        std::fs::write(dir.join("root/css/site.css"), SITE_CSS).unwrap();
        std::fs::write(dir.join("root/a..b.txt"), DOTS_TXT).unwrap();
        std::fs::write(dir.join("root/..a.txt"), DOTS_TXT).unwrap();
        std::fs::write(dir.join("root/big.js"), big_js()).unwrap();
        std::fs::write(dir.join("root/worker.mjs"), WORKER_MJS).unwrap();
        std::fs::write(dir.join("mapped/app.js"), APP_JS).unwrap();

        Self { dir }
    }

    fn folder(&self, name: &str) -> String {
        self.dir.join(name).to_str().unwrap().to_string()
    }
}

impl Drop for Site {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// `/static/...` is served out of `<dir>/mapped`, everything else out of `<dir>/root` - the way
/// `./wwwroot` is, only at a place a test can make.
async fn start_server(site: &Site) -> u16 {
    start_server_with(site, |static_files| static_files).await
}

/// [`start_server`], with the middleware configured further by `configure`.
async fn start_server_with(
    site: &Site,
    configure: impl FnOnce(StaticFilesMiddleware) -> StaticFilesMiddleware,
) -> u16 {
    let port = free_port();

    let mut static_files = configure(StaticFilesMiddleware::new().add_index_file("index.html"));
    static_files
        .file_folders
        .push(FilesMapping::new("/static", &site.folder("mapped")));
    static_files
        .file_folders
        .push(FilesMapping::new("/", &site.folder("root")));

    let mut server = MyHttpServer::new(SocketAddr::from(([127, 0, 0, 1], port)));
    server.add_middleware(Arc::new(static_files));

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

/// Sends a GET for `target` exactly as it is written and returns the whole response.
async fn get(port: u16, target: &str) -> String {
    let request = format!("GET {target} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n");

    let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    stream.write_all(request.as_bytes()).await.unwrap();
    stream.flush().await.unwrap();

    let mut buf = Vec::new();
    let _ = tokio::time::timeout(Duration::from_secs(10), stream.read_to_end(&mut buf)).await;

    String::from_utf8_lossy(&buf).to_string()
}

struct Response {
    status: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl Response {
    /// The value of the header `name`, which must not be sent more than once.
    fn header(&self, name: &str) -> Option<&str> {
        let mut values = self
            .headers
            .iter()
            .filter(|(header_name, _)| header_name.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str());

        let result = values.next();
        assert!(values.next().is_none(), "{name} is sent more than once");
        result
    }

    fn etag(&self) -> String {
        self.header("etag").expect("an ETag").to_string()
    }
}

/// Sends a GET for `target` with `headers` - a name given twice goes as two lines - and returns
/// the response.
async fn request(port: u16, target: &str, headers: &[(&str, &str)]) -> Response {
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

    let head_end = buf
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .unwrap_or_else(|| panic!("no end of headers in: {}", String::from_utf8_lossy(&buf)));

    let head = String::from_utf8_lossy(&buf[..head_end]).to_string();
    let mut lines = head.split("\r\n");

    let status = lines
        .next()
        .and_then(|status_line| status_line.split_whitespace().nth(1))
        .and_then(|status| status.parse().ok())
        .unwrap_or_else(|| panic!("no status in: {head}"));

    let headers = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.trim().to_string(), value.trim().to_string()))
        .collect();

    Response {
        status,
        headers,
        body: buf[head_end + 4..].to_vec(),
    }
}

fn assert_served(response: &str, content: &str, target: &str) {
    assert!(response.starts_with("HTTP/1.1 200"), "{target}: {response}");
    assert!(response.ends_with(content), "{target}: {response}");
}

fn assert_forbidden(response: &str, target: &str) {
    assert!(response.starts_with("HTTP/1.1 403"), "{target}: {response}");
    assert!(!response.contains(SECRET), "{target}: {response}");
}

// ─────────────────────────────── tests ───────────────────────────────

#[tokio::test]
async fn the_files_of_the_folders_are_served() {
    let site = Site::new();
    let port = start_server(&site).await;

    for target in ["/", "/index.html", "/index.html?v=1"] {
        assert_served(&get(port, target).await, INDEX_HTML, target);
    }

    assert_served(&get(port, "/css/site.css").await, SITE_CSS, "/css/site.css");
    assert_served(&get(port, "/static/app.js").await, APP_JS, "/static/app.js");
}

/// Each of these names `<dir>/secret.txt` to a file system - or would, once something on the
/// way decoded the path.
#[tokio::test]
async fn a_path_with_a_parent_segment_is_forbidden() {
    let site = Site::new();
    let port = start_server(&site).await;

    for target in [
        "/../secret.txt",
        "/css/../../secret.txt",
        "/../secret.txt?v=1",
        "/..\\secret.txt",
        "/%2e%2e/secret.txt",
        "/%2E%2E/secret.txt",
        "/.%2e/secret.txt",
        "/%2e./secret.txt",
        "/..%2fsecret.txt",
        "/..%5csecret.txt",
        // Out of a mapped folder too.
        "/static/../secret.txt",
        "/static/%2e%2e/secret.txt",
        // A parent segment is refused even where it stays in the folder: the answer does not
        // depend on what is on the disk.
        "/css/../index.html",
        "/..",
        "/css/..",
    ] {
        assert_forbidden(&get(port, target).await, target);
    }

    // The files are served after that as they were.
    assert_served(&get(port, "/index.html").await, INDEX_HTML, "/index.html");
}

/// Two dots inside a name are a part of the name, not a segment.
#[tokio::test]
async fn a_name_with_dots_in_it_is_served() {
    let site = Site::new();
    let port = start_server(&site).await;

    for target in ["/a..b.txt", "/..a.txt"] {
        assert_served(&get(port, target).await, DOTS_TXT, target);
    }
}

#[tokio::test]
async fn files_go_with_their_content_type() {
    let site = Site::new();
    let port = start_server(&site).await;

    for (target, content_type) in [
        ("/index.html", "text/html; charset=utf-8"),
        ("/css/site.css", "text/css; charset=utf-8"),
        ("/static/app.js", "text/javascript; charset=utf-8"),
        // A module script or worker is not run by a browser without a JavaScript type.
        ("/worker.mjs", "text/javascript; charset=utf-8"),
        ("/a..b.txt", "text/plain; charset=utf-8"),
    ] {
        let response = request(port, target, &[]).await;
        assert_eq!(response.status, 200, "{target}");
        assert_eq!(
            response.header("content-type"),
            Some(content_type),
            "{target}"
        );
    }
}

#[tokio::test]
async fn the_etag_is_quoted_and_matched_however_the_client_spells_it() {
    let site = Site::new();
    let port = start_server_with(&site, |static_files| static_files.with_etag()).await;

    let ok = request(port, "/css/site.css", &[]).await;
    assert_eq!(ok.status, 200);
    assert!(ok.body.ends_with(SITE_CSS.as_bytes()));

    let etag = ok.etag();
    assert!(
        etag.len() > 2 && etag.starts_with('"') && etag.ends_with('"'),
        "{etag}"
    );
    let opaque_tag = &etag[1..etag.len() - 1];
    assert!(!opaque_tag.contains('"'), "{etag}");

    for if_none_match in [
        etag.clone(),
        format!("W/{etag}"),
        format!("\"other\", {etag}"),
        format!("\"other\",{etag} , W/\"another\""),
        "*".to_string(),
        // The way 0.9.0 sent it - the same hash, without quotes.
        opaque_tag.to_string(),
    ] {
        let response = request(port, "/css/site.css", &[("If-None-Match", &if_none_match)]).await;
        assert_eq!(response.status, 304, "If-None-Match: {if_none_match}");
        assert_eq!(response.header("etag"), Some(etag.as_str()));
        assert!(response.body.is_empty(), "If-None-Match: {if_none_match}");
    }

    for if_none_match in [
        "\"other\"".to_string(),
        "W/\"other\"".to_string(),
        format!("{etag}x"),
        format!("{etag}, \"unterminated"),
        format!("\"{opaque_tag}x\""),
    ] {
        let response = request(port, "/css/site.css", &[("If-None-Match", &if_none_match)]).await;
        assert_eq!(response.status, 200, "If-None-Match: {if_none_match}");
        assert!(response.body.ends_with(SITE_CSS.as_bytes()));
    }
}

#[tokio::test]
async fn if_none_match_sent_in_several_lines_is_one_list() {
    let site = Site::new();
    let port = start_server_with(&site, |static_files| static_files.with_etag()).await;

    let etag = request(port, "/index.html", &[]).await.etag();

    let response = request(
        port,
        "/index.html",
        &[("If-None-Match", "\"other\""), ("If-None-Match", &etag)],
    )
    .await;
    assert_eq!(response.status, 304);

    let response = request(
        port,
        "/index.html",
        &[
            ("If-None-Match", "\"other\""),
            ("If-None-Match", "\"another\""),
        ],
    )
    .await;
    assert_eq!(response.status, 200);
}

#[tokio::test]
async fn not_modified_carries_the_caching_headers_and_no_content_type() {
    let site = Site::new();
    let port = start_server_with(&site, |static_files| {
        static_files
            .with_etag()
            .add_header("X-Frame-Options", "DENY")
    })
    .await;

    let ok = request(port, "/index.html", &[]).await;
    assert_eq!(ok.status, 200);
    assert_eq!(ok.header("content-type"), Some("text/html; charset=utf-8"));
    assert_eq!(ok.header("cache-control"), Some("no-cache"));
    assert_eq!(ok.header("vary"), Some("Accept-Encoding"));

    let etag = ok.etag();
    let not_modified = request(port, "/index.html", &[("If-None-Match", &etag)]).await;

    assert_eq!(not_modified.status, 304);
    assert_eq!(not_modified.header("etag"), Some(etag.as_str()));
    assert_eq!(not_modified.header("cache-control"), Some("no-cache"));
    assert_eq!(not_modified.header("vary"), Some("Accept-Encoding"));
    assert_eq!(not_modified.header("x-frame-options"), Some("DENY"));
    assert_eq!(not_modified.header("content-type"), None);
    assert_eq!(not_modified.header("content-encoding"), None);
    assert!(not_modified.body.is_empty());
}

#[tokio::test]
async fn a_compressed_variant_has_a_weak_etag() {
    let site = Site::new();
    let port = start_server_with(&site, |static_files| {
        static_files.enable_files_caching().with_etag()
    })
    .await;

    let gzip = request(port, "/big.js", &[("Accept-Encoding", "gzip")]).await;
    assert_eq!(gzip.status, 200);
    assert_eq!(gzip.header("content-encoding"), Some("gzip"));
    assert_eq!(
        gzip.header("content-type"),
        Some("text/javascript; charset=utf-8")
    );
    let weak = gzip.etag();
    assert!(weak.starts_with("W/\"") && weak.ends_with('"'), "{weak}");

    let deflate = request(port, "/big.js", &[("Accept-Encoding", "deflate")]).await;
    assert_eq!(deflate.header("content-encoding"), Some("deflate"));
    assert_eq!(deflate.etag(), weak);

    // The raw bytes are what the hash is of: the same tag, strong.
    let raw = request(port, "/big.js", &[]).await;
    assert_eq!(raw.header("content-encoding"), None);
    assert_eq!(raw.body, big_js().as_bytes());
    let strong = raw.etag();
    assert!(strong.starts_with('"'), "{strong}");
    assert_eq!(format!("W/{strong}"), weak);

    // A 304 carries the ETag of the variant the request would get - whichever one the client has.
    let not_modified = request(
        port,
        "/big.js",
        &[("Accept-Encoding", "gzip"), ("If-None-Match", &strong)],
    )
    .await;
    assert_eq!(not_modified.status, 304);
    assert_eq!(not_modified.header("etag"), Some(weak.as_str()));
    assert_eq!(not_modified.header("content-encoding"), None);

    let not_modified = request(port, "/big.js", &[("If-None-Match", &weak)]).await;
    assert_eq!(not_modified.status, 304);
    assert_eq!(not_modified.header("etag"), Some(strong.as_str()));

    // Too small to be stored compressed: it goes raw to everybody, with a strong ETag.
    let small = request(port, "/css/site.css", &[("Accept-Encoding", "gzip")]).await;
    assert_eq!(small.header("content-encoding"), None);
    assert!(small.etag().starts_with('"'));
}

/// A restarted server knows no ETag yet - the client's one is matched against the file itself.
#[tokio::test]
async fn a_fresh_server_answers_the_first_conditional_request_with_304() {
    let site = Site::new();
    let configure = |static_files: StaticFilesMiddleware| {
        static_files
            .with_etag()
            .set_not_found_file("index.html".to_string())
    };

    let etag = request(
        start_server_with(&site, configure).await,
        "/index.html",
        &[],
    )
    .await
    .etag();
    let opaque_tag = etag.trim_matches('"').to_string();

    for (target, if_none_match) in [
        ("/index.html", etag.as_str()),
        ("/index.html", opaque_tag.as_str()),
        // Served by the not-found file.
        ("/no/such/route", etag.as_str()),
    ] {
        let port = start_server_with(&site, configure).await;
        let response = request(port, target, &[("If-None-Match", if_none_match)]).await;
        assert_eq!(
            response.status, 304,
            "{target} If-None-Match: {if_none_match}"
        );
        assert_eq!(response.header("etag"), Some(etag.as_str()));
    }
}

#[tokio::test]
async fn the_not_found_file_is_answered_304_too() {
    let site = Site::new();
    let port = start_server_with(&site, |static_files| {
        static_files
            .with_etag()
            .set_not_found_file("index.html".to_string())
    })
    .await;

    let ok = request(port, "/no/such/route", &[]).await;
    assert_eq!(ok.status, 200);
    assert!(ok.body.ends_with(INDEX_HTML.as_bytes()));
    let etag = ok.etag();

    let not_modified = request(port, "/another/route", &[("If-None-Match", &etag)]).await;
    assert_eq!(not_modified.status, 304);
    assert_eq!(not_modified.header("etag"), Some(etag.as_str()));
    assert_eq!(not_modified.header("cache-control"), Some("no-cache"));
    assert_eq!(not_modified.header("vary"), Some("Accept-Encoding"));
    assert_eq!(not_modified.header("content-type"), None);
}

#[tokio::test]
async fn a_no_cache_path_ignores_if_none_match() {
    let site = Site::new();
    let port = start_server_with(&site, |static_files| {
        static_files
            .with_etag()
            .add_no_cache_headers_to_response_by_path("/index.html")
    })
    .await;

    let etag = request(port, "/css/site.css", &[]).await.etag();
    assert!(etag.starts_with('"'));

    for if_none_match in ["*", "\"other\""] {
        let response = request(port, "/index.html", &[("If-None-Match", if_none_match)]).await;
        assert_eq!(response.status, 200, "If-None-Match: {if_none_match}");
        assert_eq!(response.header("etag"), None);
        assert_eq!(
            response.header("cache-control"),
            Some("no-store, no-cache, must-revalidate, max-age=0")
        );
    }
}
