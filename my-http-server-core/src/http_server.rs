use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::{service::service_fn, Response};
use hyper_util::rt::TokioIo;
#[cfg(feature = "with-telemetry")]
use my_telemetry::TelemetryEventTagsBuilder;
#[cfg(feature = "with-telemetry")]
use rust_extensions::date_time::DateTimeAsMicroseconds;
use rust_extensions::{ApplicationStates, Logger, StrOrString};
use std::{collections::HashMap, net::SocketAddr};

use std::sync::Arc;

use crate::{
    request_flow::HttpServerRequestFlow, HttpContext, HttpFailResult, HttpRequest,
    HttpServerMiddleware, HttpServerMiddlewares,
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

        let http_server_middlewares = HttpServerMiddlewares {
            middlewares: middlewares.unwrap(),
        };

        tokio::spawn(start(
            self.addr.clone(),
            Arc::new(http_server_middlewares),
            app_states,
            logger,
        ));
    }
}

pub async fn start(
    addr: SocketAddr,
    http_server_middlewares: Arc<HttpServerMiddlewares>,
    app_states: Arc<dyn ApplicationStates + Send + Sync + 'static>,
    logger: Arc<dyn Logger + Send + Sync + 'static>,
) {
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let mut http1 = http1::Builder::new();
    http1.keep_alive(true);
    loop {
        if app_states.is_shutting_down() {
            break;
        }

        let (stream, socket_addr) = listener.accept().await.unwrap();
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
                    if app_states.is_shutting_down() {
                        panic!("Application is shutting down");
                    }

                    let resp = handle_requests(
                        req,
                        http_server_middlewares.clone(),
                        socket_addr.clone(),
                        logger.clone(),
                    );
                    resp
                }),
            )
            .with_upgrades();

        tokio::task::spawn(async move {
            if let Err(err) = connection.await {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}

pub async fn handle_requests(
    req: hyper::Request<hyper::body::Incoming>,
    http_server_middlewares: Arc<HttpServerMiddlewares>,
    addr: SocketAddr,
    logger: Arc<dyn Logger + Send + Sync + 'static>,
) -> hyper::Result<Response<Full<Bytes>>> {
    let req = HttpRequest::new(req, addr);

    let method = req.method.clone();

    let mut request_ctx = HttpContext::new(req);

    #[cfg(feature = "with-telemetry")]
    let ctx = request_ctx.telemetry_context.clone();

    let path = StrOrString::create_as_short_string_or_string(request_ctx.request.get_path());
    let ip =
        StrOrString::create_as_short_string_or_string(request_ctx.request.get_ip().get_real_ip());

    #[cfg(feature = "with-telemetry")]
    let started = DateTimeAsMicroseconds::now();

    let result = tokio::spawn(async move {
        let mut flows = HttpServerRequestFlow::new(http_server_middlewares.middlewares.clone());
        let result = flows.next(&mut request_ctx).await;
        (result, request_ctx)
    });

    let (result, request_ctx) = match result.await {
        Ok(result) => (result.0, result.1),
        Err(err) => {
            #[cfg(feature = "with-telemetry")]
            my_telemetry::TELEMETRY_INTERFACE
                .write_fail(
                    &ctx,
                    started,
                    format!("[{}]{}", method, path.as_str().to_string()),
                    format!("Panic: {:?}", err),
                    TelemetryEventTagsBuilder::new()
                        .add_ip(ip.as_str().to_string())
                        .build(),
                )
                .await;

            let mut ctx = HashMap::new();
            ctx.insert("path".to_string(), path.as_str().to_string());
            ctx.insert("method".to_string(), method.to_string());
            ctx.insert("ip".to_string(), ip.as_str().to_string());

            logger.write_error(
                "HttpRequest".to_string(),
                format!("Panic: {:?}", err),
                Some(ctx),
            );

            panic!("Http Server error: [{}]{}", method, path.to_string());
        }
    };

    match result {
        Ok(ok_result) => {
            #[cfg(feature = "with-telemetry")]
            let mut ok_result = ok_result;

            if ok_result.write_telemetry {
                #[cfg(feature = "with-telemetry")]
                {
                    let mut tags = ok_result.add_telemetry_tags.take_tags().add_ip(ip);

                    if let Some(credentials) = &request_ctx.credentials {
                        tags = tags.add("user_id", credentials.get_id().to_string());
                    }
                    my_telemetry::TELEMETRY_INTERFACE
                        .write_success(
                            &ctx,
                            started,
                            format!("[{}]{}", method, path.as_str().to_string()),
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
                    ctx.insert("path".to_string(), path.as_str().to_string());
                    ctx.insert("method".to_string(), request_ctx.request.method.to_string());
                    ctx.insert("ip".to_string(), ip.as_str().to_string());
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

                #[cfg(feature = "with-telemetry")]
                {
                    let mut tags = err_result.add_telemetry_tags.take_tags().add_ip(ip);

                    if let Some(credentials) = &request_ctx.credentials {
                        tags = tags.add("user_id", credentials.get_id().to_string());
                    }

                    my_telemetry::TELEMETRY_INTERFACE
                        .write_fail(
                            &ctx,
                            started,
                            format!("[{}]{}", method, path.as_str()),
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
