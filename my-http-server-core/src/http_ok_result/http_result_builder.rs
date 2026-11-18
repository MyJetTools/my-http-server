use std::io::Write;

use flate2::write::GzEncoder;
use rust_extensions::StrOrString;

use crate::{
    aes128gcm, cookies::*, AddHttpHeaders, BodyEncryptionKey, CompressionLevel, HttpFailResult,
    HttpOutput, HttpResponseHeaders, WebContentType,
};

use super::HttpOkResult;

const EMPTY_STATUS_CODE: u16 = 204;

const COMPRESSION_EFFICIENCY_NUM: usize = 80;
const COMPRESSION_EFFICIENCY_DEN: usize = 100;

pub struct HttpResultBuilder {
    pub(crate) output: HttpOutput,
    /*
    pub(crate) status_code: u16,
    pub(crate) headers: Option<HashMap<String, String>>,
    pub(crate) content_type: Option<WebContentType>,
    pub(crate) cookies: Option<CookieJar>,
    pub(crate) content: Vec<u8>,
     */
}

fn gzip(raw: &[u8], level: CompressionLevel) -> std::io::Result<Vec<u8>> {
    let mut encoder = GzEncoder::new(Vec::with_capacity(raw.len()), level.to_gzip());
    encoder.write_all(raw)?;
    encoder.finish()
}

fn has_header(headers: &HttpResponseHeaders, name: &str) -> bool {
    headers
        .headers
        .iter()
        .any(|(key, _)| key.as_str().eq_ignore_ascii_case(name))
}

/// Puts `coding` into `Content-Encoding` after the codings already there, so the one header lists
/// them in the order they were applied.
fn add_content_coding(headers: &mut HttpResponseHeaders, coding: &str) {
    let existing = headers
        .headers
        .iter_mut()
        .find(|(key, _)| key.as_str().eq_ignore_ascii_case("content-encoding"));

    match existing {
        Some((_, value)) => {
            value.push_str(", ");
            value.push_str(coding);
        }
        None => headers.add_header("Content-Encoding".into(), coding.to_string()),
    }
}

impl HttpResultBuilder {
    pub fn new() -> Self {
        Self {
            output: HttpOutput::Empty, /*
                                       status_code: 204,
                                       headers: None,
                                       content_type: None,
                                       cookies: Default::default(),
                                       content: Default::default(),
                                        */
        }
    }

    pub fn set_content_type(mut self, new_content_type: WebContentType) -> Self {
        match &mut self.output {
            HttpOutput::Empty => {
                self.output = HttpOutput::Content {
                    status_code: EMPTY_STATUS_CODE,
                    headers: HttpResponseHeaders::new(Some(new_content_type)),
                    content: Default::default(),
                }
            }
            HttpOutput::Content { headers, .. } => {
                headers.set_content_type(new_content_type);
            }
            HttpOutput::Redirect { .. } => {
                panic!("Can not set content type at redirect output");
            }
            HttpOutput::File { headers, .. } => {
                headers.set_content_type(new_content_type);
            }
            HttpOutput::Raw(_) => {
                panic!("Can not set content type to raw response");
            }
        }

        self
    }

    pub fn set_content_type_opt(self, content_type: Option<WebContentType>) -> Self {
        if let Some(content_type) = content_type {
            return self.set_content_type(content_type);
        }
        self
    }

    pub fn add_header(
        mut self,
        key: impl Into<StrOrString<'static>>,
        value: impl Into<String>,
    ) -> Self {
        match &mut self.output {
            HttpOutput::Empty => {
                self.output = HttpOutput::Content {
                    status_code: EMPTY_STATUS_CODE,
                    headers: HttpResponseHeaders::new_with_header(key.into(), value.into()),
                    content: Default::default(),
                }
            }
            HttpOutput::Content { headers, .. } => {
                headers.add_header(key.into(), value.into());
            }
            HttpOutput::Redirect { headers, .. } => {
                headers.add_header(key.into(), value.into());
            }
            HttpOutput::File { headers, .. } => {
                headers.add_header(key.into(), value.into());
            }
            HttpOutput::Raw(_) => {
                panic!("Can not set header to raw output")
            }
        }

        self
    }

    pub fn add_header_if_some(
        self,
        key: impl Into<StrOrString<'static>>,
        value: Option<impl Into<String>>,
    ) -> Self {
        if let Some(value) = value {
            return self.add_header(key, value);
        }

        self
    }

    pub fn add_headers(
        mut self,
        headers: impl Iterator<Item = (impl Into<StrOrString<'static>>, impl Into<String>)>,
    ) -> Self {
        for header in headers {
            self = self.add_header(header.0, header.1);
        }

        self
    }

    pub fn add_headers_opt(
        mut self,
        headers: Option<impl Iterator<Item = (impl Into<StrOrString<'static>>, impl Into<String>)>>,
    ) -> Self {
        if let Some(headers) = headers {
            for header in headers {
                self = self.add_header(header.0, header.1);
            }
        }

        self
    }

    pub fn set_cookie(mut self, cookie: impl Into<Cookie>) -> Self {
        match &mut self.output {
            HttpOutput::Empty => {
                self.output = HttpOutput::Content {
                    status_code: EMPTY_STATUS_CODE,
                    headers: HttpResponseHeaders::new_with_cookie(cookie.into()),
                    content: Default::default(),
                }
            }
            HttpOutput::Content { headers, .. } => {
                headers.set_cookie(cookie.into());
            }
            HttpOutput::Redirect { headers, .. } => {
                headers.set_cookie(cookie.into());
            }
            HttpOutput::File { headers, .. } => {
                headers.set_cookie(cookie.into());
            }
            HttpOutput::Raw(_) => {
                panic!("Can not set cookie to raw response");
            }
        }

        self
    }

    pub fn set_cookies(mut self, cookies: impl IntoIterator<Item = impl Into<Cookie>>) -> Self {
        match &mut self.output {
            HttpOutput::Empty => {
                self.output = HttpOutput::Content {
                    status_code: EMPTY_STATUS_CODE,
                    headers: HttpResponseHeaders::new_with_cookies(
                        CookieJar::new().set_cookies(cookies),
                    ),
                    content: Default::default(),
                }
            }
            HttpOutput::Content { headers, .. } => {
                headers.set_cookies(cookies);
            }
            HttpOutput::Redirect { headers, .. } => {
                headers.set_cookies(cookies);
            }
            HttpOutput::File { headers, .. } => {
                headers.set_cookies(cookies);
            }
            HttpOutput::Raw(_) => {
                panic!("Can not set cookies to raw response");
            }
        }

        self
    }

    pub fn set_content_as_text(mut self, content: impl Into<String>) -> Self {
        const CONTENT_TYPE_TO_SET: WebContentType = WebContentType::Text;
        let text_content = content.into();

        match &mut self.output {
            HttpOutput::Empty => {
                self.output = HttpOutput::Content {
                    status_code: 200,
                    headers: HttpResponseHeaders::new(CONTENT_TYPE_TO_SET.into()),
                    content: text_content.into_bytes(),
                }
            }
            HttpOutput::Content {
                headers,
                content,
                status_code,
            } => {
                if *status_code == EMPTY_STATUS_CODE {
                    *status_code = 200;
                }
                headers.set_content_type(CONTENT_TYPE_TO_SET);
                *content = text_content.into_bytes();
            }
            HttpOutput::Redirect { .. } => {
                panic!("Can not set content to redirect response");
            }
            HttpOutput::File { content, .. } => {
                *content = text_content.into_bytes();
            }
            HttpOutput::Raw(_) => {
                panic!("Can not set content to raw response");
            }
        }

        self
    }

    pub fn set_status_code(mut self, status_code_to_set: u16) -> Self {
        match &mut self.output {
            HttpOutput::Empty => {
                self.output = HttpOutput::Content {
                    status_code: status_code_to_set,
                    headers: Default::default(),
                    content: Default::default(),
                }
            }
            HttpOutput::Content { status_code, .. } => {
                *status_code = status_code_to_set;
            }
            HttpOutput::Redirect { .. } => {
                panic!("Can not set status_code to redirect response");
            }
            HttpOutput::File { .. } => {
                panic!("Can not set status_code to File output response");
            }
            HttpOutput::Raw(_) => {
                panic!("Can not set content to raw response");
            }
        }
        self
    }

    pub fn set_content(mut self, content_to_set: Vec<u8>) -> Self {
        match &mut self.output {
            HttpOutput::Empty => {
                self.output = HttpOutput::Content {
                    status_code: 200,
                    headers: Default::default(),
                    content: content_to_set,
                }
            }
            HttpOutput::Content {
                content,
                status_code,
                ..
            } => {
                if *status_code == EMPTY_STATUS_CODE {
                    *status_code = 200;
                }
                *content = content_to_set;
            }
            HttpOutput::Redirect { .. } => {
                panic!("Can not set status_code to redirect response");
            }
            HttpOutput::File { content, .. } => {
                *content = content_to_set;
            }
            HttpOutput::Raw(_) => {
                panic!("Can not set content to raw response");
            }
        }
        self
    }

    pub fn with_compression(mut self, threshold: usize, level: CompressionLevel) -> Self {
        let compressed = {
            let body = match &self.output {
                HttpOutput::Content { content, .. } => content,
                HttpOutput::File { content, .. } => content,
                HttpOutput::Empty | HttpOutput::Redirect { .. } => return self,
                HttpOutput::Raw(_) => panic!("Can not compress raw response"),
            };

            if body.len() <= threshold {
                return self;
            }

            let Ok(c) = gzip(body.as_slice(), level) else {
                return self;
            };

            if c.len() * COMPRESSION_EFFICIENCY_DEN > body.len() * COMPRESSION_EFFICIENCY_NUM {
                return self;
            }

            c
        };

        match &mut self.output {
            HttpOutput::Content { content, .. } => *content = compressed,
            HttpOutput::File { content, .. } => *content = compressed,
            _ => unreachable!(),
        }

        self.add_header("Content-Encoding", "gzip")
    }

    /// Encrypts the body with `key`: `Content-Encoding: aes128gcm` (RFC 8188), a fresh random salt
    /// every time, records of 4096 bytes, no key id in the header. This is how my-reverse-proxy's
    /// `docs/encrypted-service-data.md` has a service answer. `Content-Type` stays the type of the
    /// data as it was before encryption. The response also gets `Cache-Control: no-store`, unless
    /// the action set a `Cache-Control` of its own.
    ///
    /// What is encrypted is the body as it is at the call, so call this after the content is set.
    /// A body that [`with_compression`](Self::with_compression) has already gzipped is encrypted as
    /// it is, and the header becomes `Content-Encoding: gzip, aes128gcm`: the codings in the order
    /// they were applied (RFC 9110 §8.4). Compressing afterwards gains nothing, since encrypted
    /// bytes do not compress, so `with_compression` then leaves the body alone.
    ///
    /// - `Content`, `File` with a body → the body is encrypted, whatever the status code.
    /// - An empty body, `Empty`, `Redirect` → nothing to encrypt: a no-op. A `204` must not get a
    ///   body of encryption overhead.
    /// - `Raw` → panics (the framework can't introspect a pre-built `MyHttpResponse`).
    ///
    /// Panics only if the operating system has no random bytes for the salt. A body is never sent
    /// unencrypted instead.
    pub fn encode(mut self, key: &BodyEncryptionKey) -> Self {
        let headers = match &mut self.output {
            HttpOutput::Content {
                content, headers, ..
            }
            | HttpOutput::File {
                content, headers, ..
            } => {
                if content.is_empty() {
                    return self;
                }

                *content = aes128gcm::encrypt(key.as_ikm(), content);
                headers
            }
            HttpOutput::Empty | HttpOutput::Redirect { .. } => return self,
            HttpOutput::Raw(_) => panic!("Can not encode raw response"),
        };

        add_content_coding(headers, aes128gcm::CONTENT_ENCODING);

        if !has_header(headers, "cache-control") {
            headers.add_header("Cache-Control".into(), "no-store".to_string());
        }

        self
    }

    pub fn into_ok_result(self, write_telemetry: bool) -> Result<HttpOkResult, HttpFailResult> {
        let output = self.build();
        Ok(HttpOkResult {
            write_telemetry,
            #[cfg(feature = "with-telemetry")]
            add_telemetry_tags: my_telemetry::TelemetryEventTagsBuilder::new(),
            output,
        })
    }

    pub fn build(self) -> HttpOutput {
        self.output
    }

    pub fn into_err<TResult>(
        self,
        write_log: bool,
        write_telemetry: bool,
    ) -> Result<TResult, HttpFailResult> {
        let output = self.build();
        Err(HttpFailResult::new(output, write_log, write_telemetry))
    }

    pub fn into_http_fail_result(self, write_log: bool, write_telemetry: bool) -> HttpFailResult {
        let output = self.build();
        HttpFailResult::new(output, write_log, write_telemetry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn find_header<'a>(headers: &'a HttpResponseHeaders, name: &str) -> Option<&'a str> {
        headers
            .headers
            .iter()
            .find(|(k, _)| k.as_str().eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    #[test]
    fn empty_is_noop() {
        let built = HttpResultBuilder::new()
            .with_compression(1024, CompressionLevel::Balanced)
            .build();
        match built {
            HttpOutput::Empty => {}
            other => panic!("expected Empty, got {:?}", other),
        }
    }

    #[test]
    fn redirect_is_noop() {
        let built = HttpOutput::as_redirect("/elsewhere", false)
            .with_compression(1, CompressionLevel::Balanced)
            .build();
        match built {
            HttpOutput::Redirect { headers, .. } => {
                assert!(find_header(&headers, "Content-Encoding").is_none());
            }
            other => panic!("expected Redirect, got {:?}", other),
        }
    }

    #[test]
    fn body_below_threshold_is_unchanged() {
        let raw = vec![b'a'; 5 * 1024];
        let built = HttpResultBuilder {
            output: HttpOutput::as_content(raw.clone(), None),
        }
        .with_compression(10 * 1024, CompressionLevel::Balanced)
            .build();

        match built {
            HttpOutput::Content {
                content, headers, ..
            } => {
                assert_eq!(content, raw);
                assert!(find_header(&headers, "Content-Encoding").is_none());
            }
            other => panic!("expected Content, got {:?}", other),
        }
    }

    #[test]
    fn highly_compressible_body_is_compressed() {
        let raw = vec![b'a'; 50 * 1024];
        let built = HttpResultBuilder {
            output: HttpOutput::as_content(raw.clone(), None),
        }
        .with_compression(10 * 1024, CompressionLevel::Balanced)
            .build();

        match built {
            HttpOutput::Content {
                content, headers, ..
            } => {
                assert!(content.len() * 100 <= raw.len() * 80);
                assert_eq!(find_header(&headers, "Content-Encoding"), Some("gzip"));
                let mut decoder = flate2::read::GzDecoder::new(content.as_slice());
                let mut roundtrip = Vec::new();
                std::io::Read::read_to_end(&mut decoder, &mut roundtrip).unwrap();
                assert_eq!(roundtrip, raw);
            }
            other => panic!("expected Content, got {:?}", other),
        }
    }

    #[test]
    fn incompressible_body_is_left_alone() {
        let mut raw = Vec::with_capacity(50 * 1024);
        let mut seed: u32 = 0x9E37_79B1;
        for _ in 0..50 * 1024 {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            raw.push((seed >> 24) as u8);
        }

        let built = HttpResultBuilder {
            output: HttpOutput::as_content(raw.clone(), None),
        }
        .with_compression(10 * 1024, CompressionLevel::Balanced)
            .build();

        match built {
            HttpOutput::Content {
                content, headers, ..
            } => {
                assert_eq!(content, raw);
                assert!(find_header(&headers, "Content-Encoding").is_none());
            }
            other => panic!("expected Content, got {:?}", other),
        }
    }

    /// Text that compresses, but not trivially - so the level makes a difference.
    fn words(len: usize) -> Vec<u8> {
        const WORDS: [&str; 8] = [
            "alpha ", "bravo ", "charlie ", "delta ", "echo ", "foxtrot ", "golf ", "hotel ",
        ];

        let mut raw = Vec::with_capacity(len);
        let mut seed: u32 = 0x9E37_79B1;
        while raw.len() < len {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            raw.extend_from_slice(WORDS[(seed >> 29) as usize].as_bytes());
        }
        raw
    }

    fn compressed_content(raw: &[u8], level: CompressionLevel) -> (Vec<u8>, Option<String>) {
        let built = HttpResultBuilder {
            output: HttpOutput::as_content(raw.to_vec(), None),
        }
        .with_compression(0, level)
        .build();

        match built {
            HttpOutput::Content {
                content, headers, ..
            } => {
                let encoding = find_header(&headers, "Content-Encoding").map(|v| v.to_string());
                (content, encoding)
            }
            other => panic!("expected Content, got {:?}", other),
        }
    }

    #[test]
    fn the_level_is_applied() {
        let raw = words(50 * 1024);

        let (fast, fast_encoding) = compressed_content(&raw, CompressionLevel::Fast);
        let (best, best_encoding) = compressed_content(&raw, CompressionLevel::Best);

        assert_eq!(fast_encoding.as_deref(), Some("gzip"));
        assert_eq!(best_encoding.as_deref(), Some("gzip"));
        assert!(best.len() < fast.len(), "best {} fast {}", best.len(), fast.len());

        for content in [fast, best] {
            let mut roundtrip = Vec::new();
            std::io::Read::read_to_end(
                &mut flate2::read::GzDecoder::new(content.as_slice()),
                &mut roundtrip,
            )
            .unwrap();
            assert_eq!(roundtrip, raw);
        }
    }

    /// Level 0 only frames the bytes, so the result is larger than the body and never passes the
    /// efficiency check.
    #[test]
    fn a_stored_level_is_never_worth_it() {
        let raw = words(50 * 1024);

        let (content, encoding) = compressed_content(&raw, CompressionLevel::Level(0));

        assert_eq!(content, raw);
        assert_eq!(encoding, None);
    }

    fn key(byte: u8) -> BodyEncryptionKey {
        BodyEncryptionKey::new("test", [byte; BodyEncryptionKey::LEN])
    }

    fn content_and_headers(output: HttpOutput) -> (u16, Vec<u8>, HttpResponseHeaders) {
        match output {
            HttpOutput::Content {
                status_code,
                content,
                headers,
            } => (status_code, content, headers),
            other => panic!("expected Content, got {:?}", other),
        }
    }

    #[test]
    fn an_encoded_body_opens_with_the_key_and_keeps_its_content_type() {
        let built = HttpOutput::as_text("ip_white_lists: {}")
            .encode(&key(7))
            .build();

        let (status_code, content, headers) = content_and_headers(built);

        assert_eq!(status_code, 200);
        assert_ne!(content, b"ip_white_lists: {}");
        assert_eq!(
            aes128gcm::decrypt(&[7u8; 32], &content).unwrap(),
            b"ip_white_lists: {}"
        );
        assert!(aes128gcm::decrypt(&[8u8; 32], &content).is_err());

        assert_eq!(find_header(&headers, "Content-Encoding"), Some("aes128gcm"));
        assert_eq!(find_header(&headers, "Cache-Control"), Some("no-store"));
        assert_eq!(
            headers.content_type.as_ref().map(|value| value.as_str()),
            Some(WebContentType::Text.as_str())
        );
    }

    /// The status code is the action's: an error can be answered encrypted too.
    #[test]
    fn any_status_code_is_encoded() {
        let built = HttpResultBuilder::new()
            .set_status_code(409)
            .set_content(b"conflict".to_vec())
            .encode(&key(1))
            .build();

        let (status_code, content, _) = content_and_headers(built);

        assert_eq!(status_code, 409);
        assert_eq!(
            aes128gcm::decrypt(&[1u8; 32], &content).unwrap(),
            b"conflict"
        );
    }

    #[test]
    fn a_cache_control_of_the_action_is_kept() {
        let built = HttpOutput::as_text("data")
            .add_header("Cache-Control", "private, max-age=10")
            .encode(&key(1))
            .build();

        let (_, _, headers) = content_and_headers(built);

        let cache_control: Vec<_> = headers
            .headers
            .iter()
            .filter(|(name, _)| name.as_str().eq_ignore_ascii_case("cache-control"))
            .map(|(_, value)| value.as_str())
            .collect();

        assert_eq!(cache_control, vec!["private, max-age=10"]);
    }

    /// Compressed first, then encrypted: one header listing both, in that order, and the bytes
    /// under the encryption are the gzip ones.
    #[test]
    fn a_compressed_body_is_encrypted_as_it_is() {
        let raw = vec![b'a'; 50 * 1024];

        let built = HttpResultBuilder {
            output: HttpOutput::as_content(raw.clone(), None),
        }
        .with_compression(1024, CompressionLevel::Balanced)
        .encode(&key(3))
        .build();

        let (_, content, headers) = content_and_headers(built);

        let content_encoding: Vec<_> = headers
            .headers
            .iter()
            .filter(|(name, _)| name.as_str().eq_ignore_ascii_case("content-encoding"))
            .map(|(_, value)| value.as_str())
            .collect();
        assert_eq!(content_encoding, vec!["gzip, aes128gcm"]);

        let gzipped = aes128gcm::decrypt(&[3u8; 32], &content).unwrap();
        let mut plain = Vec::new();
        std::io::Read::read_to_end(
            &mut flate2::read::GzDecoder::new(gzipped.as_slice()),
            &mut plain,
        )
        .unwrap();
        assert_eq!(plain, raw);
    }

    /// Encrypted bytes do not compress, so compressing after `encode` leaves them alone.
    #[test]
    fn compressing_after_encoding_changes_nothing() {
        let raw = vec![b'a'; 50 * 1024];

        let built = HttpResultBuilder {
            output: HttpOutput::as_content(raw.clone(), None),
        }
        .encode(&key(3))
        .with_compression(1024, CompressionLevel::Balanced)
        .build();

        let (_, content, headers) = content_and_headers(built);

        assert_eq!(find_header(&headers, "Content-Encoding"), Some("aes128gcm"));
        assert_eq!(aes128gcm::decrypt(&[3u8; 32], &content).unwrap(), raw);
    }

    #[test]
    fn nothing_to_encode_is_a_noop() {
        match HttpResultBuilder::new().encode(&key(1)).build() {
            HttpOutput::Empty => {}
            other => panic!("expected Empty, got {:?}", other),
        }

        match HttpOutput::as_redirect("/elsewhere", false)
            .encode(&key(1))
            .build()
        {
            HttpOutput::Redirect { headers, .. } => {
                assert!(find_header(&headers, "Content-Encoding").is_none());
            }
            other => panic!("expected Redirect, got {:?}", other),
        }

        // A 204 with a header set: a Content with no body, which stays without one.
        let built = HttpResultBuilder::new()
            .add_header("X-Test", "1")
            .encode(&key(1))
            .build();
        let (status_code, content, headers) = content_and_headers(built);
        assert_eq!(status_code, 204);
        assert!(content.is_empty());
        assert!(find_header(&headers, "Content-Encoding").is_none());
    }

    #[test]
    fn a_file_is_encoded() {
        let builder = HttpResultBuilder {
            output: HttpOutput::File {
                file_name: "settings.yaml".to_string(),
                content: b"hosts: {}".to_vec(),
                headers: Default::default(),
            },
        };

        match builder.encode(&key(5)).build() {
            HttpOutput::File {
                content, headers, ..
            } => {
                assert_eq!(
                    aes128gcm::decrypt(&[5u8; 32], &content).unwrap(),
                    b"hosts: {}"
                );
                assert_eq!(find_header(&headers, "Content-Encoding"), Some("aes128gcm"));
            }
            other => panic!("expected File, got {:?}", other),
        }
    }
}

impl AddHttpHeaders for HttpResultBuilder {
    fn add_header(&mut self, key: impl Into<String>, value: impl Into<String>) {
        match &mut self.output {
            HttpOutput::Empty => {
                self.output = HttpOutput::Content {
                    status_code: EMPTY_STATUS_CODE,
                    headers: HttpResponseHeaders::new_with_header(key.into().into(), value.into()),
                    content: Default::default(),
                }
            }
            HttpOutput::Content { headers, .. } => {
                headers.add_header(key.into().into(), value.into());
            }
            HttpOutput::Redirect { headers, .. } => {
                headers.add_header(key.into().into(), value.into());
            }
            HttpOutput::File { headers, .. } => {
                headers.add_header(key.into().into(), value.into());
            }
            HttpOutput::Raw(_) => {
                panic!("Can not set header to raw output")
            }
        }
    }
}
