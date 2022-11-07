use std::sync::Arc;

use crate::{HttpContext, HttpFailResult, HttpOkResult, HttpServerMiddleware};

use crate::RequestCredentials;

pub struct HttpServerRequestFlow<TRequestCredentials: RequestCredentials + Send + Sync + 'static> {
    middlewares: Vec<
        Arc<
            dyn HttpServerMiddleware<TRequestCredentials = TRequestCredentials>
                + Send
                + Sync
                + 'static,
        >,
    >,
}

impl<TRequestCredentials: RequestCredentials + Send + Sync + 'static>
    HttpServerRequestFlow<TRequestCredentials>
{
    pub fn new(
        middlewares: Vec<
            Arc<
                dyn HttpServerMiddleware<TRequestCredentials = TRequestCredentials>
                    + Send
                    + Sync
                    + 'static,
            >,
        >,
    ) -> Self {
        Self { middlewares }
    }
    pub async fn next(
        &mut self,
        ctx: &mut HttpContext<TRequestCredentials>,
    ) -> Result<HttpOkResult, HttpFailResult> {
        if self.middlewares.is_empty() {
            let not_found = HttpFailResult::as_not_found("404 - Not Found".to_string(), false);

            return Err(not_found);
        }

        let middleware = self.middlewares.remove(0);
        let result = middleware.handle_request(ctx, self).await;
        result
    }
}
