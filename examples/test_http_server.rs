use std::{sync::Arc, net::SocketAddr, time::Duration};

use my_http_server::middlewares::healthcheck::healthcheck_middleware::HealthcheckMiddleware;
use my_http_server::MyHttpServer;

mod app;

#[tokio::main]
async fn main() {
    let app = crate::app::AppContext::new();
    let app = Arc::new(app);

    let mut http_server: MyHttpServer = MyHttpServer::new(SocketAddr::from(([0, 0, 0, 0], 5000)));

    http_server.add_middleware(Arc::new(HealthcheckMiddleware::new()));

    http_server.start(app);

    loop {
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
