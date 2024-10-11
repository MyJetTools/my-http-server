use crate::{HttpContext, HttpFailResult, HttpOkResult};
use async_trait::async_trait;
use hyper::Method;
use rust_extensions::date_time::DateTimeAsMicroseconds;

#[async_trait]
pub trait HttpServerMiddleware {
    async fn handle_request(
        &self,
        ctx: &mut HttpContext,
    ) -> Option<Result<HttpOkResult, HttpFailResult>>;
}

pub struct HttpRequestData {
    pub started: DateTimeAsMicroseconds,
    pub method: Method,
    pub path: String,
    pub ip: String,
}

pub struct ResponseData {
    pub status_code: u16,
    pub content_type: String,
    pub content_length: usize,
    pub has_error: bool,
}

impl ResponseData {
    pub fn from(result: &Result<HttpOkResult, HttpFailResult>) -> Self {
        match result {
            Ok(ok) => Self {
                status_code: ok.output.get_status_code(),
                content_type: ok.output.get_content_type().to_string(),
                content_length: ok.output.get_content_size(),
                has_error: false,
            },
            Err(fail) => Self {
                status_code: 500,
                content_type: fail.content_type.as_str().to_string(),
                content_length: 0,
                has_error: true,
            },
        }
    }
}

#[async_trait]
pub trait HttpServerTechMiddleware {
    async fn got_result(&self, request: &HttpRequestData, http_result: &ResponseData);
}
