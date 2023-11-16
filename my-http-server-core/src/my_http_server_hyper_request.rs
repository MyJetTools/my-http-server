use http_body_util::BodyExt;
use hyper::{
    body::{Body, Bytes},
    header::HeaderValue,
    HeaderMap, Method, Uri,
};

use crate::{HttpFailResult, HttpPath, HttpRequestBody};

pub enum BodyState {
    Request(Option<hyper::Request<hyper::body::Incoming>>),
    Incoming(Option<hyper::body::Incoming>),
    Body(HttpRequestBody),
}

impl BodyState {
    pub async fn convert_to_body_if_requires(
        &mut self,
    ) -> Result<Option<&HttpRequestBody>, HttpFailResult> {
        let bytes = match self {
            BodyState::Request(request) => {
                todo!("Make sure it works");
                /*
                let request = request.take().unwrap();
                let (parts, body) = request.into_parts();
                let bytes = read_bytes(body).await?;
                 */
            }
            BodyState::Incoming(incoming) => {
                let incoming = incoming.take().unwrap();
                read_bytes(incoming).await?
            }
            BodyState::Body(body) => {
                return Ok(Some(body));
            }
        };

        let result = HttpRequestBody::new(bytes, None);
        *self = BodyState::Body(result);
        Ok(self.try_unwrap_as_body())
    }

    fn try_unwrap_as_body(&self) -> Option<&HttpRequestBody> {
        match self {
            BodyState::Request(_) => None,
            BodyState::Incoming(_) => None,
            BodyState::Body(body) => {
                return Some(body);
            }
        }
    }

    fn unwrap_as_request(&self) -> &hyper::Request<hyper::body::Incoming> {
        match self {
            BodyState::Request(req) => {
                todo!("implement")
            }
            BodyState::Incoming(_) => panic!("Body is already converted to Incoming state"),
            BodyState::Body(body) => panic!("Body is already converted to Read content state"),
        }
    }

    fn try_unwrap_as_request(&self) -> Option<&hyper::Request<hyper::body::Incoming>> {
        match self {
            BodyState::Request(req) => {
                todo!("implement")
            }
            BodyState::Incoming(_) => panic!("Body is already converted to Incoming state"),
            BodyState::Body(body) => panic!("Body is already converted to Read content state"),
        }
    }
}

pub struct MyHttpServerHyperRequest {
    body: BodyState,
    headers: HeaderMap,
    method: Method,
    uri: Uri,
    http_path: HttpPath,
}

impl MyHttpServerHyperRequest {
    pub fn new(req: hyper::Request<hyper::body::Incoming>) -> Self {
        let (parts, body) = req.into_parts();

        let headers = parts.headers;
        let method = parts.method;
        let uri = parts.uri;

        let http_path = HttpPath::from_str(uri.path());

        Self {
            body: BodyState::Incoming(body.into()),
            headers,
            method,
            uri,
            http_path,
        }
    }
    pub fn get_http_path(&self) -> &HttpPath {
        &self.http_path
    }

    pub fn get_uri(&self) -> &Uri {
        &self.uri
    }

    pub fn get_header_case_sensitive(&self, header_name: &str) -> Option<&HeaderValue> {
        self.headers.get(header_name)
    }

    pub fn get_header_case_insensitive(&self, header_name: &str) -> Option<&str> {
        let header_name = header_name.to_lowercase();

        for (key, value) in &self.headers {
            if key.as_str().to_lowercase() == header_name {
                match value.to_str() {
                    Ok(value) => return Some(value),
                    Err(_) => return None,
                }
            }
        }

        None
    }

    pub fn get_header_as_str_case_sensitive(&self, header_name: &str) -> Option<&str> {
        self.get_header_case_sensitive(header_name)?.to_str().ok()
    }

    pub fn get_method(&self) -> Method {
        self.method.clone()
    }

    pub async fn get_body(&mut self) -> Result<&HttpRequestBody, HttpFailResult> {
        let result = self.body.convert_to_body_if_requires().await?;
        Ok(result.unwrap())
    }

    pub fn unwrap_as_request(&self) -> &hyper::Request<hyper::body::Incoming> {
        self.body.unwrap_as_request()
    }

    pub fn try_unwrap_as_request(&self) -> Option<&hyper::Request<hyper::body::Incoming>> {
        self.body.try_unwrap_as_request()
    }
}

async fn read_bytes(
    incoming: impl Body<Data = Bytes, Error = hyper::Error>,
) -> Result<Vec<u8>, HttpFailResult> {
    let collected = incoming.collect().await?;
    let bytes = collected.to_bytes();
    Ok(bytes.into())
}
