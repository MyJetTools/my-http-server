use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::{service::service_fn, Response};
use hyper_util::rt::TokioIo;
#[cfg(feature = "with-telemetry")]
use my_telemetry::TelemetryEventTagsBuilder;
#[cfg(feature = "with-telemetry")]
use rust_extensions::date_time::DateTimeAsMicroseconds;
use rust_extensions::{ApplicationStates, Logger};
use std::{collections::HashMap, net::SocketAddr, time::Duration};

use std::sync::Arc;

use crate::MyHttpServerHyperRequest;
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
    /*
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
    */

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    loop {
        let (stream, socket_addr) = listener.accept().await.unwrap();
        let io = TokioIo::new(stream);
        let http_server_data = http_server_data.clone();
        let logger: Arc<dyn Logger + Send + Sync> = logger.clone();

        tokio::task::spawn(async move {
            let http_server_data = http_server_data.clone();
            let logger = logger.clone();
            let socket_addr = socket_addr.clone();

            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(move |req| {
                        let req = MyHttpServerHyperRequest::new(req);
                        let resp = handle_requests(
                            req,
                            http_server_data.clone(),
                            socket_addr.clone(),
                            logger.clone(),
                        );
                        resp
                    }),
                )
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}

pub async fn handle_requests(
    req: MyHttpServerHyperRequest,
    http_server_data: Arc<HttpServerData>,
    addr: SocketAddr,
    logger: Arc<dyn Logger + Send + Sync + 'static>,
) -> hyper::Result<Response<Full<Bytes>>> {
    let req = HttpRequest::new(req, addr);
    let mut request_ctx = HttpContext::new(req);

    #[cfg(feature = "with-telemetry")]
    let ctx = request_ctx.telemetry_context.clone();

    let path = request_ctx.request.get_path().to_string();
    let method = request_ctx.request.get_method().to_string();
    let ip = request_ctx.request.get_ip().to_string();

    #[cfg(feature = "with-telemetry")]
    let started = DateTimeAsMicroseconds::now();

    let result = tokio::spawn(async move {
        let mut flows = HttpServerRequestFlow::new(http_server_data.middlewares.clone());
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
                    format!("[{}]{}", method, path),
                    format!("Panic: {:?}", err),
                    TelemetryEventTagsBuilder::new()
                        .add_ip(ip.to_string())
                        .build(),
                )
                .await;

            let mut ctx = HashMap::new();
            ctx.insert("path".to_string(), path.to_string());
            ctx.insert("method".to_string(), method.to_string());
            ctx.insert("ip".to_string(), ip);

            logger.write_error(
                "HttpRequest".to_string(),
                format!("Panic: {:?}", err),
                Some(ctx),
            );

            panic!("Http Server error: [{}]{}", method, path);
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
                            format!("[{}]{}", method, path),
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
                            format!("[{}]{}", method, path),
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
