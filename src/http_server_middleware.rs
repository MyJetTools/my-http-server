use crate::{request_flow::HttpServerRequestFlow, HttpContext, HttpFailResult, HttpOkResult};
use async_trait::async_trait;

use crate::RequestCredentials;

#[async_trait]
pub trait HttpServerMiddleware {
    type TRequestCredentials: RequestCredentials + Send + Sync + 'static;
    async fn handle_request(
        &self,
        ctx: &mut HttpContext<Self::TRequestCredentials>,
        get_next: &mut HttpServerRequestFlow<Self::TRequestCredentials>,
    ) -> Result<HttpOkResult, HttpFailResult>;
}
