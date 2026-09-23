use std::sync::Arc;

use crate::{HttpServerMiddleware, HttpServerTechMiddleware};

pub struct HttpServerMiddlewares {
    pub middlewares: Vec<Arc<dyn HttpServerMiddleware + Send + Sync + 'static>>,
    pub tech_middlewares: Vec<Arc<dyn HttpServerTechMiddleware + Send + Sync + 'static>>,
    /// Idle timeout for reading a request body — see
    /// [`BodyExpectations::read_timeout`](crate::BodyExpectations::read_timeout). Set through
    /// [`MyHttpServer::set_body_read_timeout`](crate::MyHttpServer::set_body_read_timeout);
    /// `None` waits forever, which is what this server has always done.
    pub body_read_timeout: Option<std::time::Duration>,
    /// How large a compressed request body may grow once decompressed - see
    /// [`BodyExpectations::max_decompressed_body_size`](crate::BodyExpectations::max_decompressed_body_size).
    /// Set through
    /// [`MyHttpServer::set_max_decompressed_body_size`](crate::MyHttpServer::set_max_decompressed_body_size).
    pub max_decompressed_body_size: usize,
}
