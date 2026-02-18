use std::collections::HashMap;

use my_http_server_core::*;
use rust_extensions::StrOrString;

use crate::{calc_etag, EtagCaches, FilesAccess, FilesMapping, NoCache, RootPaths};

pub struct StaticFilesMiddleware {
    pub file_folders: Vec<FilesMapping>,
    pub index_paths: RootPaths,
    pub index_files: Vec<StrOrString<'static>>,
    pub not_found_file: Option<String>,
    pub files_access: FilesAccess,
    pub headers: HashMap<String, String>,
    etag_caches: Option<EtagCaches>,
    no_cache: NoCache,
}

impl StaticFilesMiddleware {
    pub const DEFAULT_FOLDER: &'static str = "./wwwroot";
    pub fn new() -> Self {
        /*
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
        */
        Self {
            file_folders: Default::default(),
            index_files: Default::default(),
            not_found_file: None,
            files_access: FilesAccess::new(),
            index_paths: Default::default(),
            headers: HashMap::new(),
            etag_caches: Default::default(),
            no_cache: Default::default(),
        }
    }

    pub fn add_index_file(mut self, str: impl Into<StrOrString<'static>>) -> Self {
        self.index_files.push(str.into());
        self
    }

    pub fn add_file_mapping(mut self, str: impl Into<StrOrString<'static>>) -> Self {
        self.index_files.push(str.into());
        self
    }

    pub fn with_etag(mut self) -> Self {
        self.etag_caches = Some(Default::default());
        self
    }

    pub fn add_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    pub fn add_index_path(mut self, path: &'static str) -> Self {
        self.index_paths.add(path);
        self
    }

    pub fn enable_files_caching(mut self) -> Self {
        self.files_access.enable_caching();
        self
    }

    pub fn set_path_not_to_cache(mut self, path: impl Into<String>) -> Self {
        self.no_cache.add_path(path.into());
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
        http_path: &HttpPath,
        segment: usize,
        etag_header: Option<&str>,
    ) -> Option<Result<HttpOkResult, HttpFailResult>> {
        let path = http_path.as_str_from_segment(segment);
        if self.index_paths.is_my_path(path) {
            for index_file in self.index_files.iter() {
                let file_name = get_file_name(file_folder, index_file.as_str());

                if let Ok(file_content) = self.files_access.get(file_name.as_str()).await {
                    return Some(self.compile_response(http_path, path, file_content).await);
                }
            }
        }

        let file = get_file_name(file_folder, path);

        match self.files_access.get(file.as_str()).await {
            Ok(file_content) => {
                let result = self.compile_response(http_path, path, file_content).await;
                return Some(result);
            }
            Err(_) => {
                return self.handle_not_found(file_folder, etag_header).await;
            }
        }
    }

    async fn handle_not_found(
        &self,
        file_folder: &str,
        etag_header: Option<&str>,
    ) -> Option<Result<HttpOkResult, HttpFailResult>> {
        let not_found_file = self.not_found_file.as_ref()?;
        let file = get_file_name(file_folder, not_found_file);

        match self.files_access.get(file.as_str()).await {
            Ok(file_content) => {
                let etag = calc_etag(&file_content);

                if let Some(etag_header) = etag_header {
                    if etag == etag_header {
                        return Some(HttpOutput::as_not_modified().into_ok_result(false));
                    }
                }

                let result = HttpOutput::from_builder()
                    .add_headers_opt(self.get_headers())
                    .add_header("ETag", etag)
                    .add_header("Cache-Control", "no-cache, no-store, must-revalidate")
                    .add_header("Pragma", "no-cache")
                    .add_header("Expires", "0")
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

    async fn compile_response(
        &self,
        http_path: &HttpPath,
        path: &str,
        file_content: Vec<u8>,
    ) -> Result<HttpOkResult, HttpFailResult> {
        let (etag, cache_control, pragma, expires) = if let Some(etag_cache) =
            self.etag_caches.as_ref()
        {
            let etag = calc_etag(file_content.as_slice());
            etag_cache.set(http_path, etag.clone()).await;

            let (cache_control, pragma, expires) = if self.no_cache.marked_as_no_cache(http_path) {
                (
                    "no-cache, no-store, must-revalidate",
                    Some("no-cache"),
                    Some("0"),
                )
            } else {
                ("no-cache", None, None)
            };

            (Some(etag), Some(cache_control), pragma, expires)
        } else {
            (None, None, None, None)
        };

        let result = HttpOutput::from_builder()
            .add_headers_opt(self.get_headers())
            .add_header_if_some("ETag", etag)
            .add_header_if_some("Cache-Control", cache_control)
            .add_header_if_some("Pragma", pragma)
            .add_header_if_some("Expires", expires)
            .set_content_type_opt(WebContentType::detect_by_extension(path))
            .set_content(file_content)
            .into_ok_result(false);

        return result;
    }
}

#[async_trait::async_trait]
impl HttpServerMiddleware for StaticFilesMiddleware {
    async fn handle_request(
        &self,
        ctx: &mut HttpContext,
    ) -> Option<Result<HttpOkResult, HttpFailResult>> {
        let path = &ctx.request.http_path;

        let mut etag_header = None;
        if let Some(etag) = ctx
            .request
            .get_headers()
            .try_get_case_insensitive("if-none-match")
        {
            if let Ok(etag) = etag.as_str() {
                etag_header = Some(etag);
                if let Some(etag_cache) = self.etag_caches.as_ref() {
                    if etag_cache.check_etag(path, etag).await {
                        return Some(HttpOutput::as_not_modified().build().into_ok_result(false));
                    }
                }
            }
        }

        for mapping in self.file_folders.iter() {
            if ctx.request.http_path.is_starting_with(&mapping.uri_prefix) {
                /*

                */

                if let Some(result) = self
                    .handle_folder(
                        mapping.folder_path.as_str(),
                        path,
                        mapping.uri_prefix.segments_amount(),
                        etag_header,
                    )
                    .await
                {
                    return Some(result);
                }
            }
        }

        if let Some(result) = self
            .handle_folder(Self::DEFAULT_FOLDER, path, 0, etag_header)
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
    let path_ends = file_folder.ends_with('/');
    let file_starts = path.starts_with('/');

    if path_ends && file_starts {
        return format!("{}{}", &file_folder[..file_folder.len() - 1], path);
    }
    if path_ends && !file_starts {
        return format!("{}{}", file_folder, path);
    }
    if !path_ends && file_starts {
        return format!("{}{}", file_folder, path);
    }

    format!("{}/{}", file_folder, path)
}
