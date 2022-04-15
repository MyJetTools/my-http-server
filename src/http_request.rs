use std::net::SocketAddr;

use crate::{
    http_path::{GetPathValueResult, PathSegments},
    HttpFailResult, HttpRequestBody, QueryString, QueryStringDataSource, RequestIp,
};
use hyper::{Body, Method, Request, Uri};

pub enum RequestData {
    AsRaw(Request<Body>),
    AsHttpBody(HttpRequestBody),
    None,
}

impl RequestData {
    pub fn is_none(&self) -> bool {
        match self {
            RequestData::None => true,
            _ => false,
        }
    }
}
pub struct HttpRequest {
    pub method: Method,
    pub uri: Uri,
    pub req: RequestData,
    path_lower_case: String,
    addr: SocketAddr,
    pub route: Option<PathSegments>,
}

impl HttpRequest {
    pub fn new(req: Request<Body>, addr: SocketAddr) -> Self {
        let uri = req.uri().clone();

        let path_lower_case = req.uri().path().to_lowercase();
        let method = req.method().clone();

        Self {
            req: RequestData::AsRaw(req),
            path_lower_case,
            addr,
            route: None,
            uri,
            method,
        }
    }

    pub fn get_query_string(&self) -> Result<QueryString, HttpFailResult> {
        match self.uri.query() {
            Some(src) => {
                let query_string = QueryString::new(src, QueryStringDataSource::QueryString)?;
                Ok(query_string)
            }
            None => Err(HttpFailResult::as_forbidden(Some(
                "No query string found".to_string(),
            ))),
        }
    }

    pub fn get_value_from_path(&self, key: &str) -> Result<&str, HttpFailResult> {
        let path = self.get_path();

        if self.route.is_none() {
            return Err(HttpFailResult::as_forbidden(Some(format!(
                "Path [{}] does not has keys in it",
                path
            ))));
        }

        let route = self.route.as_ref().unwrap();

        match route.get_value(path, key) {
            GetPathValueResult::Value(value) => Ok(value),
            GetPathValueResult::NoKeyInTheRoute => Err(HttpFailResult::as_forbidden(Some(
                format!("Route [{}] does not have key[{}]", route.path, key),
            ))),
            GetPathValueResult::NoValue => Err(HttpFailResult::as_forbidden(Some(format!(
                "Route [{}] does not have value for the path [{}] with the key [{}]",
                route.path,
                self.get_path(),
                key
            )))),
        }
    }

    pub fn get_value_from_path_optional(&self, key: &str) -> Result<Option<&str>, HttpFailResult> {
        let path = self.get_path();

        if self.route.is_none() {
            return Err(HttpFailResult::as_forbidden(Some(format!(
                "No route found to extract key [{}] from the path [{}]",
                key, path
            ))));
        }

        let route = self.route.as_ref().unwrap();

        match route.get_value(path, key) {
            GetPathValueResult::Value(value) => Ok(Some(value)),
            GetPathValueResult::NoValue => Ok(None),
            GetPathValueResult::NoKeyInTheRoute => Err(HttpFailResult::as_forbidden(Some(
                format!("Route [{}] does not have key[{}]", route.path, key),
            ))),
        }
    }

    pub fn get_value_from_path_optional_as_string(
        &self,
        key: &str,
    ) -> Result<Option<String>, HttpFailResult> {
        let result = self.get_value_from_path_optional(key)?;

        match result {
            Some(value) => Ok(Some(value.to_owned())),
            None => Ok(None),
        }
    }

    pub async fn get_body(&mut self) -> Result<HttpRequestBody, HttpFailResult> {
        let mut req = RequestData::None;

        std::mem::swap(&mut self.req, &mut req);

        match req {
            RequestData::AsRaw(req) => {
                let body = req.into_body();
                let full_body = hyper::body::to_bytes(body).await?;

                let body = full_body.into_iter().collect::<Vec<u8>>();

                return Ok(HttpRequestBody::new(body));
            }
            RequestData::AsHttpBody(result) => {
                return Ok(result);
            }
            RequestData::None => {
                panic!(
                    "You are trying to get access to body for a second time which is not allowed"
                );
            }
        }
    }

    pub fn get_path(&self) -> &str {
        self.uri.path()
    }

    pub fn get_path_lower_case(&self) -> &str {
        &self.path_lower_case
    }

    pub fn get_headers(&self) -> &hyper::HeaderMap<hyper::header::HeaderValue> {
        if let RequestData::AsRaw(req) = &self.req {
            return req.headers();
        }

        panic!("Headers can no be read after reading body");
    }

    pub fn get_ip(&self) -> RequestIp {
        let headers = self.get_headers();
        let ip_header = headers.get("X-Forwarded-For");

        if let Some(ip_value) = ip_header {
            let forwared_ip = std::str::from_utf8(ip_value.as_bytes()).unwrap();

            let result: Vec<&str> = forwared_ip.split(",").map(|itm| itm.trim()).collect();

            return RequestIp::Forwarded(result);
        }

        return RequestIp::Result(self.addr.to_string());
    }

    pub fn get_required_header(&self, header_name: &str) -> Result<&str, HttpFailResult> {
        for (http_header, value) in self.get_headers() {
            let http_header = http_header.as_str();
            if http_header == header_name {
                return Ok(value.to_str().unwrap());
            }
        }

        return Err(HttpFailResult::required_parameter_is_missing(
            header_name,
            QueryStringDataSource::QueryString.as_str(),
        ));
    }

    pub fn get_optional_header(&self, header_name: &str) -> Option<&str> {
        for (http_header, value) in self.get_headers() {
            let http_header = http_header.as_str();
            if http_header == header_name {
                return Some(value.to_str().unwrap());
            }
        }

        return None;
    }

    pub fn get_method(&self) -> &Method {
        &self.method
    }

    pub fn get_host(&self) -> &str {
        std::str::from_utf8(&self.get_headers().get("host").unwrap().as_bytes()).unwrap()
    }

    pub fn get_scheme(&self) -> String {
        let headers = self.get_headers();
        let proto_header = headers.get("X-Forwarded-Proto");

        if let Some(scheme) = proto_header {
            let bytes = scheme.as_bytes();
            return String::from_utf8(bytes.to_vec()).unwrap();
        }

        let scheme = self.uri.scheme();

        match scheme {
            Some(scheme) => {
                return scheme.to_string();
            }
            None => "http".to_string(),
        }
    }
}
