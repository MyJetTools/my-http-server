use http::Uri;

pub enum MyHyperHttpRequest {
    Incoming(hyper::Request<hyper::body::Incoming>),
    Full(hyper::Request<http_body_util::Full<bytes::Bytes>>),
}

impl MyHyperHttpRequest {
    pub fn uri(&self) -> &Uri {
        match self {
            MyHyperHttpRequest::Incoming(req) => &req.uri(),
            MyHyperHttpRequest::Full(req) => &req.uri(),
        }
    }
    pub fn headers(&self) -> &http::HeaderMap<http::header::HeaderValue> {
        match self {
            MyHyperHttpRequest::Incoming(req) => req.headers(),
            MyHyperHttpRequest::Full(req) => req.headers(),
        }
    }

    pub fn version(&self) -> http::Version {
        match self {
            MyHyperHttpRequest::Incoming(req) => req.version(),
            MyHyperHttpRequest::Full(req) => req.version(),
        }
    }

    pub fn extensions(&self) -> &http::Extensions {
        match self {
            MyHyperHttpRequest::Incoming(req) => req.extensions(),
            MyHyperHttpRequest::Full(req) => req.extensions(),
        }
    }
}
