use http::{HeaderMap, HeaderValue};

use hyper::Uri;

use my_http_utils::http_input::HttpBodyAsStream;

use crate::{
    BodyContentType, BodyExpectations, HttpFailResult, HttpRequestBody, HttpRequestBodyContent,
    MyHyperHttpRequest,
};

pub struct RequestData {
    parts: hyper::http::request::Parts,
    body: Option<HttpRequestBody>,
}

impl RequestData {
    pub fn new(req: hyper::Request<hyper::body::Incoming>) -> Result<Self, HttpFailResult> {
        let (parts, incoming) = req.into_parts();

        let content_type = match parts.headers.get("content-type") {
            Some(header_value) => match header_value.to_str() {
                Ok(content_type) => content_type,
                Err(_) => {
                    return Err(HttpFailResult::as_validation_error(
                        "header content-type has response is not as string",
                    ))
                }
            },
            None => "",
        };

        let content_type = BodyContentType::from_content_type(content_type)?;

        let body = HttpRequestBody::Incoming {
            incoming: Some(incoming),
            content_type,
        };
        let result = Self {
            parts,
            body: Some(body),
        };

        Ok(result)
    }

    pub async fn get_body(&mut self) -> Result<&HttpRequestBodyContent, HttpFailResult> {
        let expectations = self.body_expectations();

        match self.body.as_mut() {
            Some(body) => body.get_http_request_body(expectations).await,
            None => {
                panic!("Body is removed and can not be accessed")
            }
        }
    }

    pub async fn receive_body(&mut self) -> Result<HttpRequestBodyContent, HttpFailResult> {
        let expectations = self.body_expectations();

        match self.body.take() {
            Some(body) => return body.into_http_request_body(expectations).await,
            None => {
                panic!("Body is taken by some middleware before")
            }
        }
    }

    /// Takes the body as a stream of chunks. Like [`receive_body`](Self::receive_body) it *takes*
    /// the body, so afterwards `get_body` / `take_my_hyper_http_request` behave exactly as they do
    /// after any other middleware consumed it.
    pub fn take_body_stream(
        &mut self,
        buffer: usize,
    ) -> Result<HttpBodyAsStream, HttpFailResult> {
        let expectations = self.body_expectations();

        match self.body.take() {
            Some(body) => Ok(body.into_body_stream(expectations, buffer)),
            None => Err(HttpFailResult::as_fatal_error(
                "Body is taken by some middleware before".to_string(),
            )),
        }
    }

    /// What this request promised about its body — used by both body-reading paths to tell a
    /// complete body from one the client abandoned.
    fn body_expectations(&self) -> BodyExpectations {
        BodyExpectations {
            version: self.parts.version,
            content_length: self.content_length(),
        }
    }

    /// `Content-Length` when the client sent a valid one. `None` for a chunked body — and for a
    /// malformed header, which is not worth failing the request over: the length is a hint here,
    /// the body is read frame by frame either way.
    fn content_length(&self) -> Option<u64> {
        self.parts
            .headers
            .get("content-length")?
            .to_str()
            .ok()?
            .trim()
            .parse::<u64>()
            .ok()
    }

    pub fn uri(&self) -> &Uri {
        &self.parts.uri
    }

    pub fn headers(&self) -> &HeaderMap<HeaderValue> {
        &self.parts.headers
    }

    pub fn extensions(&self) -> &http::Extensions {
        &self.parts.extensions
    }

    pub fn extensions_mut(&mut self) -> &mut http::Extensions {
        &mut self.parts.extensions
    }

    pub fn take_my_hyper_http_request(&mut self) -> MyHyperHttpRequest {
        match self.body.take() {
            Some(body) => match body {
                HttpRequestBody::Incoming { mut incoming, .. } => {
                    let result =
                        hyper::Request::from_parts(self.parts.clone(), incoming.take().unwrap());

                    return MyHyperHttpRequest::Incoming(result);
                }
                HttpRequestBody::Full(body) => {
                    let body = body.as_slice().to_vec();

                    let body = http_body_util::Full::new(bytes::Bytes::from(body));

                    let req = hyper::Request::from_parts(self.parts.clone(), body);

                    MyHyperHttpRequest::Full(req)
                }
            },
            None => {
                panic!("Body is taken by some middleware before")
            }
        }
    }
}
