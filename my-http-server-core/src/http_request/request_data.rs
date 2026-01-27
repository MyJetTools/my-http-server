use http::{HeaderMap, HeaderValue};

use hyper::Uri;

use crate::{
    BodyContentType, HttpFailResult, HttpRequestBody, HttpRequestBodyContent, MyHyperHttpRequest,
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
        match self.body.as_mut() {
            Some(body) => body.get_http_request_body().await,
            None => {
                panic!("Body is removed and can not be accessed")
            }
        }
    }

    pub async fn receive_body(&mut self) -> Result<HttpRequestBodyContent, HttpFailResult> {
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
