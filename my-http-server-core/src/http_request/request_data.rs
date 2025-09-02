use http::{HeaderMap, HeaderValue};

use hyper::Uri;

use crate::{HttpFailResult, HttpRequestBody, HttpRequestBodyMode, MyHyperHttpRequest};

pub struct RequestData {
    parts: hyper::http::request::Parts,
    body: Option<HttpRequestBodyMode>,
}

impl RequestData {
    pub fn new(req: hyper::Request<hyper::body::Incoming>) -> Self {
        let parts = req.into_parts();
        Self {
            parts: parts.0,
            body: Some(HttpRequestBodyMode::Incoming(Some(parts.1))),
        }
    }

    pub async fn get_body(&mut self) -> Result<&HttpRequestBody, HttpFailResult> {
        match self.body.as_mut() {
            Some(body) => body.get_http_request_body().await,
            None => {
                panic!("Body is removed and can not be accessed")
            }
        }
    }

    pub async fn receive_body(&mut self) -> Result<HttpRequestBody, HttpFailResult> {
        match self.body.take() {
            Some(body) => return body.into_http_request_body().await,
            None => {
                panic!("Body is taken by some middleware before")
            }
        }
    }

    pub fn uri(&self) -> &Uri {
        &self.parts.uri
    }

    pub fn headers(&self) -> &HeaderMap<HeaderValue> {
        &self.parts.headers
    }

    pub fn take_my_hyper_http_request(&mut self) -> MyHyperHttpRequest {
        match self.body.take() {
            Some(body) => match body {
                HttpRequestBodyMode::Incoming(mut incoming) => {
                    let result =
                        hyper::Request::from_parts(self.parts.clone(), incoming.take().unwrap());

                    return MyHyperHttpRequest::Incoming(result);
                }
                HttpRequestBodyMode::Full(body) => {
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
