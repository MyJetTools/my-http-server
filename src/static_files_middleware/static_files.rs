use crate::{
    request_flow::HttpServerRequestFlow, HttpContext, HttpFailResult, HttpOkResult, HttpOutput,
    HttpServerMiddleware, WebContentType,
};

pub struct FilesMapping {
    pub uri_prefix: String,
    pub folder_path: String,
}

impl FilesMapping {
    pub fn to_lowercase(&mut self) {
        self.uri_prefix = self.uri_prefix.to_lowercase();
        self.folder_path = self.folder_path.to_lowercase();
    }
}

pub struct StaticFilesMiddleware {
    pub file_folders: Option<Vec<FilesMapping>>,
    pub index_files: Option<Vec<String>>,
}

impl StaticFilesMiddleware {
    pub fn new(mappings: Option<Vec<FilesMapping>>, index_files: Option<Vec<String>>) -> Self {
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

        let file_folders = if let Some(mut mappings) = mappings {
            for mapping in &mut mappings {
                mapping.to_lowercase();
            }

            Some(mappings)
        } else {
            None
        };

        Self {
            file_folders,
            index_files,
        }
    }

    async fn handle_folder(
        &self,
        file_folder: &str,
        path: &str,
    ) -> Option<Result<HttpOkResult, HttpFailResult>> {
        if path == "/" {
            if let Some(index_files) = &self.index_files {
                for index_file in index_files {
                    let file_name = get_file_name(file_folder, index_file);
                    if let Ok(file_content) = super::files::get(file_name.as_str()).await {
                        let output = HttpOutput::Content {
                            headers: None,
                            content_type: WebContentType::detect_by_extension(path),
                            content: file_content,
                        };

                        return Some(Ok(HttpOkResult {
                            write_telemetry: false,
                            output,
                        }));
                    }
                }
            }
        }

        let file = get_file_name(file_folder, path);

        match super::files::get(file.as_str()).await {
            Ok(file_content) => {
                let output = HttpOutput::Content {
                    headers: None,
                    content_type: WebContentType::detect_by_extension(path),
                    content: file_content,
                };

                return Some(Ok(HttpOkResult {
                    write_telemetry: false,
                    output,
                }));
            }
            Err(_) => {
                return None;
            }
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

        if let Some(mappings) = &self.file_folders {
            for mapping in mappings {
                if path.starts_with(mapping.uri_prefix.as_str()) {
                    if let Some(result) = self
                        .handle_folder(&mapping.folder_path[mapping.uri_prefix.len()..], path)
                        .await
                    {
                        return result;
                    }
                }
            }
        }

        if let Some(result) = self.handle_folder(super::files::DEFAULT_FOLDER, path).await {
            return result;
        }

        get_next.next(ctx).await
    }
}

fn get_file_name(file_folder: &str, path: &str) -> String {
    format!("{}{}", file_folder, path)
}
