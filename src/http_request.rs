use std::net::SocketAddr;

use crate::{
    http_path::{GetPathValueResult, PathSegments},
    HttpFailResult, HttpRequestBody, QueryString, QueryStringDataSource, RequestIp,
};
use hyper::{Body, Method, Request, Uri};

#[derive(Clone, Debug)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Options,
    Connect,
    Trace,
    Head,
}

impl HttpMethod {
    pub fn new(hyper_method: &Method) -> Self {
        match hyper_method {
            &Method::GET => HttpMethod::Get,
            &Method::POST => HttpMethod::Post,
            &Method::PUT => HttpMethod::Put,
            &Method::DELETE => HttpMethod::Delete,
            &Method::OPTIONS => HttpMethod::Options,
            &Method::CONNECT => HttpMethod::Connect,
            &Method::TRACE => HttpMethod::Trace,
            &Method::HEAD => HttpMethod::Head,
            _ => panic!("Method {} is not supported", hyper_method),
        }
    }
}

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
    pub fn is_http_body(&self) -> bool {
        match self {
            RequestData::AsHttpBody(_) => true,
            _ => false,
        }
    }
}
pub struct HttpRequest<'s> {
    pub method: HttpMethod,
    pub uri: Uri,
    pub req: RequestData,
    path_lower_case: String,
    addr: SocketAddr,
    pub route: Option<PathSegments>,
    query_string: Option<QueryString<'s>>,
}

impl<'s> HttpRequest<'s> {
    pub fn new(req: Request<Body>, addr: SocketAddr) -> Self {
        let uri = req.uri().clone();

        let path_lower_case = req.uri().path().to_lowercase();
        let method = HttpMethod::new(&req.method());

        Self {
            req: RequestData::AsRaw(req),
            path_lower_case,
            addr,
            route: None,
            uri,
            method,
            query_string: None,
        }
    }

    pub fn init_query_string(&'s mut self) -> Result<(), HttpFailResult> {
        if self.query_string.is_some() {
            return Ok(());
        }
        match self.uri.query() {
            Some(src) => {
                let query_string = QueryString::new(src, QueryStringDataSource::QueryString)?;
                self.query_string = query_string.into();
                Ok(())
            }
            None => Err(HttpFailResult::as_forbidden(Some(
                "No query string found".to_string(),
            ))),
        }
    }

    pub fn get_query_string(&'s self) -> Result<&QueryString<'s>, HttpFailResult> {
        if self.query_string.is_none() {}
        Ok(self.query_string.as_ref().unwrap())
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

    async fn init_body(&mut self) -> Result<(), HttpFailResult> {
        if self.req.is_http_body() {
            return Ok(());
        }

        if self.req.is_none() {
            return Ok(());
        }

        let mut result = RequestData::None;
        std::mem::swap(&mut self.req, &mut result);

        if let RequestData::AsRaw(req) = result {
            let body = req.into_body();
            let full_body = hyper::body::to_bytes(body).await?;

            let body = full_body.into_iter().collect::<Vec<u8>>();

            self.req = RequestData::AsHttpBody(HttpRequestBody::new(body));
        }

        Ok(())
    }

    pub async fn get_body(&'s mut self) -> Result<&'s HttpRequestBody, HttpFailResult> {
        self.init_body().await?;

        match &self.req {
            RequestData::AsRaw(_) => {
                panic!("Somehow we are here");
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

    pub async fn receive_body(&'s mut self) -> Result<HttpRequestBody, HttpFailResult> {
        self.init_body().await?;

        let mut result = RequestData::None;
        std::mem::swap(&mut self.req, &mut result);

        match result {
            RequestData::AsRaw(_) => {
                panic!("Somehow we are here");
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

    pub fn set_body(&mut self, body: HttpRequestBody) {
        self.req = RequestData::AsHttpBody(body);
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

    pub fn get_method(&self) -> HttpMethod {
        self.method.clone()
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
