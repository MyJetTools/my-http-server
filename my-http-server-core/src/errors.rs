use crate::{HttpFailResult, HttpOutput, HttpResponseHeaders, WebContentType};

impl From<hyper::Error> for HttpFailResult {
    fn from(src: hyper::Error) -> Self {
        let output = HttpOutput::Content {
            status_code: 501,
            headers: HttpResponseHeaders::new(WebContentType::Text.into()),
            content: format!("{:?}", src).into_bytes(),
        };

        HttpFailResult::new(output, true, true)
    }
}

impl From<std::string::FromUtf8Error> for HttpFailResult {
    fn from(src: std::string::FromUtf8Error) -> Self {
        Self::as_fatal_error(format!("{}", src))
    }
}
