use std::collections::HashMap;

use tokio::sync::Mutex;

use crate::{calc_etag, try_zstd};

#[derive(Clone)]
pub struct CachedContent {
    pub data: Vec<u8>,
    pub is_zstd: bool,
    pub etag: Option<String>,
}

pub struct FilesAccess {
    cache: Mutex<HashMap<String, CachedContent>>,
    enable_caching: bool,
    enable_etag: bool,
}

impl FilesAccess {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            enable_caching: false,
            enable_etag: false,
        }
    }

    pub fn enable_caching(&mut self) {
        self.enable_caching = true;
    }

    pub fn enable_etag(&mut self) {
        self.enable_etag = true;
    }

    pub async fn get(&self, filename: &str) -> std::io::Result<CachedContent> {
        if self.enable_caching {
            let cache = self.cache.lock().await;
            if let Some(content) = cache.get(filename) {
                return Ok(content.clone());
            }
        }

        let raw = tokio::fs::read(filename).await?;

        if !self.enable_caching {
            return Ok(CachedContent {
                data: raw,
                is_zstd: false,
                etag: None,
            });
        }

        let etag = if self.enable_etag {
            Some(calc_etag(&raw))
        } else {
            None
        };

        let entry = match try_zstd(&raw) {
            Some(compressed) => CachedContent {
                data: compressed,
                is_zstd: true,
                etag,
            },
            None => CachedContent {
                data: raw,
                is_zstd: false,
                etag,
            },
        };

        let mut cache = self.cache.lock().await;
        cache.insert(filename.to_string(), entry.clone());

        Ok(entry)
    }
}
