use std::collections::HashMap;

use crate::{HttpRequest, HttpRequestBody, QueryString};

pub struct HttpRequestBucket {
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<HttpRequestBody>,
    pub query_string: Option<QueryString>,
}

impl HttpRequestBucket {
    pub fn new() -> Self {
        HttpRequestBucket {
            headers: None,
            body: None,
            query_string: None,
        }
    }

    pub fn populate_header(&mut self, key: &str, http_request: &HttpRequest) {
        if let Some(value) = http_request.get_headers().get(key) {
            if self.headers.is_none() {
                self.headers = Some(HashMap::new());
            }

            self.headers
                .as_mut()
                .unwrap()
                .insert(key.to_string(), value.to_str().unwrap().to_string());
        }
    }

    pub fn populate_query(&mut self, query_string: QueryString) {
        self.query_string = Some(query_string);
    }

    pub fn populate_body(&mut self, body: Option<HttpRequestBody>) {
        self.body = body;
    }

    pub fn get_query_string(&self) -> &QueryString {
        if let Some(ref query_string) = self.query_string {
            query_string
        } else {
            panic!("HttpRequestBucket::get_query_string() called when query_string is None");
        }
    }

    pub fn get_header_value(&self, key: &str) -> Option<&str> {
        if let Some(ref headers) = self.headers {
            let result = headers.get(key)?;
            Some(result)
        } else {
            None
        }
    }

    pub fn get_body(&mut self) -> Option<HttpRequestBody> {
        let mut result = None;
        std::mem::swap(&mut self.body, &mut result);
        result
    }
}
