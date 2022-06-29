use hyper::{
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};

use rust_extensions::{ApplicationStates, Logger};
use std::{net::SocketAddr, time::Duration};

use std::sync::Arc;

use crate::{request_flow::HttpServerRequestFlow, HttpContext, HttpRequest, HttpServerMiddleware};

pub struct HttpServerData {
    middlewares: Vec<Arc<dyn HttpServerMiddleware + Send + Sync + 'static>>,
}

pub struct MyHttpServer {
    pub addr: SocketAddr,
    middlewares: Option<Vec<Arc<dyn HttpServerMiddleware + Send + Sync + 'static>>>,
}

impl MyHttpServer {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            middlewares: Some(Vec::new()),
        }
    }
    pub fn add_middleware(
        &mut self,
        middleware: Arc<dyn HttpServerMiddleware + Send + Sync + 'static>,
    ) {
        match &mut self.middlewares {
            Some(middlewares) => middlewares.push(middleware),
            None => {
                panic!("Cannot add middleware after server is started");
            }
        }
    }

    pub fn start(
        &mut self,
        app_states: Arc<dyn ApplicationStates + Send + Sync + 'static>,
        logger: Arc<dyn Logger + Send + Sync + 'static>,
    ) {
        let mut middlewares = None;

        std::mem::swap(&mut middlewares, &mut self.middlewares);

        if middlewares.is_none() {
            panic!("You can not start HTTP server two times");
        }

        logger.write_info(
            "Starting Http Server".to_string(),
            format!("Http server starts at: {:?}", self.addr),
            None,
        );

        let http_server_data = HttpServerData {
            middlewares: middlewares.unwrap(),
        };

        tokio::spawn(start(
            self.addr.clone(),
            Arc::new(http_server_data),
            app_states,
        ));
    }
}

pub async fn start(
    addr: SocketAddr,
    http_server_data: Arc<HttpServerData>,
    app_states: Arc<dyn ApplicationStates + Send + Sync + 'static>,
) {
    let http_server_data_spawned = http_server_data.clone();

    let make_service = make_service_fn(move |conn: &AddrStream| {
        let http_server_data = http_server_data_spawned.clone();
        let addr = conn.remote_addr();

        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                handle_requests(req, http_server_data.clone(), addr)
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_service);

    let server = server.with_graceful_shutdown(shutdown_signal(app_states));

    if let Err(e) = server.await {
        eprintln!("Http Server error: {}", e);
    }
}

pub async fn handle_requests(
    req: Request<Body>,
    http_server_data: Arc<HttpServerData>,
    addr: SocketAddr,
) -> hyper::Result<Response<Body>> {
    let req = HttpRequest::new(req, addr);
    let mut ctx = HttpContext::new(req);

    let mut flows = HttpServerRequestFlow::new(http_server_data.middlewares.clone());

    let result = flows.next(&mut ctx).await;

    match result {
        Ok(ok_result) => Ok(ok_result.into()),
        Err(err_result) => Ok(err_result.into()),
    }
}

async fn shutdown_signal(app: Arc<dyn ApplicationStates + Send + Sync + 'static>) {
    let duration = Duration::from_secs(1);
    while !app.is_shutting_down() {
        tokio::time::sleep(duration).await;
    }
}
