use std::{net::SocketAddr, sync::Arc};

use my_http_server::controllers::swagger::SwaggerMiddleware;
use my_http_server::MyHttpServer;
use rust_extensions::AppStates;

pub fn setup_server(app_states: Arc<AppStates>) {
    let mut http_server = MyHttpServer::new(SocketAddr::from(([0, 0, 0, 0], 8000)));

    let controllers = Arc::new(super::builder::build_controllers());

    let swagger_middleware =
        SwaggerMiddleware::new(controllers.clone(), "Test".to_string(), "0.1.0".to_string());

    http_server.add_middleware(Arc::new(swagger_middleware));

    http_server.add_middleware(controllers);

    http_server.start(app_states, my_logger::LOGGER.clone());
}
