use crate::{
    request_flow::HttpServerRequestFlow, HttpContext, HttpFailResult, HttpOkResult, HttpOutput,
    HttpServerMiddleware,
};

pub struct StaticFilesMiddleware {
    pub file_folder: String,
    pub index_files: Option<Vec<String>>,
}

impl StaticFilesMiddleware {
    pub fn new(file_folder: Option<&str>, index_files: Option<Vec<String>>) -> Self {
        let file_folder = if let Some(file_folder) = file_folder {
            file_folder.to_lowercase()
        } else {
            super::files::DEFAULT_FOLDER.to_string()
        };

        Self {
            file_folder,
            index_files,
        }
    }
}

#[async_trait::async_trait]
impl HttpServerMiddleware for StaticFilesMiddleware {
    async fn handle_request(
        &self,
        ctx: &mut HttpContext,
        get_next: &mut HttpServerRequestFlow,
    ) -> Result<HttpOkResult, HttpFailResult> {
        let path = {
            let mut path = ctx.request.get_path_lower_case();

            if path == "/" {
                if let Some(index_files) = &self.index_files {
                    for index_file in index_files {
                        path = index_file.as_str();
                        break;
                    }
                }
            }

            path
        };

        let file = format!("{}{}", self.file_folder, path);

        match super::files::get(file.as_str()).await {
            Ok(file_content) => {
                let output = HttpOutput::Content {
                    headers: None,
                    content_type: None,
                    content: file_content,
                };

                return Ok(HttpOkResult {
                    write_telemetry: false,
                    output,
                });
            }
            Err(_) => {
                return get_next.next(ctx).await;
            }
        }
    }
}
