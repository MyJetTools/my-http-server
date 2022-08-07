use crate::{
    request_flow::HttpServerRequestFlow, HttpContext, HttpFailResult, HttpOkResult, HttpOutput,
    HttpServerMiddleware, WebContentType,
};

pub struct StaticFilesMiddleware {
    pub file_folder: String,
    pub index_files: Option<Vec<String>>,
}

impl StaticFilesMiddleware {
    pub fn new(file_folder: Option<&str>, index_files: Option<Vec<String>>) -> Self {
        let index_files = if let Some(index_file_to_check) = index_files {
            let mut index_files_result = Vec::with_capacity(index_file_to_check.len());

            for index_file in index_file_to_check {
                if index_file.starts_with('/') {
                    index_files_result.push(index_file);
                } else {
                    index_files_result.push(format!("/{}", index_file));
                }
            }

            Some(index_files_result)
        } else {
            None
        };

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
        let path = ctx.request.get_path_lower_case();

        if path == "/" {
            if let Some(index_files) = &self.index_files {
                for index_file in index_files {
                    let file_name = get_file_name(self.file_folder.as_str(), index_file);
                    if let Ok(file_content) = super::files::get(file_name.as_str()).await {
                        let output = HttpOutput::Content {
                            headers: None,
                            content_type: WebContentType::detect_by_extension(path),
                            content: file_content,
                        };

                        return Ok(HttpOkResult {
                            write_telemetry: false,
                            output,
                        });
                    }
                }
            }
        }

        let file = get_file_name(self.file_folder.as_str(), path);

        match super::files::get(file.as_str()).await {
            Ok(file_content) => {
                let output = HttpOutput::Content {
                    headers: None,
                    content_type: WebContentType::detect_by_extension(path),
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

fn get_file_name(file_folder: &str, path: &str) -> String {
    format!("{}{}", file_folder, path)
}
