use std::collections::HashMap;

use tokio::sync::Mutex;

pub struct FilesAccess {
    cache: Mutex<HashMap<String, Vec<u8>>>,
    enable_caching: bool,
}

impl FilesAccess {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            enable_caching: false,
        }
    }

    pub fn enable_caching(&mut self) {
        self.enable_caching = true;
    }

    pub async fn get(&self, filename: &str) -> std::io::Result<Vec<u8>> {
        if self.enable_caching {
            let cache = self.cache.lock().await;
            if let Some(content) = cache.get(filename) {
                return Ok(content.clone());
            }
        }

        let result = tokio::fs::read(filename).await?;

        if self.enable_caching {
            let mut cache = self.cache.lock().await;
            cache.insert(filename.to_string(), result.clone());
        }

        return Ok(result);
    }
}
