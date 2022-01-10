use hyper::{
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server, Uri,
};
use my_telemetry::MyTelemetry;
use rust_extensions::{ApplicationStates, StopWatch};
use std::{net::SocketAddr, time::Duration};

use std::sync::Arc;

use crate::{HttpContext, HttpFailResult, HttpServerMiddleware};

pub struct HttpServerData {
    middlewares: Vec<Arc<dyn HttpServerMiddleware + Send + Sync + 'static>>,
    telemetry: Option<Arc<dyn MyTelemetry + Send + Sync + 'static>>,
}

pub struct MyHttpServer {
    pub addr: SocketAddr,
    middlewares: Vec<Arc<dyn HttpServerMiddleware + Send + Sync + 'static>>,
    telemetry: Option<Arc<dyn MyTelemetry + Send + Sync + 'static>>,
}

impl MyHttpServer {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            middlewares: Vec::new(),
            telemetry: None,
        }
    }
    pub fn add_middleware(
        &mut self,
        middleware: Arc<dyn HttpServerMiddleware + Send + Sync + 'static>,
    ) {
        self.middlewares.push(middleware);
    }

    pub fn set_telemetry(&mut self, telemetry: Arc<dyn MyTelemetry + Send + Sync + 'static>) {
        self.telemetry = Some(telemetry);
    }

    pub fn start<TAppStates>(&self, app_states: Arc<TAppStates>)
    where
        TAppStates: ApplicationStates + Send + Sync + 'static,
    {
        let http_server_data = HttpServerData {
            middlewares: self.middlewares.clone(),
            telemetry: self.telemetry.clone(),
        };

        tokio::spawn(start(
            self.addr.clone(),
            Arc::new(http_server_data),
            app_states,
        ));
    }
}

pub async fn start<TAppStates>(
    addr: SocketAddr,
    http_server_data: Arc<HttpServerData>,
    app_states: Arc<TAppStates>,
) where
    TAppStates: ApplicationStates + Send + Sync + 'static,
{
    let http_server_data_spawned = http_server_data.clone();

    let make_service = make_service_fn(move |conn: &AddrStream| {
        let http_server_data = http_server_data_spawned.clone();
        let addr = conn.remote_addr();

        async move {
            Ok::<_, hyper::Error>(service_fn(move |_req| {
                handle_requests(_req, http_server_data.clone(), addr)
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_service);

    let server = server.with_graceful_shutdown(shutdown_signal(app_states));

    if let Err(e) = server.await {
        eprintln!("Http Server error: {}", e);
    }
}

pub struct RequestTelemetry<'s> {
    telemetry: &'s Arc<dyn MyTelemetry + Send + Sync + 'static>,
    sw: StopWatch,
    method: hyper::Method,
    uri: Uri,
}

pub async fn handle_requests(
    req: Request<Body>,
    http_server_data: Arc<HttpServerData>,
    addr: SocketAddr,
) -> hyper::Result<Response<Body>> {
    let mut ctx = HttpContext::new(req, addr);

    let my_telemetry = if let Some(telemetry) = &http_server_data.telemetry {
        let mut sw = StopWatch::new();

        sw.start();

        Some(RequestTelemetry {
            telemetry,
            sw,
            method: ctx.get_method().clone(),
            uri: ctx.req.uri().clone(),
        })
    } else {
        None
    };

    for middleware in &http_server_data.middlewares {
        match middleware.handle_request(ctx).await {
            Ok(result) => match result {
                crate::MiddleWareResult::Ok(ok_result) => {
                    if let Some(mut my_telemetry) = my_telemetry {
                        my_telemetry.sw.pause();
                        my_telemetry.telemetry.track_url_duration(
                            my_telemetry.method,
                            my_telemetry.uri,
                            ok_result.get_status_code(),
                            my_telemetry.sw.duration(),
                        );
                    }

                    return Ok(ok_result.into());
                }
                crate::MiddleWareResult::Next(next_ctx) => {
                    ctx = next_ctx;
                }
            },
            Err(fail_result) => {
                if fail_result.write_telemetry {
                    if let Some(mut my_telemetry) = my_telemetry {
                        my_telemetry.sw.pause();
                        my_telemetry.telemetry.track_url_duration(
                            my_telemetry.method,
                            my_telemetry.uri,
                            fail_result.status_code,
                            my_telemetry.sw.duration(),
                        );
                    }
                }

                return Ok(fail_result.into());
            }
        }
    }

    let not_found = HttpFailResult::as_not_found("Page not found".to_string(), false);

    return Ok(not_found.into());
}

async fn shutdown_signal<TAppStates: ApplicationStates>(app: Arc<TAppStates>) {
    let duration = Duration::from_secs(1);
    while !app.is_shutting_down() {
        tokio::time::sleep(duration).await;
    }
}
