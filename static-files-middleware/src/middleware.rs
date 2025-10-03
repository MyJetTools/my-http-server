use std::collections::HashMap;

use my_http_server_core::{
    AddHttpHeaders, HttpContext, HttpFailResult, HttpOkResult, HttpOutput, HttpPath,
    HttpServerMiddleware, WebContentType,
};

use crate::FilesAccess;

pub struct FilesMapping {
    pub uri_prefix: HttpPath,
    pub folder_path: String,
}

impl FilesMapping {
    pub fn new(uri_prefix: &str, folder_path: &str) -> Self {
        Self {
            uri_prefix: HttpPath::from_str(uri_prefix),
            folder_path: folder_path.to_string(),
        }
    }

    pub fn to_lowercase(&mut self) {
        self.folder_path = self.folder_path.to_lowercase();
    }
}

pub struct StaticFilesMiddleware {
    pub file_folders: Option<Vec<FilesMapping>>,
    pub index_files: Option<Vec<String>>,
    pub not_found_file: Option<String>,
    pub files_access: FilesAccess,
    pub headers: HashMap<String, String>,
}

impl StaticFilesMiddleware {
    pub const DEFAULT_FOLDER: &'static str = "./wwwroot";
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
            not_found_file: None,
            files_access: FilesAccess::new(),
            headers: HashMap::new(),
        }
    }

    pub fn add_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    pub fn enable_files_caching(mut self) -> Self {
        self.files_access.enable_caching();
        self
    }

    pub fn set_not_found_file(mut self, file_name: String) -> Self {
        if file_name.starts_with('/') {
            self.not_found_file = Some(file_name);
        } else {
            self.not_found_file = Some(format!("/{}", file_name));
        }
        self
    }

    fn get_headers<'s>(&'s self) -> Option<impl Iterator<Item = (&'s str, &'s str)>> {
        if self.headers.is_empty() {
            None
        } else {
            Some(
                self.headers
                    .iter()
                    .map(|itm| (itm.0.as_str(), itm.1.as_str())),
            )
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

                    if let Ok(file_content) = self.files_access.get(file_name.as_str()).await {
                        let result = HttpOutput::from_builder()
                            .add_headers_opt(self.get_headers())
                            .set_content_type_opt(WebContentType::detect_by_extension(path))
                            .set_content(file_content)
                            .into_ok_result(false);

                        return Some(result);
                    }
                }
            }
        }

        let file = get_file_name(file_folder, path);

        match self.files_access.get(file.as_str()).await {
            Ok(file_content) => {
                let result = HttpOutput::from_builder()
                    .add_headers_opt(self.get_headers())
                    .set_content_type_opt(WebContentType::detect_by_extension(path))
                    .set_content(file_content)
                    .into_ok_result(false);

                return Some(result);
            }
            Err(_) => {
                return self.handle_not_found(file_folder).await;
            }
        }
    }

    async fn handle_not_found(
        &self,
        file_folder: &str,
    ) -> Option<Result<HttpOkResult, HttpFailResult>> {
        let not_found_file = self.not_found_file.as_ref()?;
        let file = get_file_name(file_folder, not_found_file);

        match self.files_access.get(file.as_str()).await {
            Ok(file_content) => {
                let result = HttpOutput::from_builder()
                    .add_headers_opt(self.get_headers())
                    .set_content_type_opt(WebContentType::detect_by_extension(not_found_file))
                    .set_content(file_content)
                    .into_ok_result(false);

                return Some(result);
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
    ) -> Option<Result<HttpOkResult, HttpFailResult>> {
        if let Some(mappings) = &self.file_folders {
            for mapping in mappings {
                if ctx.request.http_path.is_starting_with(&mapping.uri_prefix) {
                    let path = ctx
                        .request
                        .http_path
                        .as_str_from_segment(mapping.uri_prefix.segments_amount());

                    if let Some(result) =
                        self.handle_folder(mapping.folder_path.as_str(), path).await
                    {
                        return Some(result);
                    }
                }
            }
        }

        if let Some(result) = self
            .handle_folder(
                Self::DEFAULT_FOLDER,
                ctx.request.http_path.as_str_from_segment(0),
            )
            .await
        {
            return Some(result);
        }

        None
    }
}

impl AddHttpHeaders for StaticFilesMiddleware {
    fn add_header(&mut self, header_name: impl Into<String>, header_value: impl Into<String>) {
        self.headers.insert(header_name.into(), header_value.into());
    }
}

fn get_file_name(file_folder: &str, path: &str) -> String {
    format!("{}{}", file_folder, path)
}
