use futures::FutureExt;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::StatusCode;
use std::panic::AssertUnwindSafe;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto;
#[cfg(feature = "with-telemetry")]
use my_telemetry::TelemetryEventTagsBuilder;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use rust_extensions::{ApplicationStates, Logger};

use std::sync::atomic::AtomicI64;
use std::{collections::HashMap, net::SocketAddr};
use tokio::sync::Mutex;

use std::sync::Arc;

use crate::{
    HttpContext, HttpFailResult, HttpRequest, HttpServerMiddleware, HttpServerMiddlewares,
};
use crate::{HttpOkResult, SocketAddress};

use crate::http_server_middleware::*;
use my_hyper_utils::*;

pub const PANIC_HTTP_CODE: StatusCode = StatusCode::INTERNAL_SERVER_ERROR;

#[derive(Clone)]
pub struct HttpConnectionsCounter {
    connections: Arc<AtomicI64>,
}

impl HttpConnectionsCounter {
    pub fn get_connections_amount(&self) -> i64 {
        self.connections.load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[derive(Debug, Clone)]
pub enum ListenAddr {
    Tcp(SocketAddr),
    Unix(Arc<String>),
}

pub struct MyHttpServer {
    pub addr: ListenAddr,
    middlewares: Option<Vec<Arc<dyn HttpServerMiddleware + Send + Sync + 'static>>>,
    tech_middlewares: Option<Vec<Arc<dyn HttpServerTechMiddleware + Send + Sync + 'static>>>,
    connections: Arc<AtomicI64>,
    body_read_timeout: Option<std::time::Duration>,
    max_decompressed_body_size: usize,
    tcp_nodelay: bool,
}

impl MyHttpServer {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr: ListenAddr::Tcp(addr),
            middlewares: Some(Vec::new()),
            tech_middlewares: Some(Vec::new()),
            connections: Arc::new(AtomicI64::new(0)),
            body_read_timeout: None,
            max_decompressed_body_size: crate::DEFAULT_MAX_DECOMPRESSED_BODY_SIZE,
            tcp_nodelay: true,
        }
    }

    #[cfg(unix)]
    pub fn new_as_unix_socket(unix_socket_addr: impl Into<String>) -> Self {
        Self {
            addr: ListenAddr::Unix(Arc::new(unix_socket_addr.into())),
            middlewares: Some(Vec::new()),
            tech_middlewares: Some(Vec::new()),
            connections: Arc::new(AtomicI64::new(0)),
            body_read_timeout: None,
            max_decompressed_body_size: crate::DEFAULT_MAX_DECOMPRESSED_BODY_SIZE,
            tcp_nodelay: true,
        }
    }

    /// Whether `TCP_NODELAY` is set on every accepted TCP connection - `true` unless switched off,
    /// as in Go's `net/http` and in nginx.
    ///
    /// Without it, Nagle's algorithm holds back the last small segment of a response until the
    /// client acknowledges the previous one, and a client delaying its ACK (up to ~40 ms on Linux)
    /// delays the end of the response just as long. HTTP/2, writing many small frames over one
    /// connection, notices it most.
    ///
    /// `false` leaves accepted connections as the OS hands them out, with Nagle's algorithm on.
    /// A Unix socket has no such option and is not affected either way.
    pub fn set_tcp_nodelay(&mut self, value: bool) {
        self.tcp_nodelay = value;
    }

    /// Stop waiting on a client that has gone quiet mid-body.
    ///
    /// This is an **idle** timeout — it is restarted for every piece of the body, so a large but
    /// progressing upload is never cut off, and time spent waiting for a slow *handler* to consume
    /// the body does not count against it either. Without it, a client that announces a large body
    /// and then goes silent holds a connection (and, for a streamed body, a pump task) forever.
    ///
    /// Off by default, to keep the behaviour this server has always had.
    pub fn set_body_read_timeout(&mut self, timeout: std::time::Duration) {
        self.body_read_timeout = Some(timeout);
    }

    /// How large a request body announced with a `Content-Encoding` may grow once decompressed.
    /// Past it the request is answered `413 Payload Too Large`, and the decoder is stopped as soon
    /// as it crosses the line - it never inflates the whole bomb first.
    ///
    /// Only decoded bodies are held to it: a body with no `Content-Encoding` is taken as it is,
    /// and a `#[http_body_as_stream]` body is never decoded by this server at all.
    ///
    /// [`DEFAULT_MAX_DECOMPRESSED_BODY_SIZE`](crate::DEFAULT_MAX_DECOMPRESSED_BODY_SIZE) (64 MiB)
    /// unless set.
    pub fn set_max_decompressed_body_size(&mut self, max_size: usize) {
        self.max_decompressed_body_size = max_size;
    }

    pub fn add_middleware(
        &mut self,
        middleware: Arc<dyn HttpServerMiddleware + Send + Sync + 'static>,
    ) {
        if self.middlewares.is_none() {
            panic!("You can not add middleware after starting server");
        }
        self.middlewares.as_mut().unwrap().push(middleware);
    }

    pub fn add_tech_middleware(
        &mut self,
        middleware: Arc<dyn HttpServerTechMiddleware + Send + Sync + 'static>,
    ) {
        if self.middlewares.is_none() {
            panic!("You can not add tech middleware after starting server");
        }
        self.tech_middlewares.as_mut().unwrap().push(middleware);
    }

    pub fn get_http_connections_counter(&self) -> HttpConnectionsCounter {
        HttpConnectionsCounter {
            connections: self.connections.clone(),
        }
    }

    pub fn start(
        &mut self,
        app_states: Arc<dyn ApplicationStates + Send + Sync + 'static>,
        logger: Arc<dyn Logger + Send + Sync + 'static>,
    ) {
        let middlewares = self.middlewares.take();

        if middlewares.is_none() {
            panic!("You can not start HTTP server two times");
        }

        logger.write_info(
            "Starting Http Server".to_string(),
            format!("Http server (auto h1/h2) starts at: {:?}", self.addr),
            None,
        );

        let http_server_middlewares = HttpServerMiddlewares {
            middlewares: middlewares.unwrap(),
            tech_middlewares: self.tech_middlewares.take().unwrap(),
            body_read_timeout: self.body_read_timeout,
            max_decompressed_body_size: self.max_decompressed_body_size,
            tcp_nodelay: self.tcp_nodelay,
        };

        let connections = self.connections.clone();
        match &self.addr {
            ListenAddr::Tcp(socket_addr) => {
                let socket_addr = socket_addr.clone();
                tokio::spawn(start_http_auto(
                    socket_addr,
                    Arc::new(http_server_middlewares),
                    app_states,
                    logger,
                    connections,
                ));
            }
            ListenAddr::Unix(unix_socket_addr) => {
                let unix_socket_addr = unix_socket_addr.clone();
                tokio::spawn(start_http_auto_unix_socket(
                    unix_socket_addr,
                    Arc::new(http_server_middlewares),
                    app_states,
                    logger,
                    connections,
                ));
            }
        }
    }

    /// Starts server in HTTP/1 only mode. Use it when auto h1/h2 negotiation is not desired.
    pub fn start_h1(
        &mut self,
        app_states: Arc<dyn ApplicationStates + Send + Sync + 'static>,
        logger: Arc<dyn Logger + Send + Sync + 'static>,
    ) {
        let middlewares = self.middlewares.take();

        if middlewares.is_none() {
            panic!("You can not start HTTP server two times");
        }

        logger.write_info(
            "Starting Http Server".to_string(),
            format!("Http server (h1 only) starts at: {:?}", self.addr),
            None,
        );

        let http_server_middlewares = HttpServerMiddlewares {
            middlewares: middlewares.unwrap(),
            tech_middlewares: self.tech_middlewares.take().unwrap(),
            body_read_timeout: self.body_read_timeout,
            max_decompressed_body_size: self.max_decompressed_body_size,
            tcp_nodelay: self.tcp_nodelay,
        };

        let connections = self.connections.clone();
        match &self.addr {
            ListenAddr::Tcp(socket_addr) => {
                let socket_addr = socket_addr.clone();
                tokio::spawn(start_http_1(
                    socket_addr,
                    Arc::new(http_server_middlewares),
                    app_states,
                    logger,
                    connections,
                ));
            }
            ListenAddr::Unix(unix_socket_addr) => {
                let unix_socket_addr = unix_socket_addr.clone();
                tokio::spawn(start_http_1_unix_socket(
                    unix_socket_addr,
                    Arc::new(http_server_middlewares),
                    app_states,
                    logger,
                    connections,
                ));
            }
        }
    }

    pub fn start_h2(
        &mut self,
        app_states: Arc<dyn ApplicationStates + Send + Sync + 'static>,
        logger: Arc<dyn Logger + Send + Sync + 'static>,
    ) {
        let middlewares = self.middlewares.take();

        let tech_middlewares = self.tech_middlewares.take();
        if middlewares.is_none() {
            panic!("You can not start HTTP2 server two times");
        }

        logger.write_info(
            "Starting Http2 Server".to_string(),
            format!("Http2 server starts at: {:?}", self.addr),
            None,
        );

        let http_server_middlewares = HttpServerMiddlewares {
            middlewares: middlewares.unwrap(),
            tech_middlewares: tech_middlewares.unwrap(),
            body_read_timeout: self.body_read_timeout,
            max_decompressed_body_size: self.max_decompressed_body_size,
            tcp_nodelay: self.tcp_nodelay,
        };

        let connections = self.connections.clone();

        match &self.addr {
            ListenAddr::Tcp(socket_addr) => {
                let socket_addr = socket_addr.clone();
                tokio::spawn(start_http_2(
                    socket_addr,
                    Arc::new(http_server_middlewares),
                    app_states,
                    logger,
                    connections,
                ));
            }
            ListenAddr::Unix(_) => {
                panic!("Unix socket does not support Http2 yet");
            }
        }
    }

    pub fn start_auto(
        &mut self,
        app_states: Arc<dyn ApplicationStates + Send + Sync + 'static>,
        logger: Arc<dyn Logger + Send + Sync + 'static>,
    ) {
        let middlewares = self.middlewares.take();
        let tech_middlewares = self.tech_middlewares.take();

        if middlewares.is_none() {
            panic!("You can not start HTTP server two times");
        }

        logger.write_info(
            "Starting Http Server (auto h1/h2)".to_string(),
            format!("Http server (auto) starts at: {:?}", self.addr),
            None,
        );

        let http_server_middlewares = HttpServerMiddlewares {
            middlewares: middlewares.unwrap(),
            tech_middlewares: tech_middlewares.unwrap(),
            body_read_timeout: self.body_read_timeout,
            max_decompressed_body_size: self.max_decompressed_body_size,
            tcp_nodelay: self.tcp_nodelay,
        };

        let connections = self.connections.clone();

        match &self.addr {
            ListenAddr::Tcp(socket_addr) => {
                let socket_addr = socket_addr.clone();
                tokio::spawn(start_http_auto(
                    socket_addr,
                    Arc::new(http_server_middlewares),
                    app_states,
                    logger,
                    connections,
                ));
            }
            ListenAddr::Unix(unix_socket_addr) => {
                let unix_socket_addr = unix_socket_addr.clone();
                tokio::spawn(start_http_auto_unix_socket(
                    unix_socket_addr,
                    Arc::new(http_server_middlewares),
                    app_states,
                    logger,
                    connections,
                ));
            }
        }
    }
}

pub async fn start_http_1(
    addr: SocketAddr,
    http_server_middlewares: Arc<HttpServerMiddlewares>,
    app_states: Arc<dyn ApplicationStates + Send + Sync + 'static>,
    logger: Arc<dyn Logger + Send + Sync + 'static>,
    connections: Arc<AtomicI64>,
) {
    let listener = tokio::net::TcpListener::bind(addr).await;

    if let Err(err) = &listener {
        let err = format!("Can not start http server at {}. Err: {:?}", addr, err);
        eprintln!("{}", err);
        panic!("{}", err);
    }

    let listener = listener.unwrap();
    let listen_addr = addr.to_string();
    let tcp_nodelay = http_server_middlewares.tcp_nodelay;

    let mut http1 = http1::Builder::new();
    http1.keep_alive(true);
    loop {
        if app_states.is_shutting_down() {
            break;
        }

        let Some((stream, socket_addr)) =
            accept_tcp(&listener, &listen_addr, tcp_nodelay, &logger).await
        else {
            continue;
        };

        connections.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let io = TokioIo::new(stream);
        let http_server_middlewares = http_server_middlewares.clone();
        let logger: Arc<dyn Logger + Send + Sync> = logger.clone();
        let app_states = app_states.clone();

        let http_server_middlewares = http_server_middlewares.clone();
        let logger = logger.clone();
        let socket_addr = socket_addr.clone();
        let app_states = app_states.clone();

        let connection = http1
            .serve_connection(
                io,
                service_fn(move |req| {
                    let resp = handle_requests(
                        req,
                        http_server_middlewares.clone(),
                        SocketAddress::Tcp(socket_addr.clone()),
                        logger.clone(),
                        app_states.is_shutting_down(),
                    );

                    resp
                }),
            )
            .with_upgrades();

        let connections_clone = connections.clone();
        tokio::task::spawn(async move {
            if let Err(_err) = connection.await {
                //println!("Error serving connection: {:?}", err);
            }

            connections_clone.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        });
    }
}

#[cfg(not(unix))]
pub async fn start_http_1_unix_socket(
    unix_socket: Arc<String>,
    http_server_middlewares: Arc<HttpServerMiddlewares>,
    app_states: Arc<dyn ApplicationStates + Send + Sync + 'static>,
    logger: Arc<dyn Logger + Send + Sync + 'static>,
    connections: Arc<AtomicI64>,
) {
    panic!("Unix socket is not supported by OS");
}

#[cfg(unix)]
pub async fn start_http_1_unix_socket(
    unix_socket: Arc<String>,
    http_server_middlewares: Arc<HttpServerMiddlewares>,
    app_states: Arc<dyn ApplicationStates + Send + Sync + 'static>,
    logger: Arc<dyn Logger + Send + Sync + 'static>,
    connections: Arc<AtomicI64>,
) {
    let _ = tokio::fs::remove_file(unix_socket.as_str()).await;
    let listener = tokio::net::UnixListener::bind(unix_socket.as_str());

    if let Err(err) = &listener {
        let err = format!(
            "Can not start http server at {}. Err: {:?}",
            unix_socket, err
        );
        eprintln!("{}", err);
        panic!("{}", err);
    }

    let listener = listener.unwrap();

    let mut http1 = http1::Builder::new();
    http1.keep_alive(true);
    loop {
        if app_states.is_shutting_down() {
            break;
        }

        let (stream, socket_addr) = match listener.accept().await {
            Ok(accepted) => accepted,
            Err(err) => {
                handle_accept_error(err, &unix_socket, &logger).await;
                continue;
            }
        };

        connections.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let io = TokioIo::new(stream);
        let http_server_middlewares = http_server_middlewares.clone();
        let logger: Arc<dyn Logger + Send + Sync> = logger.clone();
        let app_states = app_states.clone();

        let http_server_middlewares = http_server_middlewares.clone();
        let logger = logger.clone();
        let socket_addr = SocketAddress::Unix(Arc::new(format!("{:?}", socket_addr)));
        let app_states = app_states.clone();

        let connection = http1
            .serve_connection(
                io,
                service_fn(move |req| {
                    let resp = handle_requests(
                        req,
                        http_server_middlewares.clone(),
                        socket_addr.clone(),
                        logger.clone(),
                        app_states.is_shutting_down(),
                    );

                    resp
                }),
            )
            .with_upgrades();

        let connections_clone = connections.clone();
        tokio::task::spawn(async move {
            if let Err(_err) = connection.await {
                //println!("Error serving connection: {:?}", err);
            }

            connections_clone.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        });
    }
}

pub async fn start_http_2(
    addr: SocketAddr,
    http_server_middlewares: Arc<HttpServerMiddlewares>,
    app_states: Arc<dyn ApplicationStates + Send + Sync + 'static>,
    logger: Arc<dyn Logger + Send + Sync + 'static>,
    connections: Arc<AtomicI64>,
) {
    let listener = tokio::net::TcpListener::bind(addr).await;

    if let Err(err) = &listener {
        panic!("Error starting h2 server at {}. Err: {:?}", addr, err);
    }

    let listener = listener.unwrap();
    let listen_addr = addr.to_string();
    let tcp_nodelay = http_server_middlewares.tcp_nodelay;

    let mut http2_builder = hyper::server::conn::http2::Builder::new(TokioExecutor::new());
    http2_builder.enable_connect_protocol();
    let http2_builder = Arc::new(http2_builder);

    loop {
        if app_states.is_shutting_down() {
            break;
        }

        let Some((stream, socket_addr)) =
            accept_tcp(&listener, &listen_addr, tcp_nodelay, &logger).await
        else {
            continue;
        };

        connections.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let io = TokioIo::new(stream);
        let http_server_middlewares = http_server_middlewares.clone();
        let logger: Arc<dyn Logger + Send + Sync> = logger.clone();
        let app_states = app_states.clone();

        let http_server_middlewares = http_server_middlewares.clone();
        let logger = logger.clone();
        let socket_addr = socket_addr.clone();
        let app_states = app_states.clone();

        let builder = http2_builder.clone();

        let connection = builder.serve_connection(
            io,
            service_fn(move |req| {
                let resp = handle_requests(
                    req,
                    http_server_middlewares.clone(),
                    SocketAddress::Tcp(socket_addr.clone()),
                    logger.clone(),
                    app_states.is_shutting_down(),
                );
                resp
            }),
        );

        let connections_clone = connections.clone();
        tokio::task::spawn(async move {
            if let Err(_err) = connection.await {
                //println!("Error serving connection: {:?}", err);
            }

            connections_clone.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        });
    }
}

pub async fn start_http_auto(
    addr: SocketAddr,
    http_server_middlewares: Arc<HttpServerMiddlewares>,
    app_states: Arc<dyn ApplicationStates + Send + Sync + 'static>,
    logger: Arc<dyn Logger + Send + Sync + 'static>,
    connections: Arc<AtomicI64>,
) {
    let listener = tokio::net::TcpListener::bind(addr).await;

    if let Err(err) = &listener {
        let err = format!("Can not start http server at {}. Err: {:?}", addr, err);
        eprintln!("{}", err);
        panic!("{}", err);
    }

    let listener = listener.unwrap();
    let listen_addr = addr.to_string();
    let tcp_nodelay = http_server_middlewares.tcp_nodelay;

    let mut auto_builder = auto::Builder::new(TokioExecutor::new());
    auto_builder.http1().keep_alive(true);
    auto_builder.http2().enable_connect_protocol();
    let auto_builder = Arc::new(auto_builder);

    loop {
        if app_states.is_shutting_down() {
            break;
        }

        let Some((stream, socket_addr)) =
            accept_tcp(&listener, &listen_addr, tcp_nodelay, &logger).await
        else {
            continue;
        };

        connections.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let io = TokioIo::new(stream);
        let http_server_middlewares = http_server_middlewares.clone();
        let logger: Arc<dyn Logger + Send + Sync> = logger.clone();
        let app_states = app_states.clone();
        let builder = auto_builder.clone();
        let connections_clone = connections.clone();

        tokio::task::spawn(async move {
            let connection = builder.serve_connection_with_upgrades(
                io,
                service_fn(move |req| {
                    let resp = handle_requests(
                        req,
                        http_server_middlewares.clone(),
                        SocketAddress::Tcp(socket_addr.clone()),
                        logger.clone(),
                        app_states.is_shutting_down(),
                    );
                    resp
                }),
            );

            if let Err(_err) = connection.await {
                //println!("Error serving connection: {:?}", err);
            }

            connections_clone.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        });
    }
}

#[cfg(not(unix))]
pub async fn start_http_auto_unix_socket(
    _unix_socket: Arc<String>,
    _http_server_middlewares: Arc<HttpServerMiddlewares>,
    _app_states: Arc<dyn ApplicationStates + Send + Sync + 'static>,
    _logger: Arc<dyn Logger + Send + Sync + 'static>,
    _connections: Arc<AtomicI64>,
) {
    panic!("Unix socket is not supported by OS");
}

#[cfg(unix)]
pub async fn start_http_auto_unix_socket(
    unix_socket: Arc<String>,
    http_server_middlewares: Arc<HttpServerMiddlewares>,
    app_states: Arc<dyn ApplicationStates + Send + Sync + 'static>,
    logger: Arc<dyn Logger + Send + Sync + 'static>,
    connections: Arc<AtomicI64>,
) {
    let _ = tokio::fs::remove_file(unix_socket.as_str()).await;
    let listener = tokio::net::UnixListener::bind(unix_socket.as_str());

    if let Err(err) = &listener {
        let err = format!(
            "Can not start http server at {}. Err: {:?}",
            unix_socket, err
        );
        eprintln!("{}", err);
        panic!("{}", err);
    }

    let listener = listener.unwrap();

    let mut auto_builder = auto::Builder::new(TokioExecutor::new());
    auto_builder.http1().keep_alive(true);
    auto_builder.http2().enable_connect_protocol();
    let auto_builder = Arc::new(auto_builder);

    loop {
        if app_states.is_shutting_down() {
            break;
        }

        let (stream, socket_addr) = match listener.accept().await {
            Ok(accepted) => accepted,
            Err(err) => {
                handle_accept_error(err, &unix_socket, &logger).await;
                continue;
            }
        };

        connections.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let io = TokioIo::new(stream);
        let http_server_middlewares = http_server_middlewares.clone();
        let logger: Arc<dyn Logger + Send + Sync> = logger.clone();
        let socket_addr = SocketAddress::Unix(Arc::new(format!("{:?}", socket_addr)));
        let app_states = app_states.clone();
        let builder = auto_builder.clone();
        let connections_clone = connections.clone();

        tokio::task::spawn(async move {
            let connection = builder.serve_connection_with_upgrades(
                io,
                service_fn(move |req| {
                    let resp = handle_requests(
                        req,
                        http_server_middlewares.clone(),
                        socket_addr.clone(),
                        logger.clone(),
                        app_states.is_shutting_down(),
                    );
                    resp
                }),
            );

            if let Err(_err) = connection.await {
                //println!("Error serving connection: {:?}", err);
            }

            connections_clone.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        });
    }
}

/// How long an accept loop rests after `accept()` failed for a reason other than the one
/// connection being lost - typically `EMFILE` / `ENFILE`, the process or the system out of file
/// descriptors. Going round at once would only spin on the same error until some are freed.
const ACCEPT_ERROR_PAUSE: std::time::Duration = std::time::Duration::from_millis(100);

/// Accepts the next connection of a TCP accept loop, and sets `TCP_NODELAY` on it if asked to.
/// `None` when accepting failed: the error has been dealt with by [`handle_accept_error`], and the
/// loop just goes round again.
async fn accept_tcp(
    listener: &tokio::net::TcpListener,
    listen_addr: &str,
    tcp_nodelay: bool,
    logger: &Arc<dyn Logger + Send + Sync + 'static>,
) -> Option<(tokio::net::TcpStream, SocketAddr)> {
    let (stream, socket_addr) = match listener.accept().await {
        Ok(accepted) => accepted,
        Err(err) => {
            handle_accept_error(err, listen_addr, logger).await;
            return None;
        }
    };

    if tcp_nodelay {
        if let Err(err) = stream.set_nodelay(true) {
            // Typically the peer is already gone. Not a reason to drop the connection: it is
            // served either way, only without the option.
            let mut ctx = HashMap::new();
            ctx.insert("addr".to_string(), listen_addr.to_string());
            ctx.insert("ip".to_string(), socket_addr.ip().to_string());

            logger.write_debug_info(
                "Http server accept".to_string(),
                format!("Can not set TCP_NODELAY: {err}"),
                Some(ctx),
            );
        }
    }

    Some((stream, socket_addr))
}

/// What an accept loop does with a failed `accept()`, before it goes round again. A failed
/// `accept()` must not end the loop: the server would stop taking connections while the process
/// went on living.
///
/// - The peer gave up on its connection before it was accepted - a normal event, which costs that
///   one connection and nothing else: logged at debug level, and the next `accept()` follows at once.
/// - Anything else - out of file descriptors, say - is an error, and is logged as one; the loop
///   then rests [`ACCEPT_ERROR_PAUSE`] instead of spinning on it.
async fn handle_accept_error(
    err: std::io::Error,
    listen_addr: &str,
    logger: &Arc<dyn Logger + Send + Sync + 'static>,
) {
    let mut ctx = HashMap::new();
    ctx.insert("addr".to_string(), listen_addr.to_string());

    if is_connection_error(&err) {
        logger.write_debug_info(
            "Http server accept".to_string(),
            format!("The connection was lost before it was accepted: {err}"),
            Some(ctx),
        );
        return;
    }

    logger.write_error(
        "Http server accept".to_string(),
        format!(
            "Can not accept a connection: {err}. Retrying in {} ms",
            ACCEPT_ERROR_PAUSE.as_millis()
        ),
        Some(ctx),
    );

    tokio::time::sleep(ACCEPT_ERROR_PAUSE).await;
}

/// An `accept()` error which is about the one connection being accepted, not about the listener.
fn is_connection_error(err: &std::io::Error) -> bool {
    matches!(
        err.kind(),
        std::io::ErrorKind::ConnectionAborted
            | std::io::ErrorKind::ConnectionReset
            | std::io::ErrorKind::ConnectionRefused
    )
}

pub async fn handle_requests(
    req: hyper::Request<hyper::body::Incoming>,
    http_server_middlewares: Arc<HttpServerMiddlewares>,
    addr: SocketAddress,
    logger: Arc<dyn Logger + Send + Sync + 'static>,
    app_is_shutting_down: bool,
) -> hyper::Result<my_hyper_utils::MyHttpResponse> {
    if app_is_shutting_down {
        return compile_app_is_shutting_down_http_response();
    }

    // A request which can not even be taken apart - a target which is not a path, a header which
    // is not text - is the client's mistake: it is answered with the error it makes, before any
    // middleware sees it. It must not take the connection down with it.
    let mut req = match HttpRequest::new(req, addr) {
        Ok(req) => req,
        Err(err) => return Ok(err.into()),
    };
    req.set_body_read_timeout(http_server_middlewares.body_read_timeout);
    req.set_max_decompressed_body_size(http_server_middlewares.max_decompressed_body_size);

    let method = req.method.clone();
    let mut request_ctx = HttpContext::new(req);

    #[cfg(feature = "with-telemetry")]
    let ctx = request_ctx.telemetry_context.clone();

    let request_data = Arc::new(HttpRequestData {
        method,
        path: request_ctx.request.get_path().to_string(),
        ip: request_ctx.request.get_ip().get_real_ip().to_string(),
        started: DateTimeAsMicroseconds::now(),
    });

    let client_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

    let client_id_spawned = client_id.clone();
    let http_server_middlewares_cloned = http_server_middlewares.clone();
    let flow_execution_future = AssertUnwindSafe(async move {
        let mut credentials_assigned = false;

        for middleware in http_server_middlewares_cloned.middlewares.iter() {
            let result = middleware.handle_request(&mut request_ctx).await;

            if let Some(credentials) = request_ctx.credentials.as_ref() {
                if !credentials_assigned {
                    credentials_assigned = true;
                    let mut client_id_access = client_id_spawned.lock().await;
                    *client_id_access = Some(credentials.get_id().to_string());
                }
            }

            if let Some(http_result) = result {
                return MiddleWareFlowResult {
                    http_context: request_ctx,
                    http_result: http_result,
                };
            }
        }

        MiddleWareFlowResult {
            http_context: request_ctx,
            http_result: Err(HttpFailResult::as_not_found(
                "404 - Not Found".to_string(),
                false,
            )),
        }
    })
    .catch_unwind();

    let flow_execution_result = match flow_execution_future.await {
        Ok(flow_execution_result) => {
            if http_server_middlewares.tech_middlewares.len() > 0 {
                let response_data = ResponseData::from(&flow_execution_result.http_result);
                let request_data = request_data.clone();
                tokio::spawn(async move {
                    for middleware in http_server_middlewares.tech_middlewares.iter() {
                        middleware.got_result(&request_data, &response_data).await;
                    }
                });
            }

            flow_execution_result
        }
        Err(panic_payload) => {
            let panic_msg = if let Some(s) = panic_payload.downcast_ref::<&'static str>() {
                (*s).to_string()
            } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                s.clone()
            } else {
                "panic".to_string()
            };

            let request_data = Arc::new(request_data);
            let request_data_cloned = request_data.clone();
            tokio::spawn(async move {
                for middleware in http_server_middlewares.tech_middlewares.iter() {
                    middleware
                        .got_result(
                            &request_data_cloned,
                            &ResponseData {
                                status_code: PANIC_HTTP_CODE.as_u16(),
                                content_type: Some("text/plain".into()),
                                content_length: 0,
                                has_error: true,
                            },
                        )
                        .await;
                }
            });

            let client_id = client_id.lock().await.take();
            #[cfg(feature = "with-telemetry")]
            {
                let mut tags =
                    TelemetryEventTagsBuilder::new().add_ip(request_data.ip.as_str().to_string());

                if let Some(client_id) = client_id.as_ref() {
                    tags = tags.add("client_id", client_id.to_string());
                }

                my_telemetry::TELEMETRY_INTERFACE.write_fail(
                    &ctx,
                    request_data.started,
                    format!("[{}]{}", request_data.method, request_data.path.to_string()),
                    format!("Panic: {}", panic_msg),
                    tags.build(),
                );
            }

            let mut ctx = HashMap::new();
            ctx.insert("path".to_string(), request_data.path.to_string());
            ctx.insert("method".to_string(), request_data.method.to_string());
            ctx.insert("ip".to_string(), request_data.ip.to_string());

            if let Some(client_id) = client_id.as_ref() {
                ctx.insert("client_id".to_string(), client_id.to_string());
            }

            logger.write_error(
                "HttpRequest".to_string(),
                format!("Panic: {}", panic_msg),
                Some(ctx),
            );

            return Ok((PANIC_HTTP_CODE, "Internal server error").to_my_http_response());
        }
    };

    match flow_execution_result.http_result {
        Ok(ok_result) => {
            #[cfg(feature = "with-telemetry")]
            let mut ok_result = ok_result;

            if ok_result.write_telemetry {
                #[cfg(feature = "with-telemetry")]
                {
                    let mut tags = ok_result
                        .add_telemetry_tags
                        .take_tags()
                        .add_ip(request_data.ip.to_string());

                    if let Some(credentials) = &flow_execution_result.http_context.credentials {
                        tags = tags.add("client_id", credentials.get_id().to_string());
                    }

                    let telemetry_data = if let Some(process_name) =
                        flow_execution_result.http_context.process_name.as_ref()
                    {
                        format!("[{}]{}", request_data.method, process_name)
                    } else {
                        format!("[{}]{}", request_data.method, request_data.path)
                    };

                    my_telemetry::TELEMETRY_INTERFACE.write_success(
                        &ctx,
                        request_data.started,
                        telemetry_data,
                        format!("Status code: {}", ok_result.output.get_status_code()),
                        tags.into(),
                    );
                }
            }

            Ok(ok_result.output.into())
        }
        Err(err_result) => {
            #[cfg(feature = "with-telemetry")]
            let mut err_result = err_result;

            if err_result.write_telemetry {
                if err_result.write_to_log {
                    let mut ctx = HashMap::new();
                    ctx.insert("path".to_string(), request_data.path.to_string());
                    ctx.insert(
                        "method".to_string(),
                        flow_execution_result
                            .http_context
                            .request
                            .method
                            .to_string(),
                    );
                    ctx.insert("ip".to_string(), request_data.ip.to_string());
                    ctx.insert(
                        "httpCode".to_string(),
                        err_result.output.get_status_code().to_string(),
                    );

                    if let Some(credentials) = &flow_execution_result.http_context.credentials {
                        ctx.insert("client_id".to_string(), credentials.get_id().to_string());
                    }

                    logger.write_warning(
                        "HttpRequest".to_string(),
                        format!(
                            "Http request finished with error: {}",
                            err_result.output.get_text_as_error()
                        ),
                        Some(ctx),
                    );
                }

                #[cfg(feature = "with-telemetry")]
                {
                    let mut tags = err_result
                        .add_telemetry_tags
                        .take_tags()
                        .add_ip(request_data.ip.to_string());

                    if let Some(credentials) = &flow_execution_result.http_context.credentials {
                        tags = tags.add("client_id".to_string(), credentials.get_id().to_string());
                    }

                    let telemetry_data = if let Some(process_name) =
                        flow_execution_result.http_context.process_name.as_ref()
                    {
                        format!("[{}]{}", request_data.method, process_name)
                    } else {
                        format!("[{}]{}", request_data.method, request_data.path)
                    };

                    my_telemetry::TELEMETRY_INTERFACE.write_fail(
                        &ctx,
                        request_data.started,
                        telemetry_data,
                        format!("Status code: {}", err_result.output.get_status_code()),
                        tags.into(),
                    );
                }
            }

            Ok(err_result.output.into())
        }
    }
}

pub struct MiddleWareFlowResult {
    pub http_context: HttpContext,
    pub http_result: Result<HttpOkResult, HttpFailResult>,
}

fn compile_app_is_shutting_down_http_response() -> hyper::Result<my_hyper_utils::MyHttpResponse> {
    let builder = hyper::Response::builder().status(502);
    let content = "Application is shutting down";
    hyper::Result::Ok((builder, content.as_bytes().to_vec()).to_my_http_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct RecordingLogger {
        records: std::sync::Mutex<Vec<(&'static str, String)>>,
    }

    impl RecordingLogger {
        fn record(&self, level: &'static str, message: String) {
            self.records.lock().unwrap().push((level, message));
        }

        fn levels(&self) -> Vec<&'static str> {
            self.records.lock().unwrap().iter().map(|r| r.0).collect()
        }
    }

    impl Logger for RecordingLogger {
        fn write_info(&self, _p: String, m: String, _c: Option<HashMap<String, String>>) {
            self.record("info", m);
        }
        fn write_warning(&self, _p: String, m: String, _c: Option<HashMap<String, String>>) {
            self.record("warning", m);
        }
        fn write_error(&self, _p: String, m: String, _c: Option<HashMap<String, String>>) {
            self.record("error", m);
        }
        fn write_fatal_error(&self, _p: String, m: String, _c: Option<HashMap<String, String>>) {
            self.record("fatal", m);
        }
        fn write_debug_info(&self, _p: String, m: String, _c: Option<HashMap<String, String>>) {
            self.record("debug", m);
        }
    }

    /// Accepts one connection through `accept_tcp`, with `tcp_nodelay` as given.
    async fn accept_one(tcp_nodelay: bool) -> tokio::net::TcpStream {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let client = tokio::spawn(tokio::net::TcpStream::connect(addr));

        let logger: Arc<dyn Logger + Send + Sync + 'static> = Arc::new(RecordingLogger::default());
        let (stream, _) = accept_tcp(&listener, &addr.to_string(), tcp_nodelay, &logger)
            .await
            .expect("a connection is accepted");

        let _client = client.await.unwrap().unwrap();

        stream
    }

    #[tokio::test]
    async fn an_accepted_tcp_connection_gets_tcp_nodelay() {
        assert!(accept_one(true).await.nodelay().unwrap());
    }

    #[tokio::test]
    async fn tcp_nodelay_switched_off_leaves_the_connection_as_it_was() {
        assert!(!accept_one(false).await.nodelay().unwrap());
    }

    #[test]
    fn tcp_nodelay_is_on_by_default() {
        let mut server = MyHttpServer::new(SocketAddr::from(([127, 0, 0, 1], 0)));
        assert!(server.tcp_nodelay);

        server.set_tcp_nodelay(false);
        assert!(!server.tcp_nodelay);
    }

    #[tokio::test]
    async fn an_accept_error_is_logged_and_the_loop_rests() {
        let recording = Arc::new(RecordingLogger::default());
        let logger: Arc<dyn Logger + Send + Sync + 'static> = recording.clone();

        let started = std::time::Instant::now();
        handle_accept_error(
            std::io::Error::other("Too many open files"),
            "127.0.0.1:0",
            &logger,
        )
        .await;

        assert!(started.elapsed() >= ACCEPT_ERROR_PAUSE);
        assert_eq!(recording.levels(), vec!["error"]);
    }

    #[tokio::test]
    async fn a_connection_lost_before_accept_is_logged_at_debug_level_without_a_rest() {
        let recording = Arc::new(RecordingLogger::default());
        let logger: Arc<dyn Logger + Send + Sync + 'static> = recording.clone();

        for kind in [
            std::io::ErrorKind::ConnectionAborted,
            std::io::ErrorKind::ConnectionReset,
            std::io::ErrorKind::ConnectionRefused,
        ] {
            let started = std::time::Instant::now();
            handle_accept_error(std::io::Error::from(kind), "127.0.0.1:0", &logger).await;
            assert!(started.elapsed() < ACCEPT_ERROR_PAUSE, "{kind:?}");
        }

        assert_eq!(recording.levels(), vec!["debug", "debug", "debug"]);
    }
}
