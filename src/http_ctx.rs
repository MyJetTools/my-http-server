use crate::HttpRequest;

pub struct HttpContext {
    pub request: HttpRequest,
}

impl HttpContext {
    pub fn new(request: HttpRequest) -> Self {
        Self { request }
    }
}
