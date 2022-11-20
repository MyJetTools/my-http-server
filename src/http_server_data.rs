use std::sync::Arc;

use crate::HttpServerMiddleware;

pub struct HttpServerData {
    pub middlewares: Vec<Arc<dyn HttpServerMiddleware + Send + Sync + 'static>>,
}
