use crate::{request_flow::HttpServerRequestFlow, HttpContext, HttpFailResult, HttpOkResult};
use async_trait::async_trait;

#[async_trait]
pub trait HttpServerMiddleware {
    async fn handle_request<'s, 'c>(
        &'s self,
        ctx: &'c mut HttpContext<'c>,
        get_next: &'s mut HttpServerRequestFlow,
    ) -> Result<HttpOkResult, HttpFailResult>;
}
