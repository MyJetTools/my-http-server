use std::sync::Arc;

use crate::{HttpContext, HttpFailResult, HttpOkResult, HttpServerMiddleware};

pub struct HttpServerRequestFlow {
    middlewares: Vec<Arc<dyn HttpServerMiddleware + Send + Sync + 'static>>,
}

impl HttpServerRequestFlow {
    pub fn new(middlewares: Vec<Arc<dyn HttpServerMiddleware + Send + Sync + 'static>>) -> Self {
        Self { middlewares }
    }
    pub async fn next(&mut self, ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        if self.middlewares.is_empty() {
            let not_found = HttpFailResult::as_not_found("Page not found".to_string(), false);

            return Err(not_found);
        }

        let middleware = self.middlewares.remove(0);
        return middleware.handle_request(ctx, self).await;
    }
}
