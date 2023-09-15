use crate::{request_flow::HttpServerRequestFlow, HttpContext, HttpFailResult, HttpOkResult};
use async_trait::async_trait;

#[async_trait]
pub trait HttpServerMiddleware {
    async fn handle_request(
        &self,
        ctx: &mut HttpContext,
        get_next: &mut HttpServerRequestFlow,
    ) -> Result<HttpOkResult, HttpFailResult>;
}
