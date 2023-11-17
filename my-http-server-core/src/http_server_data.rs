use std::sync::Arc;

use crate::HttpServerMiddleware;

pub struct HttpServerMiddlewares {
    pub middlewares: Vec<Arc<dyn HttpServerMiddleware + Send + Sync + 'static>>,
}
