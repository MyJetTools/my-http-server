use std::collections::HashMap;

use hyper::{http::HeaderValue, Body, Request};

pub struct CachedHeaders {
    headers: HashMap<String, HeaderValue>,
}

impl CachedHeaders {
    pub fn new(req: &mut Request<Body>) -> Self {
        let mut headers = HashMap::new();
        for (header_name, value) in req.headers_mut().drain() {
            if let Some(header_name) = header_name {
                headers.insert(header_name.as_str().to_lowercase(), value);
            }
        }
        Self { headers }
    }

    pub fn get(&self, name: &str) -> Option<&HeaderValue> {
        self.headers.get(name)
    }
}
