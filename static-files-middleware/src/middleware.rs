use my_http_server_core::*;
use rust_extensions::StrOrString;

use crate::{
    calc_etag, deflate_compress, gzip_decompress, CachedContent, EtagCaches, FilesAccess,
    FilesMapping, NoCache, RootPaths, NO_CACHE_CACHE_CONTROL, NO_CACHE_EXPIRES, NO_CACHE_PRAGMA,
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
    gzip: bool,
    deflate: bool,
}

/// Either the path is registered as "never cache it" - and then there is nothing
/// to negotiate with the client, or we are doing a regular ETag negotiation -
/// with or without an `If-None-Match` header from the client.
#[derive(Clone, Copy)]
enum CachePolicy<'s> {
    /// Path is registered with `add_no_cache_headers_to_response_by_path`
    NoCache,
    /// Regular path. Client sent us `If-None-Match`
    IfNoneMatch(&'s str),
    /// Regular path. Client did not send us `If-None-Match`
    Regular,
}

impl<'s> CachePolicy<'s> {
    pub fn get_if_none_match(&self) -> Option<&'s str> {
        match self {
            CachePolicy::IfNoneMatch(etag) => Some(etag),
            _ => None,
        }
    }
}

/// Caching related headers of the response
#[derive(Default)]
struct CacheHeaders {
    etag: Option<String>,
    cache_control: Option<&'static str>,
    pragma: Option<&'static str>,
    expires: Option<&'static str>,
}

impl CacheHeaders {
    /// Client is not allowed to cache the content at all
    pub fn no_cache() -> Self {
        Self {
            etag: None,
            cache_control: Some(NO_CACHE_CACHE_CONTROL),
            pragma: Some(NO_CACHE_PRAGMA),
            expires: Some(NO_CACHE_EXPIRES),
        }
    }

    /// Client caches the content but revalidates it with `If-None-Match` each time
    pub fn with_etag(etag: String) -> Self {
        Self {
            etag: Some(etag),
            cache_control: Some("no-cache"),
            pragma: None,
            expires: None,
        }
    }

    pub fn apply(self, builder: HttpResultBuilder) -> HttpResultBuilder {
        builder
            .add_header_if_some("ETag", self.etag)
            .add_header_if_some("Cache-Control", self.cache_control)
            .add_header_if_some("Pragma", self.pragma)
            .add_header_if_some("Expires", self.expires)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ResponseEncoding {
    Identity,
    Gzip,
    Deflate,
}

impl ResponseEncoding {
    fn header_value(self) -> Option<&'static str> {
        match self {
            ResponseEncoding::Identity => None,
            ResponseEncoding::Gzip => Some("gzip"),
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

    /// Registers a path which must never be cached by the client.
    ///
    /// Path is matched segment by segment (case-insensitive, trailing slash does
    /// not matter) and the query string is ignored - so `/` matches `/?ver=123`.
    ///
    /// For such a path the middleware does not deal with ETag at all: no
    /// `If-None-Match` check, no `304 Not Modified`, no `ETag` header. Instead the
    /// content is always served with the full set of no-cache headers.
    pub fn add_no_cache_headers_to_response_by_path(mut self, path: impl Into<String>) -> Self {
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
        accepted: AcceptedEncodings,
        cache_policy: CachePolicy<'_>,
    ) -> Option<Result<HttpOkResult, HttpFailResult>> {
        let path = http_path.as_str_from_segment(segment);
        if self.index_paths.is_my_path(path) {
            for index_file in self.index_files.iter() {
                let file_name = get_file_name(file_folder, index_file.as_str());

                if let Ok(file_content) = self.files_access.get(file_name.as_str()).await {
                    return Some(
                        self.compile_response(http_path, path, file_content, accepted, cache_policy)
                            .await,
                    );
                }
            }
        }

        let file = get_file_name(file_folder, path);

        match self.files_access.get(file.as_str()).await {
            Ok(file_content) => {
                let result = self
                    .compile_response(http_path, path, file_content, accepted, cache_policy)
                    .await;
                return Some(result);
            }
            Err(_) => {
                return self
                    .handle_not_found(file_folder, accepted, cache_policy)
                    .await;
            }
        }
    }

    async fn handle_not_found(
        &self,
        file_folder: &str,
        accepted: AcceptedEncodings,
        cache_policy: CachePolicy<'_>,
    ) -> Option<Result<HttpOkResult, HttpFailResult>> {
        let not_found_file = self.not_found_file.as_ref()?;
        let file = get_file_name(file_folder, not_found_file);

        if let Some(etag_header) = cache_policy.get_if_none_match() {
            if let Some(etag_caches) = self.etag_caches.as_ref() {
                if etag_caches.is_not_found(etag_header).await {
                    return Some(HttpOutput::as_not_modified().into_ok_result(false));
                }
            }
        }

        let file_content = self.files_access.get(file.as_str()).await.ok()?;

        let (body, encoding, cache_headers) = match cache_policy {
            CachePolicy::NoCache => {
                let (body, encoding) = match body_without_etag(&file_content, accepted) {
                    Ok(result) => result,
                    Err(err) => return Some(Err(content_preparation_error(err))),
                };

                (body, encoding, CacheHeaders::no_cache())
            }
            CachePolicy::IfNoneMatch(_) | CachePolicy::Regular => {
                let (body, encoding, etag) = match build_response_body(&file_content, accepted) {
                    Ok(result) => result,
                    Err(err) => return Some(Err(content_preparation_error(err))),
                };

                if let Some(etag_caches) = self.etag_caches.as_ref() {
                    etag_caches.set_not_found(etag.clone()).await;
                }

                (body, encoding, CacheHeaders::with_etag(etag))
            }
        };

        let mut builder = cache_headers
            .apply(HttpOutput::from_builder().add_headers_opt(self.get_headers()))
            .add_header("Vary", "Accept-Encoding")
            .set_content_type_opt(WebContentType::detect_by_extension(not_found_file));

        if let Some(enc) = encoding.header_value() {
            builder = builder.add_header("Content-Encoding", enc);
        }

        Some(builder.set_content(body).into_ok_result(false))
    }

    async fn compile_response(
        &self,
        http_path: &HttpPath,
        path: &str,
        file_content: CachedContent,
        accepted: AcceptedEncodings,
        cache_policy: CachePolicy<'_>,
    ) -> Result<HttpOkResult, HttpFailResult> {
        let (body, encoding, cache_headers) = match cache_policy {
            CachePolicy::NoCache => {
                let (body, encoding) =
                    body_without_etag(&file_content, accepted).map_err(content_preparation_error)?;

                (body, encoding, CacheHeaders::no_cache())
            }
            CachePolicy::IfNoneMatch(_) | CachePolicy::Regular => match self.etag_caches.as_ref() {
                Some(etag_cache) => {
                    let (body, encoding, etag) = build_response_body(&file_content, accepted)
                        .map_err(content_preparation_error)?;

                    etag_cache.set(http_path, etag.clone()).await;

                    (body, encoding, CacheHeaders::with_etag(etag))
                }
                None => {
                    let (body, encoding) = body_without_etag(&file_content, accepted)
                        .map_err(content_preparation_error)?;

                    (body, encoding, CacheHeaders::default())
                }
            },
        };

        let mut builder = cache_headers
            .apply(HttpOutput::from_builder().add_headers_opt(self.get_headers()))
            .add_header("Vary", "Accept-Encoding")
            .set_content_type_opt(WebContentType::detect_by_extension(path));

        if let Some(enc) = encoding.header_value() {
            builder = builder.add_header("Content-Encoding", enc);
        }

        builder.set_content(body).into_ok_result(false)
    }
}

#[async_trait::async_trait]
impl HttpServerMiddleware for StaticFilesMiddleware {
    async fn handle_request(
        &self,
        ctx: &mut HttpContext,
    ) -> Option<Result<HttpOkResult, HttpFailResult>> {
        let path = &ctx.request.http_path;

        let mut cache_policy = if self.no_cache.marked_as_no_cache(path) {
            CachePolicy::NoCache
        } else {
            CachePolicy::Regular
        };

        if let CachePolicy::Regular = cache_policy {
            if let Some(etag) = ctx
                .request
                .get_headers()
                .try_get_case_insensitive("if-none-match")
            {
                if let Ok(etag) = etag.as_str() {
                    cache_policy = CachePolicy::IfNoneMatch(etag);
                    if let Some(etag_cache) = self.etag_caches.as_ref() {
                        if etag_cache.check_etag(path, etag).await {
                            return Some(
                                HttpOutput::as_not_modified().build().into_ok_result(false),
                            );
                        }
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
                        accepted,
                        cache_policy,
                    )
                    .await
                {
                    return Some(result);
                }
            }
        }

        if let Some(result) = self
            .handle_folder(Self::DEFAULT_FOLDER, path, 0, accepted, cache_policy)
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

fn content_preparation_error(err: std::io::Error) -> HttpFailResult {
    HttpFailResult::as_fatal_error(format!("Failed to prepare cached content: {}", err))
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
        if name.eq_ignore_ascii_case("gzip") {
            out.gzip = true;
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
    if cached.is_gzip {
        if accepted.gzip {
            let etag = match &cached.etag {
                Some(e) => e.clone(),
                None => calc_etag(&gzip_decompress(&cached.data)?),
            };
            return Ok((cached.data.clone(), ResponseEncoding::Gzip, etag));
        }

        let raw = gzip_decompress(&cached.data)?;
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
    if !cached.is_gzip {
        return Ok((cached.data.clone(), ResponseEncoding::Identity));
    }

    if accepted.gzip {
        return Ok((cached.data.clone(), ResponseEncoding::Gzip));
    }

    let raw = gzip_decompress(&cached.data)?;
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
    fn parses_gzip() {
        let a = parse_accept_encoding("gzip");
        assert!(a.gzip);
        assert!(!a.deflate);
    }

    #[test]
    fn parses_deflate() {
        let a = parse_accept_encoding("deflate");
        assert!(!a.gzip);
        assert!(a.deflate);
    }

    #[test]
    fn parses_both() {
        let a = parse_accept_encoding("gzip, deflate, br");
        assert!(a.gzip);
        assert!(a.deflate);
    }

    #[test]
    fn respects_case() {
        let a = parse_accept_encoding("GZip, DEFLATE");
        assert!(a.gzip);
        assert!(a.deflate);
    }

    #[test]
    fn parses_qvalues() {
        let a = parse_accept_encoding("gzip;q=1.0, deflate;q=0.5");
        assert!(a.gzip);
        assert!(a.deflate);
    }

    #[test]
    fn rejects_when_neither() {
        let a = parse_accept_encoding("br, zstd");
        assert!(!a.gzip);
        assert!(!a.deflate);
    }

    #[test]
    fn not_confused_by_substring() {
        let a = parse_accept_encoding("gzip2, deflate-raw");
        assert!(!a.gzip);
        assert!(!a.deflate);
    }
}
