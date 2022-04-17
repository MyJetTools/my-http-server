use crate::HttpRequest;

pub struct HttpContext<'s> {
    pub request: HttpRequest<'s>,
}

impl<'s> HttpContext<'s> {
    pub fn new(request: HttpRequest<'s>) -> Self {
        Self { request }
    }
}
