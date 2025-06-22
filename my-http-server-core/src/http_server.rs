use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::StatusCode;
use hyper_util::rt::{TokioExecutor, TokioIo};
#[cfg(feature = "with-telemetry")]
use my_telemetry::TelemetryEventTagsBuilder;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use rust_extensions::{ApplicationStates, Logger};
use std::sync::atomic::AtomicI64;
use std::{collections::HashMap, net::SocketAddr};
use tokio::sync::Mutex;

use std::sync::Arc;

use crate::HttpOkResult;
use crate::{
    HttpContext, HttpFailResult, HttpRequest, HttpServerMiddleware, HttpServerMiddlewares,
};

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

pub struct MyHttpServer {
    pub addr: SocketAddr,
    middlewares: Option<Vec<Arc<dyn HttpServerMiddleware + Send + Sync + 'static>>>,
    tech_middlewares: Option<Vec<Arc<dyn HttpServerTechMiddleware + Send + Sync + 'static>>>,
    connections: Arc<AtomicI64>,
}

impl MyHttpServer {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            middlewares: Some(Vec::new()),
            tech_middlewares: Some(Vec::new()),
            connections: Arc::new(AtomicI64::new(0)),
        }
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
            format!("Http server starts at: {:?}", self.addr),
            None,
        );

        let http_server_middlewares = HttpServerMiddlewares {
            middlewares: middlewares.unwrap(),
            tech_middlewares: self.tech_middlewares.take().unwrap(),
        };

        let connections = self.connections.clone();
        tokio::spawn(start_http_1(
            self.addr.clone(),
            Arc::new(http_server_middlewares),
            app_states,
            logger,
            connections,
        ));
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
        };

        let connections = self.connections.clone();
        tokio::spawn(start_http_2(
            self.addr.clone(),
            Arc::new(http_server_middlewares),
            app_states,
            logger,
            connections,
        ));
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

    let mut http1 = http1::Builder::new();
    http1.keep_alive(true);
    loop {
        if app_states.is_shutting_down() {
            break;
        }

        let (stream, socket_addr) = listener.accept().await.unwrap();

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
            if let Err(err) = connection.await {
                println!("Error serving connection: {:?}", err);
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

    let http2_builder = Arc::new(hyper::server::conn::http2::Builder::new(
        TokioExecutor::new(),
    ));

    loop {
        if app_states.is_shutting_down() {
            break;
        }

        let (stream, socket_addr) = listener.accept().await.unwrap();

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
                    socket_addr.clone(),
                    logger.clone(),
                    app_states.is_shutting_down(),
                );
                resp
            }),
        );

        let connections_clone = connections.clone();
        tokio::task::spawn(async move {
            if let Err(err) = connection.await {
                println!("Error serving connection: {:?}", err);
            }

            connections_clone.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        });
    }
}

pub async fn handle_requests(
    req: hyper::Request<hyper::body::Incoming>,
    http_server_middlewares: Arc<HttpServerMiddlewares>,
    addr: SocketAddr,
    logger: Arc<dyn Logger + Send + Sync + 'static>,
    app_is_shutting_down: bool,
) -> hyper::Result<my_hyper_utils::MyHttpResponse> {
    if app_is_shutting_down {
        return compile_app_is_shutting_down_http_response();
    }

    let req = HttpRequest::new(req, addr);

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
    let flow_execution_result = tokio::spawn(async move {
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
    });

    let flow_execution_result = match flow_execution_result.await {
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
        Err(err) => {
            let request_data = Arc::new(request_data);
            let request_data_cloned = request_data.clone();
            tokio::spawn(async move {
                for middleware in http_server_middlewares.tech_middlewares.iter() {
                    middleware
                        .got_result(
                            &request_data_cloned,
                            &ResponseData {
                                status_code: PANIC_HTTP_CODE.as_u16(),
                                content_type: "text/plain".to_string(),
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

                my_telemetry::TELEMETRY_INTERFACE
                    .write_fail(
                        &ctx,
                        request_data.started,
                        format!("[{}]{}", request_data.method, request_data.path.to_string()),
                        format!("Panic: {:?}", err),
                        tags.build(),
                    )
                    .await;
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
                format!("Panic: {:?}", err),
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

                    my_telemetry::TELEMETRY_INTERFACE
                        .write_success(
                            &ctx,
                            request_data.started,
                            telemetry_data,
                            format!("Status code: {}", ok_result.output.get_status_code()),
                            tags.into(),
                        )
                        .await;
                }
            }

            Ok(ok_result.into())
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
                    ctx.insert("httpCode".to_string(), err_result.status_code.to_string());

                    if let Some(credentials) = &flow_execution_result.http_context.credentials {
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

                    my_telemetry::TELEMETRY_INTERFACE
                        .write_fail(
                            &ctx,
                            request_data.started,
                            telemetry_data,
                            format!("Status code: {}", err_result.status_code),
                            tags.into(),
                        )
                        .await;
                }
            }

            Ok(err_result.into())
        }
    }
}

fn get_error_text(err: &HttpFailResult) -> &str {
    if err.content.len() > 256 {
        std::str::from_utf8(&err.content[..256]).unwrap()
    } else {
        std::str::from_utf8(&err.content).unwrap()
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
