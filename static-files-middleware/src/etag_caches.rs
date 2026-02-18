use my_http_server_core::HttpPath;

use tokio::sync::RwLock;

struct ETagCache {
    pub path: String,
    pub etag: String,
}

#[derive(Default)]
pub struct EtagCaches {
    data: RwLock<Vec<ETagCache>>,
}

impl EtagCaches {
    pub async fn check_etag(&self, path: &HttpPath, etag: &str) -> bool {
        let read_access = self.data.read().await;

        for itm in read_access.iter() {
            if itm.path.eq_ignore_ascii_case(path.as_str()) {
                if itm.etag == etag {
                    return true;
                }
            }
        }

        false
    }

    pub async fn set(&self, path: &HttpPath, etag: String) {
        println!("Setting Etag: {} for path '{}'", etag, path.as_str());

        let mut write_access = self.data.write().await;

        for itm in write_access.iter_mut() {
            if itm.path.eq_ignore_ascii_case(path.as_str()) {
                itm.etag = etag;
                return;
            }
        }

        write_access.push(ETagCache {
            path: path.to_string(),
            etag,
        });
    }
}

pub fn calc_etag(content: &[u8]) -> String {
    use base64::Engine;
    use sha2::Digest;
    use sha2::Sha256;
    let mut sha_256 = Sha256::new();

    sha_256.update(content);
    let result = sha_256.finalize();
    base64::engine::general_purpose::STANDARD.encode(result)
}
