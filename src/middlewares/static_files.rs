use crate::{
    http_ok_result::HttpContent, request_flow::HttpServerRequestFlow, HttpContext, HttpFailResult,
    HttpOkResult, HttpServerMiddleware,
};
use async_trait::async_trait;

pub struct StaticFilesMiddleware {
    pub file_folder: String,
}

impl StaticFilesMiddleware {
    pub fn new(file_folder: Option<&str>) -> Self {
        let file_folder = if let Some(file_folder) = file_folder {
            file_folder.to_lowercase()
        } else {
            super::files::DEFAULT_FOLDER.to_string()
        };

        Self { file_folder }
    }
}

#[async_trait]

impl HttpServerMiddleware for StaticFilesMiddleware {
    async fn handle_request(
        &self,
        ctx: &mut HttpContext,
        get_next: &mut HttpServerRequestFlow,
    ) -> Result<HttpOkResult, HttpFailResult> {
        let file = format!("{}{}", self.file_folder, ctx.request.get_path_lower_case());

        match super::files::get(file.as_str()).await {
            Ok(file_content) => {
                let content = HttpContent::Content {
                    headers: None,
                    content_type: None,
                    content: file_content,
                };

                return Ok(HttpOkResult {
                    write_telemetry: true,
                    content,
                });
            }
            Err(_) => {
                return get_next.next(ctx).await;
            }
        }
    }
}
