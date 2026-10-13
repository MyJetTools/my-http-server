//! End-to-end coverage of `StaticFilesMiddleware` against a real server over a raw TCP socket:
//! it serves the files of its folders and **nothing out of them**.
//!
//! The middleware glues the request path to the folder, and the file system follows a `..` - so
//! a path with a parent segment is answered 403 before any file is looked at, however the
//! segment is spelled.
//!
//! The socket is written to by hand because an HTTP client would take the `..` out of the path
//! before sending it: what a test sends here is what gets to the wire.

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

/// Two folders next to a file which is in neither of them:
///
/// ```text
/// <dir>/secret.txt
/// <dir>/root/index.html
/// <dir>/root/css/site.css
/// <dir>/root/a..b.txt
/// <dir>/root/..a.txt
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
    let port = free_port();

    let mut static_files = StaticFilesMiddleware::new().add_index_file("index.html");
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
