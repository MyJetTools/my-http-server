use hyper::{
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};

#[cfg(feature = "my-telemetry")]
use my_telemetry::TelemetryEvent;
#[cfg(feature = "my-telemetry")]
use rust_extensions::date_time::DateTimeAsMicroseconds;
use rust_extensions::{ApplicationStates, Logger};
use std::{collections::HashMap, net::SocketAddr, time::Duration};

use std::sync::Arc;

use crate::{
    request_flow::HttpServerRequestFlow, HttpContext, HttpFailResult, HttpRequest, HttpServerData,
    HttpServerMiddleware,
};

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
        let middlewares = self.middlewares.take();

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
            logger,
        ));
    }
}

pub async fn start(
    addr: SocketAddr,
    http_server_data: Arc<HttpServerData>,
    app_states: Arc<dyn ApplicationStates + Send + Sync + 'static>,
    logger: Arc<dyn Logger + Send + Sync + 'static>,
) {
    let http_server_data_spawned = http_server_data.clone();

    let make_service = make_service_fn(move |conn: &AddrStream| {
        let http_server_data = http_server_data_spawned.clone();

        let logger_to_move = logger.clone();
        let addr = conn.remote_addr();

        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                handle_requests(req, http_server_data.clone(), addr, logger_to_move.clone())
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
    logger: Arc<dyn Logger + Send + Sync + 'static>,
) -> hyper::Result<Response<Body>> {
    let req = HttpRequest::new(req, addr);
    let mut request_ctx = HttpContext::new(req);

    #[cfg(feature = "my-telemetry")]
    let process_id = request_ctx.telemetry_context.process_id;

    let path = request_ctx.request.get_path().to_string();
    let method = request_ctx.request.get_method().to_string();
    let ip = request_ctx.request.get_ip().to_string();

    #[cfg(feature = "my-telemetry")]
    let started = DateTimeAsMicroseconds::now();

    let result = tokio::spawn(async move {
        let mut flows = HttpServerRequestFlow::new(http_server_data.middlewares.clone());
        let result = flows.next(&mut request_ctx).await;
        (result, request_ctx)
    });

    let (result, request_ctx) = match result.await {
        Ok(result) => (result.0, result.1),
        Err(err) => {
            #[cfg(feature = "my-telemetry")]
            if my_telemetry::TELEMETRY_INTERFACE.is_telemetry_set_up() {
                my_telemetry::TELEMETRY_INTERFACE
                    .write_telemetry_event(TelemetryEvent {
                        process_id: process_id,
                        started: started.unix_microseconds,
                        finished: DateTimeAsMicroseconds::now().unix_microseconds,
                        data: format!("[{}]{}", method, path),
                        success: None,
                        fail: Some(format!("Panic: {:?}", err)),
                        ip: Some(ip.clone()),
                    })
                    .await;
            }

            let mut ctx = HashMap::new();
            ctx.insert("path".to_string(), path);
            ctx.insert("method".to_string(), method);
            ctx.insert("ip".to_string(), ip);

            logger.write_error(
                "HttpRequest".to_string(),
                format!("Panic: {:?}", err),
                Some(ctx),
            );

            panic!("Http Server error");
        }
    };

    match result {
        Ok(ok_result) => {
            if ok_result.write_telemetry {
                #[cfg(feature = "my-telemetry")]
                if my_telemetry::TELEMETRY_INTERFACE.is_telemetry_set_up() {
                    my_telemetry::TELEMETRY_INTERFACE
                        .write_telemetry_event(TelemetryEvent {
                            process_id: process_id,
                            started: started.unix_microseconds,
                            finished: DateTimeAsMicroseconds::now().unix_microseconds,
                            data: format!("[{}]{}", method, path),
                            success: Some(format!(
                                "Status code: {}",
                                ok_result.output.get_status_code()
                            )),
                            fail: None,
                            ip: Some(ip),
                        })
                        .await;
                }
            }

            Ok(ok_result.into())
        }
        Err(err_result) => {
            if err_result.write_telemetry {
                if err_result.write_to_log {
                    let mut ctx = HashMap::new();
                    ctx.insert("path".to_string(), path.to_string());
                    ctx.insert("method".to_string(), method.to_string());
                    ctx.insert("ip".to_string(), ip.to_string());
                    ctx.insert("httpCode".to_string(), err_result.status_code.to_string());

                    if let Some(credentials) = &request_ctx.credentials {
                        ctx.insert("client_id".to_string(), credentials.get_id().to_string());
                    }

                    logger.write_warning(
                        "HttpRequest".to_string(),
                        format!(
                            "Http request finished with error: {}",
                            get_error_text(&err_result)
                        ),
                        Some(ctx),
                    );
                }

                #[cfg(feature = "my-telemetry")]
                if my_telemetry::TELEMETRY_INTERFACE.is_telemetry_set_up() {
                    my_telemetry::TELEMETRY_INTERFACE
                        .write_telemetry_event(TelemetryEvent {
                            process_id: process_id,
                            started: started.unix_microseconds,
                            finished: DateTimeAsMicroseconds::now().unix_microseconds,
                            data: format!("[{}]{}", method, path),
                            success: None,

                            fail: Some(format!("Status code: {}", err_result.status_code)),
                            ip: Some(ip),
                        })
                        .await;
                }
            }
            Ok(err_result.into())
        }
    }
}

async fn shutdown_signal(app: Arc<dyn ApplicationStates + Send + Sync + 'static>) {
    let duration = Duration::from_secs(1);
    while !app.is_shutting_down() {
        tokio::time::sleep(duration).await;
    }
}

fn get_error_text(err: &HttpFailResult) -> &str {
    if err.content.len() > 256 {
        std::str::from_utf8(&err.content[..256]).unwrap()
    } else {
        std::str::from_utf8(&err.content).unwrap()
    }
}
