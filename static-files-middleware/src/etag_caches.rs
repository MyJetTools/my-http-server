use my_http_server_core::HttpPath;

use rust_extensions::str_utils::StrUtils;
use tokio::sync::RwLock;

/// What a `304 Not Modified` for a path is built from without touching the file: the hash of its
/// content, and how the content is stored - which decides the variant a client gets, and so the
/// `ETag` it is answered with.
#[derive(Clone)]
pub struct KnownEtag {
    /// The opaque tag: the hash of the raw content, without quotes.
    pub opaque_tag: String,
    /// The content is stored gzipped (see `CachedContent::is_gzip`).
    pub is_gzip: bool,
}

struct ETagCache {
    pub path: String,
    pub etag: KnownEtag,
}

#[derive(Default)]
pub struct EtagCachesInner {
    paths: Vec<ETagCache>,
    not_found: Option<KnownEtag>,
}

#[derive(Default)]
pub struct EtagCaches {
    inner: RwLock<EtagCachesInner>,
}

impl EtagCaches {
    /// The ETag the content of `path` was last served with.
    pub async fn get(&self, path: &HttpPath) -> Option<KnownEtag> {
        let read_access = self.inner.read().await;

        read_access
            .paths
            .iter()
            .find(|itm| itm.path.eq_case_insensitive(path.as_str()))
            .map(|itm| itm.etag.clone())
    }

    pub async fn set(&self, path: &HttpPath, etag: KnownEtag) {
        let mut write_access = self.inner.write().await;

        for itm in write_access.paths.iter_mut() {
            if itm.path.eq_case_insensitive(path.as_str()) {
                itm.etag = etag;
                return;
            }
        }

        write_access.paths.push(ETagCache {
            path: path.to_string(),
            etag,
        });
    }

    pub async fn set_not_found(&self, etag: KnownEtag) {
        let mut write_access = self.inner.write().await;
        write_access.not_found = Some(etag);
    }

    /// The ETag the not-found file was last served with.
    pub async fn get_not_found(&self) -> Option<KnownEtag> {
        let read_access = self.inner.read().await;
        read_access.not_found.clone()
    }
}

/// The opaque tag of a content: SHA-256 of it in base64, the `=` padding stripped. Base64 has no
/// quote, space or comma in it, so the result goes into an `ETag` as it is - in quotes.
pub fn calc_etag(content: &[u8]) -> String {
    use base64::Engine;
    use sha2::Digest;
    use sha2::Sha256;
    let mut sha_256 = Sha256::new();

    sha_256.update(content);
    let result = sha_256.finalize();
    let mut result = base64::engine::general_purpose::STANDARD.encode(result);

    if result.ends_with('=') {
        result.pop();
    }

    result
}
