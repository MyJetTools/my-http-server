use std::{collections::HashMap, net::SocketAddr};

use crate::{
    http_path::{GetPathValueResult, PathSegments},
    HttpFailResult, HttpRequestBody, RequestIp, UrlEncodedData, UrlEncodedDataSource,
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
    pub fn is_http_body(&self) -> bool {
        match self {
            RequestData::AsHttpBody(_) => true,
            _ => false,
        }
    }
}
pub struct HttpRequest {
    pub method: Method,
    pub uri: Uri,
    pub req: RequestData,
    path_lower_case: String,
    pub addr: SocketAddr,
    pub route: Option<PathSegments>,
    key_values: Option<HashMap<String, Vec<u8>>>,
    x_forwarded_proto: Option<String>,
    x_forwarded_for: Option<String>,
    host: Option<String>,
}

impl HttpRequest {
    pub fn new(req: Request<Body>, addr: SocketAddr) -> Self {
        println!("Scheme:{:?}; Uri:{:?}", req.uri().scheme(), req.uri());

        let uri = req.uri().clone();

        let path_lower_case = req.uri().path().to_lowercase();
        let method = req.method().clone();

        let x_forwarded_for = if let Some(value) = req.headers().get("X-Forwarded-For") {
            Some(value.to_str().unwrap().to_string())
        } else {
            None
        };

        let x_forwarded_proto = if let Some(value) = req.headers().get("X-Forwarded-Proto") {
            Some(value.to_str().unwrap().to_string())
        } else {
            None
        };

        let host = if let Some(value) = req.headers().get("host") {
            Some(value.to_str().unwrap().to_string())
        } else {
            None
        };

        Self {
            req: RequestData::AsRaw(req),
            path_lower_case,
            addr,
            route: None,
            uri,
            method,
            key_values: None,
            x_forwarded_proto,
            x_forwarded_for,
            host,
        }
    }

    pub fn get_query_string(&self) -> Result<UrlEncodedData, HttpFailResult> {
        if let Some(query) = self.uri.query() {
            let result = UrlEncodedData::new(query, UrlEncodedDataSource::QueryString)?;
            Ok(result)
        } else {
            Err(HttpFailResult::as_forbidden(Some(
                "No query string found".to_string(),
            )))
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

    pub fn set_key_value(&mut self, key: String, value: Vec<u8>) -> Option<Vec<u8>> {
        if self.key_values.is_none() {
            self.key_values = Some(HashMap::new());
        }

        self.key_values.as_mut().unwrap().insert(key, value)
    }

    pub fn get_key_value(&self, key: &str) -> Option<&[u8]> {
        let result = self.key_values.as_ref()?.get(key)?;

        Some(result)
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

    pub async fn get_body(&mut self) -> Result<&HttpRequestBody, HttpFailResult> {
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

    pub async fn receive_body(&mut self) -> Result<HttpRequestBody, HttpFailResult> {
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
        if let Some(x_forwared_for) = &self.x_forwarded_for {
            let result: Vec<&str> = x_forwared_for.split(",").map(|itm| itm.trim()).collect();
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
            UrlEncodedDataSource::QueryString.as_str(),
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
        if let Some(host) = &self.host {
            return host;
        }

        panic!("Host is not set");
    }

    pub fn get_scheme(&self) -> &str {
        if let Some(x_forwarded_proto) = &self.x_forwarded_proto {
            return x_forwarded_proto;
        }

        let scheme = self.uri.scheme();

        match scheme {
            Some(scheme) => {
                return scheme.as_str();
            }
            None => "http",
        }
    }
}
