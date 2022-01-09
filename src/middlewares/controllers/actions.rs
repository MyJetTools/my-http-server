use crate::{HttpContext, HttpFailResult, HttpOkResult};
use async_trait::async_trait;

use super::documentation::HttpActionDescription;

#[async_trait]
pub trait GetAction {
    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult>;
    fn get_description(&self) -> Option<HttpActionDescription>;
}

#[async_trait]
pub trait PostAction {
    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult>;
    fn get_description(&self) -> Option<HttpActionDescription>;
}

#[async_trait]
pub trait PutAction {
    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult>;
    fn get_description(&self) -> Option<HttpActionDescription>;
}

#[async_trait]
pub trait DeleteAction {
    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult>;
    fn get_description(&self) -> Option<HttpActionDescription>;
}
