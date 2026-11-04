use my_http_server_core::hyper::{header::IF_NONE_MATCH, HeaderMap};
use my_http_server_core::*;
use rust_extensions::StrOrString;

use crate::{
    calc_etag, deflate_compress, gzip_decompress, has_parent_segment, CachedContent, EtagCaches,
    FilesAccess, FilesMapping, KnownEtag, NoCache, RootPaths, NO_CACHE_CACHE_CONTROL,
    NO_CACHE_EXPIRES, NO_CACHE_PRAGMA,
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
    IfNoneMatch(IfNoneMatch<'s>),
    /// Regular path. Client did not send us `If-None-Match`
    Regular,
}

impl<'s> CachePolicy<'s> {
    /// Whether the client sent an `If-None-Match` which matches `etag`. Never for a no-cache path.
    pub fn if_none_match_matches(&self, etag: &str) -> bool {
        match self {
            CachePolicy::IfNoneMatch(if_none_match) => if_none_match.matches(etag),
            _ => false,
        }
    }
}

/// The `If-None-Match` of the request - all of its lines, matched as one list.
#[derive(Clone, Copy)]
struct IfNoneMatch<'s>(&'s HeaderMap);

impl IfNoneMatch<'_> {
    fn matches(&self, etag: &str) -> bool {
        if_none_match_matches(self.0.get_all(IF_NONE_MATCH), etag)
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
    /// The variant a client gets: the stored gzip as it is if the client takes gzip, re-encoded
    /// as deflate if it takes only that, the raw bytes otherwise. A file which is not stored
    /// gzipped always goes as it is.
    fn select(is_gzip: bool, accepted: AcceptedEncodings) -> Self {
        if !is_gzip {
            return ResponseEncoding::Identity;
        }

        if accepted.gzip {
            ResponseEncoding::Gzip
        } else if accepted.deflate {
            ResponseEncoding::Deflate
        } else {
            ResponseEncoding::Identity
        }
    }

    fn header_value(self) -> Option<&'static str> {
        match self {
            ResponseEncoding::Identity => None,
            ResponseEncoding::Gzip => Some("gzip"),
            ResponseEncoding::Deflate => Some("deflate"),
        }
    }

    /// The `ETag` of this variant of a content. The raw bytes are exactly what the hash was
    /// computed over - a strong `"…"`. A compressed variant is other bytes of the same content:
    /// a strong ETag shared with the raw bytes would claim they are the same bytes, so it goes
    /// weak - `W/"…"`, which is what nginx does to an ETag when it compresses on the fly.
    fn etag(self, opaque_tag: &str) -> String {
        match self {
            ResponseEncoding::Identity => strong_etag(opaque_tag),
            ResponseEncoding::Gzip | ResponseEncoding::Deflate => weak_etag(opaque_tag),
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

        if let (CachePolicy::IfNoneMatch(_), Some(etag_caches)) =
            (cache_policy, self.etag_caches.as_ref())
        {
            if let Some(known) = etag_caches.get_not_found().await {
                let etag =
                    ResponseEncoding::select(known.is_gzip, accepted).etag(&known.opaque_tag);
                if cache_policy.if_none_match_matches(&etag) {
                    return Some(self.not_modified(etag));
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
                let (body, encoding, opaque_tag) =
                    match build_response_body(&file_content, accepted) {
                        Ok(result) => result,
                        Err(err) => return Some(Err(content_preparation_error(err))),
                    };

                let etag = encoding.etag(&opaque_tag);

                if let Some(etag_caches) = self.etag_caches.as_ref() {
                    etag_caches
                        .set_not_found(KnownEtag {
                            opaque_tag,
                            is_gzip: file_content.is_gzip,
                        })
                        .await;

                    // Nothing was known about the file yet - the first request after a start.
                    if cache_policy.if_none_match_matches(&etag) {
                        return Some(self.not_modified(etag));
                    }
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
                    let (body, encoding, opaque_tag) = build_response_body(&file_content, accepted)
                        .map_err(content_preparation_error)?;

                    let etag = encoding.etag(&opaque_tag);

                    etag_cache
                        .set(
                            http_path,
                            KnownEtag {
                                opaque_tag,
                                is_gzip: file_content.is_gzip,
                            },
                        )
                        .await;

                    // Nothing was known about the path yet - the first request after a start.
                    if cache_policy.if_none_match_matches(&etag) {
                        return self.not_modified(etag);
                    }

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

    /// `304 Not Modified` with the headers the `200` to the same request carries - the ones of
    /// `add_header`, `ETag`, `Cache-Control` and `Vary` (RFC 9110 §15.4.5). Not the ones which
    /// describe a body: no `Content-Type`, no `Content-Encoding`.
    ///
    /// `etag` is the one of the variant this request gets - see [`ResponseEncoding::etag`].
    fn not_modified(&self, etag: String) -> Result<HttpOkResult, HttpFailResult> {
        CacheHeaders::with_etag(etag)
            .apply(HttpOutput::as_not_modified().add_headers_opt(self.get_headers()))
            .add_header("Vary", "Accept-Encoding")
            .into_ok_result(false)
    }
}

#[async_trait::async_trait]
impl HttpServerMiddleware for StaticFilesMiddleware {
    async fn handle_request(
        &self,
        ctx: &mut HttpContext,
    ) -> Option<Result<HttpOkResult, HttpFailResult>> {
        let path = &ctx.request.http_path;

        // Before anything else: glued to a folder, a path with a `..` segment is a file out of
        // it - `/../../etc/hosts`. It is a file of no folder served here.
        if has_parent_segment(path.as_str()) {
            return Some(Err(HttpFailResult::as_forbidden(Some("Forbidden"))));
        }

        let headers = ctx.request.data.headers();

        let cache_policy = if self.no_cache.marked_as_no_cache(path) {
            CachePolicy::NoCache
        } else if headers.contains_key(IF_NONE_MATCH) {
            CachePolicy::IfNoneMatch(IfNoneMatch(headers))
        } else {
            CachePolicy::Regular
        };

        let accepted = ctx
            .request
            .get_headers()
            .try_get_case_insensitive("accept-encoding")
            .and_then(|h| h.as_str().ok())
            .map(parse_accept_encoding)
            .unwrap_or_default();

        // A path served before: the 304 is answered from what is known about it, without
        // touching the file.
        if let (CachePolicy::IfNoneMatch(_), Some(etag_caches)) =
            (cache_policy, self.etag_caches.as_ref())
        {
            if let Some(known) = etag_caches.get(path).await {
                let etag =
                    ResponseEncoding::select(known.is_gzip, accepted).etag(&known.opaque_tag);
                if cache_policy.if_none_match_matches(&etag) {
                    return Some(self.not_modified(etag));
                }
            }
        }

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

/// Returns (body, encoding, opaque tag). Used when ETag is required — always
/// materialises raw bytes if needed to compute the checksum. The tag is the hash
/// of the raw content whatever the encoding; [`ResponseEncoding::etag`] turns it
/// into the `ETag` of the variant.
fn build_response_body(
    cached: &CachedContent,
    accepted: AcceptedEncodings,
) -> std::io::Result<(Vec<u8>, ResponseEncoding, String)> {
    let encoding = ResponseEncoding::select(cached.is_gzip, accepted);

    if encoding == ResponseEncoding::Gzip {
        let opaque_tag = match &cached.etag {
            Some(e) => e.clone(),
            None => calc_etag(&gzip_decompress(&cached.data)?),
        };
        return Ok((cached.data.clone(), encoding, opaque_tag));
    }

    let raw = if cached.is_gzip {
        gzip_decompress(&cached.data)?
    } else {
        cached.data.clone()
    };

    let opaque_tag = match &cached.etag {
        Some(e) => e.clone(),
        None => calc_etag(&raw),
    };

    if encoding == ResponseEncoding::Deflate {
        let deflated = deflate_compress(&raw)?;
        return Ok((deflated, encoding, opaque_tag));
    }

    Ok((raw, encoding, opaque_tag))
}

/// Returns (body, encoding). Used when ETag is NOT required — avoids the
/// extra decompression needed for checksum calculation.
fn body_without_etag(
    cached: &CachedContent,
    accepted: AcceptedEncodings,
) -> std::io::Result<(Vec<u8>, ResponseEncoding)> {
    match ResponseEncoding::select(cached.is_gzip, accepted) {
        ResponseEncoding::Identity if !cached.is_gzip => {
            Ok((cached.data.clone(), ResponseEncoding::Identity))
        }
        ResponseEncoding::Identity => {
            Ok((gzip_decompress(&cached.data)?, ResponseEncoding::Identity))
        }
        ResponseEncoding::Gzip => Ok((cached.data.clone(), ResponseEncoding::Gzip)),
        ResponseEncoding::Deflate => {
            let raw = gzip_decompress(&cached.data)?;
            Ok((deflate_compress(&raw)?, ResponseEncoding::Deflate))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_accept_encoding, AcceptedEncodings, ResponseEncoding};

    #[test]
    fn a_file_stored_raw_always_goes_raw() {
        for (gzip, deflate) in [(false, false), (true, false), (false, true), (true, true)] {
            let accepted = AcceptedEncodings { gzip, deflate };
            assert!(ResponseEncoding::select(false, accepted) == ResponseEncoding::Identity);
        }
    }

    #[test]
    fn a_gzipped_file_goes_as_the_client_takes_it() {
        let select =
            |gzip, deflate| ResponseEncoding::select(true, AcceptedEncodings { gzip, deflate });

        assert!(select(true, false) == ResponseEncoding::Gzip);
        assert!(select(true, true) == ResponseEncoding::Gzip);
        assert!(select(false, true) == ResponseEncoding::Deflate);
        assert!(select(false, false) == ResponseEncoding::Identity);
    }

    #[test]
    fn only_the_raw_variant_gets_a_strong_etag() {
        assert_eq!(ResponseEncoding::Identity.etag("abc"), "\"abc\"");
        assert_eq!(ResponseEncoding::Gzip.etag("abc"), "W/\"abc\"");
        assert_eq!(ResponseEncoding::Deflate.etag("abc"), "W/\"abc\"");
    }

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
