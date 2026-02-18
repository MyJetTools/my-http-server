use my_http_server_core::HttpPath;

use rust_extensions::str_utils::StrUtils;
use tokio::sync::RwLock;

struct ETagCache {
    pub path: String,
    pub etag: String,
}

#[derive(Default)]
pub struct EtagCachesInner {
    paths: Vec<ETagCache>,
    not_found: Option<String>,
}

#[derive(Default)]
pub struct EtagCaches {
    inner: RwLock<EtagCachesInner>,
}

impl EtagCaches {
    pub async fn check_etag(&self, path: &HttpPath, etag: &str) -> bool {
        let read_access = self.inner.read().await;

        for itm in read_access.paths.iter() {
            if itm.path.eq_case_insensitive(path.as_str()) {
                if itm.etag == etag {
                    return true;
                }
            }
        }

        false
    }

    pub async fn set(&self, path: &HttpPath, etag: String) {
        println!("Setting Etag: {} for path '{}'", etag, path.as_str());

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

    pub async fn set_not_found(&self, etag: String) {
        println!("Setting Etag for not found path");
        let mut write_access = self.inner.write().await;
        write_access.not_found = Some(etag);
    }

    pub async fn is_not_found(&self, etag: &str) -> bool {
        let write_access = self.inner.read().await;
        if let Some(nf_etag) = write_access.not_found.as_ref() {
            return nf_etag.as_str() == etag;
        }

        false
    }
}

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
