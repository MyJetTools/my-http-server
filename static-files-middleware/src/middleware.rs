use my_http_server_core::*;
use rust_extensions::StrOrString;

use crate::{
    calc_etag, deflate_compress, zstd_decompress, CachedContent, EtagCaches, FilesAccess,
    FilesMapping, NoCache, RootPaths,
};

pub struct StaticFilesMiddleware {
    pub file_folders: Vec<FilesMapping>,
    pub index_paths: RootPaths,
    pub index_files: Vec<StrOrString<'static>>,
    pub not_found_file: Option<String>,
    pub files_access: FilesAccess,
    pub headers: Vec<(StrOrString<'static>, String)>,
    etag_caches: Option<EtagCaches>,
    no_cache: NoCache,
}

#[derive(Clone, Copy, Default)]
struct AcceptedEncodings {
    zstd: bool,
    deflate: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ResponseEncoding {
    Identity,
    Zstd,
    Deflate,
}

impl ResponseEncoding {
    fn header_value(self) -> Option<&'static str> {
        match self {
            ResponseEncoding::Identity => None,
            ResponseEncoding::Zstd => Some("zstd"),
            ResponseEncoding::Deflate => Some("deflate"),
        }
    }
}

impl StaticFilesMiddleware {
    pub const DEFAULT_FOLDER: &'static str = "./wwwroot";
    pub fn new() -> Self {
        Self {
            file_folders: Default::default(),
            index_files: Default::default(),
            not_found_file: None,
            files_access: FilesAccess::new(),
            index_paths: Default::default(),
            headers: Default::default(),
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
        self.files_access.enable_etag();
        self
    }

    pub fn add_header(
        mut self,
        name: impl Into<StrOrString<'static>>,
        value: impl Into<String>,
    ) -> Self {
        self.headers.push((name.into(), value.into()));
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

    fn get_headers<'s>(&'s self) -> Option<impl Iterator<Item = (StrOrString<'static>, &'s str)>> {
        if self.headers.is_empty() {
            None
        } else {
            Some(
                self.headers
                    .iter()
                    .map(|itm| (itm.0.clone(), itm.1.as_str())),
            )
        }
    }

    async fn handle_folder(
        &self,
        file_folder: &str,
        http_path: &HttpPath,
        segment: usize,
        etag_header: Option<&str>,
        accepted: AcceptedEncodings,
    ) -> Option<Result<HttpOkResult, HttpFailResult>> {
        let path = http_path.as_str_from_segment(segment);
        if self.index_paths.is_my_path(path) {
            for index_file in self.index_files.iter() {
                let file_name = get_file_name(file_folder, index_file.as_str());

                if let Ok(file_content) = self.files_access.get(file_name.as_str()).await {
                    return Some(
                        self.compile_response(http_path, path, file_content, accepted)
                            .await,
                    );
                }
            }
        }

        let file = get_file_name(file_folder, path);

        match self.files_access.get(file.as_str()).await {
            Ok(file_content) => {
                let result = self
                    .compile_response(http_path, path, file_content, accepted)
                    .await;
                return Some(result);
            }
            Err(_) => {
                return self
                    .handle_not_found(file_folder, etag_header, accepted)
                    .await;
            }
        }
    }

    async fn handle_not_found(
        &self,
        file_folder: &str,
        etag_header: Option<&str>,
        accepted: AcceptedEncodings,
    ) -> Option<Result<HttpOkResult, HttpFailResult>> {
        let not_found_file = self.not_found_file.as_ref()?;
        let file = get_file_name(file_folder, not_found_file);

        if let Some(etag_header) = etag_header {
            if let Some(etag_caches) = self.etag_caches.as_ref() {
                if etag_caches.is_not_found(etag_header).await {
                    return Some(HttpOutput::as_not_modified().into_ok_result(false));
                }
            }
        }

        match self.files_access.get(file.as_str()).await {
            Ok(file_content) => {
                let (body, encoding, etag) = match build_response_body(&file_content, accepted) {
                    Ok(v) => v,
                    Err(e) => {
                        return Some(Err(HttpFailResult::as_fatal_error(format!(
                            "Failed to prepare cached content: {}",
                            e
                        ))));
                    }
                };

                let mut builder = HttpOutput::from_builder()
                    .add_headers_opt(self.get_headers())
                    .add_header("ETag", etag.as_str())
                    .add_header("Cache-Control", "no-cache")
                    .add_header("Vary", "Accept-Encoding")
                    .set_content_type_opt(WebContentType::detect_by_extension(not_found_file));

                if let Some(enc) = encoding.header_value() {
                    builder = builder.add_header("Content-Encoding", enc);
                }

                let result = builder.set_content(body).into_ok_result(false);

                if let Some(etag_caches) = self.etag_caches.as_ref() {
                    etag_caches.set_not_found(etag).await;
                }

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
        file_content: CachedContent,
        accepted: AcceptedEncodings,
    ) -> Result<HttpOkResult, HttpFailResult> {
        let with_etag = self.etag_caches.is_some();

        let (body, encoding, etag_opt) = if with_etag {
            let (b, e, t) = build_response_body(&file_content, accepted).map_err(|err| {
                HttpFailResult::as_fatal_error(format!(
                    "Failed to prepare cached content: {}",
                    err
                ))
            })?;
            (b, e, Some(t))
        } else {
            let (b, e) = body_without_etag(&file_content, accepted).map_err(|err| {
                HttpFailResult::as_fatal_error(format!(
                    "Failed to prepare cached content: {}",
                    err
                ))
            })?;
            (b, e, None)
        };

        let (etag, cache_control, pragma, expires) = if let Some(etag_cache) =
            self.etag_caches.as_ref()
        {
            let etag = etag_opt.expect("etag must be computed when etag_caches is enabled");
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

        let mut builder = HttpOutput::from_builder()
            .add_headers_opt(self.get_headers())
            .add_header_if_some("ETag", etag)
            .add_header_if_some("Cache-Control", cache_control)
            .add_header_if_some("Pragma", pragma)
            .add_header_if_some("Expires", expires)
            .add_header("Vary", "Accept-Encoding")
            .set_content_type_opt(WebContentType::detect_by_extension(path));

        if let Some(enc) = encoding.header_value() {
            builder = builder.add_header("Content-Encoding", enc);
        }

        let result = builder.set_content(body).into_ok_result(false);

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

        let accepted = ctx
            .request
            .get_headers()
            .try_get_case_insensitive("accept-encoding")
            .and_then(|h| h.as_str().ok())
            .map(parse_accept_encoding)
            .unwrap_or_default();

        for mapping in self.file_folders.iter() {
            if ctx.request.http_path.is_starting_with(&mapping.uri_prefix) {
                if let Some(result) = self
                    .handle_folder(
                        mapping.folder_path.as_str(),
                        path,
                        mapping.uri_prefix.segments_amount(),
                        etag_header,
                        accepted,
                    )
                    .await
                {
                    return Some(result);
                }
            }
        }

        if let Some(result) = self
            .handle_folder(Self::DEFAULT_FOLDER, path, 0, etag_header, accepted)
            .await
        {
            return Some(result);
        }

        None
    }
}

impl AddHttpHeaders for StaticFilesMiddleware {
    fn add_header(&mut self, header_name: impl Into<String>, header_value: impl Into<String>) {
        self.headers
            .push((header_name.into().into(), header_value.into()));
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

fn parse_accept_encoding(header_value: &str) -> AcceptedEncodings {
    let mut out = AcceptedEncodings::default();
    for token in header_value.split(',') {
        let token = token.trim();
        let name = match token.split(';').next() {
            Some(n) => n.trim(),
            None => token,
        };
        if name.eq_ignore_ascii_case("zstd") {
            out.zstd = true;
        } else if name.eq_ignore_ascii_case("deflate") {
            out.deflate = true;
        }
    }
    out
}

/// Returns (body, encoding, etag). Used when ETag is required — always
/// materialises raw bytes if needed to compute the checksum.
fn build_response_body(
    cached: &CachedContent,
    accepted: AcceptedEncodings,
) -> std::io::Result<(Vec<u8>, ResponseEncoding, String)> {
    if cached.is_zstd {
        if accepted.zstd {
            let etag = match &cached.etag {
                Some(e) => e.clone(),
                None => calc_etag(&zstd_decompress(&cached.data)?),
            };
            return Ok((cached.data.clone(), ResponseEncoding::Zstd, etag));
        }

        let raw = zstd_decompress(&cached.data)?;
        let etag = match &cached.etag {
            Some(e) => e.clone(),
            None => calc_etag(&raw),
        };

        if accepted.deflate {
            let deflated = deflate_compress(&raw)?;
            return Ok((deflated, ResponseEncoding::Deflate, etag));
        }

        return Ok((raw, ResponseEncoding::Identity, etag));
    }

    let etag = match &cached.etag {
        Some(e) => e.clone(),
        None => calc_etag(&cached.data),
    };
    Ok((cached.data.clone(), ResponseEncoding::Identity, etag))
}

/// Returns (body, encoding). Used when ETag is NOT required — avoids the
/// extra decompression needed for checksum calculation.
fn body_without_etag(
    cached: &CachedContent,
    accepted: AcceptedEncodings,
) -> std::io::Result<(Vec<u8>, ResponseEncoding)> {
    if !cached.is_zstd {
        return Ok((cached.data.clone(), ResponseEncoding::Identity));
    }

    if accepted.zstd {
        return Ok((cached.data.clone(), ResponseEncoding::Zstd));
    }

    let raw = zstd_decompress(&cached.data)?;
    if accepted.deflate {
        let deflated = deflate_compress(&raw)?;
        return Ok((deflated, ResponseEncoding::Deflate));
    }

    Ok((raw, ResponseEncoding::Identity))
}

#[cfg(test)]
mod tests {
    use super::parse_accept_encoding;

    #[test]
    fn parses_zstd() {
        let a = parse_accept_encoding("zstd");
        assert!(a.zstd);
        assert!(!a.deflate);
    }

    #[test]
    fn parses_deflate() {
        let a = parse_accept_encoding("deflate");
        assert!(!a.zstd);
        assert!(a.deflate);
    }

    #[test]
    fn parses_both() {
        let a = parse_accept_encoding("zstd, deflate, br");
        assert!(a.zstd);
        assert!(a.deflate);
    }

    #[test]
    fn respects_case() {
        let a = parse_accept_encoding("Zstd, DEFLATE");
        assert!(a.zstd);
        assert!(a.deflate);
    }

    #[test]
    fn parses_qvalues() {
        let a = parse_accept_encoding("zstd;q=1.0, deflate;q=0.5");
        assert!(a.zstd);
        assert!(a.deflate);
    }

    #[test]
    fn rejects_when_neither() {
        let a = parse_accept_encoding("gzip, br");
        assert!(!a.zstd);
        assert!(!a.deflate);
    }

    #[test]
    fn not_confused_by_substring() {
        let a = parse_accept_encoding("gzip");
        assert!(!a.zstd);
        assert!(!a.deflate);
    }
}
