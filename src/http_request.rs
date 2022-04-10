use serde::de::DeserializeOwned;
use std::net::SocketAddr;

use crate::{
    http_path::{GetPathValueResult, PathSegments},
    url_decoder::UrlDecodeError,
    HttpFailResult, QueryString, QueryStringDataSource, RequestIp, WebContentType,
};
use hyper::{Body, Method, Request, Uri};

pub struct HttpRequest {
    pub method: Method,
    pub uri: Uri,
    pub req: Option<Request<Body>>,
    path_lower_case: String,
    addr: SocketAddr,
    pub route: Option<PathSegments>,
    query_string: Option<Result<QueryString, UrlDecodeError>>,
}

impl HttpRequest {
    pub fn new(req: Request<Body>, addr: SocketAddr) -> Self {
        let uri = req.uri().clone();

        let path_lower_case = req.uri().path().to_lowercase();
        let method = req.method().clone();

        let query_string = if let Some(query) = uri.query() {
            let query_string = QueryString::new(query, QueryStringDataSource::QueryString);
            Some(query_string)
        } else {
            None
        };

        Self {
            req: Some(req),
            path_lower_case,
            addr,
            route: None,
            uri,
            method,
            query_string,
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

    pub async fn get_body_raw(&mut self) -> Result<Vec<u8>, HttpFailResult> {
        let mut req = None;

        std::mem::swap(&mut self.req, &mut req);

        if req.is_none() {
            panic!("You are trying to get access to body for a second time which is not allowed");
        }

        let body = req.unwrap().into_body();
        let full_body = hyper::body::to_bytes(body).await?;

        Ok(full_body.iter().cloned().collect::<Vec<u8>>())
    }

    pub async fn get_body_as_string(&mut self) -> Result<String, HttpFailResult> {
        let body = self.get_body_raw().await?;
        let result = String::from_utf8(body)?;
        Ok(result)
    }

    pub async fn get_body_as_json<T>(&mut self) -> Result<T, HttpFailResult>
    where
        T: DeserializeOwned,
    {
        let body = self.get_body_raw().await?;

        match serde_json::from_slice(body.as_slice()) {
            Ok(result) => {
                return Ok(result);
            }
            Err(err) => return Err(HttpFailResult::as_fatal_error(format!("{}", err))),
        }
    }

    pub fn get_path(&self) -> &str {
        self.uri.path()
    }

    pub fn get_path_lower_case(&self) -> &str {
        &self.path_lower_case
    }

    pub fn get_headers(&self) -> &hyper::HeaderMap<hyper::header::HeaderValue> {
        if let Some(req) = &self.req {
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

    pub async fn get_form_data(&mut self) -> Result<QueryString, HttpFailResult> {
        let body = self.get_body_raw().await?;

        let a = String::from_utf8(body).unwrap();

        match QueryString::new(a.as_str(), QueryStringDataSource::FormData) {
            Ok(result) => return Ok(result),
            Err(err) => {
                let result = HttpFailResult {
                    write_telemetry: true,
                    content: format!("Can not parse Form Data. {:?}", err).into_bytes(),
                    content_type: WebContentType::Text,
                    status_code: 412,
                };

                return Err(result);
            }
        }
    }

    pub fn get_method(&self) -> &Method {
        &self.method
    }

    pub fn get_query_string(&self) -> Result<&QueryString, HttpFailResult> {
        if self.query_string.is_some() {
            let result = self.query_string.as_ref().unwrap();

            if let Err(err) = result {
                return Err(HttpFailResult::as_fatal_error(format!("{:?}", err)));
            }

            return Ok(result.as_ref().unwrap());
        }

        return Err(HttpFailResult::as_forbidden(Some(
            "No query string found".to_string(),
        )));
    }

    pub fn get_optional_value_from_query(&self, key: &str) -> Result<Option<&str>, HttpFailResult> {
        let query_string = self.get_query_string()?;
        let result = match query_string.get_optional_string_parameter(key) {
            Some(result) => Some(result.as_str()),
            None => None,
        };

        Ok(result)
    }

    pub fn get_required_value_from_query(&self, key: &str) -> Result<&str, HttpFailResult> {
        let query_string = self.get_query_string()?;
        let result = query_string.get_required_string_parameter(key)?;
        Ok(result)
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
