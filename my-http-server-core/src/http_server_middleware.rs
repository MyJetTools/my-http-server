use crate::{HttpContext, HttpFailResult, HttpOkResult};
use async_trait::async_trait;

#[async_trait]
pub trait HttpServerMiddleware {
    async fn handle_request(
        &self,
        ctx: &mut HttpContext,
    ) -> Option<Result<HttpOkResult, HttpFailResult>>;
}
