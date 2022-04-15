use crate::HttpRequest;

pub struct HttpContext {
    request: Option<HttpRequest>,
}

impl HttpContext {
    pub fn new(request: HttpRequest) -> Self {
        Self {
            request: Some(request),
        }
    }

    pub fn get_request(&self) -> &HttpRequest {
        if let Some(result) = &self.request {
            result
        } else {
            panic!("HttpContext::get_request() called when request is None");
        }
    }

    pub fn get_request_ownership(&mut self) -> HttpRequest {
        let mut result = None;

        std::mem::swap(&mut self.request, &mut result);

        if result.is_none() {
            panic!("Can not read http request for the second time");
        }

        result.unwrap()
    }
}
