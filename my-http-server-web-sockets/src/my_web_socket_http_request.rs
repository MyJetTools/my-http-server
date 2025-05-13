use hyper::{header::*, *};
use hyper_tungstenite::tungstenite::http::Extensions;

pub struct MyWebSocketHttpRequest {
    uri: Uri,
    headers: HeaderMap<HeaderValue>,
    version: Version,
    extensions: Extensions,
}

impl<'s> MyWebSocketHttpRequest {
    pub fn new(req: &Request<hyper::body::Incoming>) -> Self {
        Self {
            uri: req.uri().clone(),
            headers: req.headers().clone(),
            version: req.version(),
            extensions: req.extensions().clone(),
        }
    }

    pub fn get_uri(&self) -> &Uri {
        &self.uri
    }

    pub fn get_headers(&self) -> &HeaderMap<HeaderValue> {
        &self.headers
    }

    pub fn get_http_version(&self) -> Version {
        self.version
    }

    pub fn get_extensions(&self) -> &Extensions {
        &self.extensions
    }
}
