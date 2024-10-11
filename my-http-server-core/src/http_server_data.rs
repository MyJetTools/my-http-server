use std::sync::Arc;

use crate::{HttpServerMiddleware, HttpServerTechMiddleware};

pub struct HttpServerMiddlewares {
    pub middlewares: Vec<Arc<dyn HttpServerMiddleware + Send + Sync + 'static>>,
    pub tech_middlewares: Vec<Arc<dyn HttpServerTechMiddleware + Send + Sync + 'static>>,
}
