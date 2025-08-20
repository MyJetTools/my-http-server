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
    pub content_type: Option<String>,
    pub content_length: usize,
    pub has_error: bool,
}

impl ResponseData {
    pub fn from(result: &Result<HttpOkResult, HttpFailResult>) -> Self {
        let output = match result {
            Ok(ok) => &ok.output,
            Err(err) => &err.output,
        };

        Self {
            status_code: output.get_status_code(),
            content_type: output
                .get_content_type_as_str()
                .map(|itm| itm.to_string().into()),

            content_length: output.get_content_size(),
            has_error: false,
        }
    }
}

#[async_trait]
pub trait HttpServerTechMiddleware {
    async fn got_result(&self, request: &HttpRequestData, http_result: &ResponseData);
}
