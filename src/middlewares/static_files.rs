use crate::{HttpContext, HttpFailResult, HttpOkResult, HttpServerMiddleware, MiddleWareResult};
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
    async fn handle_request(&self, ctx: HttpContext) -> Result<MiddleWareResult, HttpFailResult> {
        let file = format!("{}{}", self.file_folder, ctx.get_path_lower_case());

        match super::files::get(file.as_str()).await {
            Ok(file_content) => {
                let result = HttpOkResult::Content {
                    content_type: None,
                    content: file_content,
                };

                return Ok(MiddleWareResult::Ok(result));
            }
            Err(_) => {
                return Ok(MiddleWareResult::Next(ctx));
            }
        }
    }
}
