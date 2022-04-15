use serde::de::DeserializeOwned;
use std::net::SocketAddr;

use crate::{
    http_path::{GetPathValueResult, PathSegments},
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
    query_string: Option<QueryString>,
    form_data: Option<QueryString>,
    raw_body: Option<Vec<u8>>,
}

impl HttpRequest {
    pub fn new(req: Request<Body>, addr: SocketAddr) -> Self {
        let uri = req.uri().clone();

        let path_lower_case = req.uri().path().to_lowercase();
        let method = req.method().clone();

        Self {
            req: Some(req),
            path_lower_case,
            addr,
            route: None,
            uri,
            method,
            query_string: None,
            form_data: None,
            raw_body: None,
        }
    }

    pub fn init_query_string(&mut self) -> Result<(), HttpFailResult> {
        match self.uri.query() {
            Some(src) => {
                let query_string = QueryString::new(src, QueryStringDataSource::QueryString)?;
                self.query_string = Some(query_string);
                Ok(())
            }
            None => Err(HttpFailResult::as_forbidden(Some(
                "No query string found".to_string(),
            ))),
        }
    }

    pub async fn init_form_data(&mut self) -> Result<(), HttpFailResult> {
        if self.raw_body.is_none() {
            self.init_body().await?;
        }

        let form_data = self.extract_form_data_from_body().await?;
        self.form_data = Some(form_data);
        Ok(())
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

    pub async fn init_body(&mut self) -> Result<(), HttpFailResult> {
        let mut req = None;

        std::mem::swap(&mut self.req, &mut req);

        if req.is_none() {
            panic!("You are trying to get access to body for a second time which is not allowed");
        }

        let body = req.unwrap().into_body();
        let full_body = hyper::body::to_bytes(body).await?;

        self.raw_body = Some(full_body.into_iter().collect::<Vec<u8>>());

        Ok(())
    }

    pub fn get_body_as_slice(&self) -> Result<&[u8], HttpFailResult> {
        if let Some(body) = &self.raw_body {
            return Ok(body);
        }

        Err(HttpFailResult::as_fatal_error(
            "You are trying to get access to body. You have to init_body first".to_string(),
        ))
    }

    pub fn get_body(&mut self) -> Result<Vec<u8>, HttpFailResult> {
        let mut result = None;

        std::mem::swap(&mut self.raw_body, &mut result);

        if let Some(body) = result {
            return Ok(body);
        }

        Err(HttpFailResult::as_fatal_error(
            "You are trying to get access to body. You have to init_body first. Or body is already taken".to_string(),
        ))
    }

    pub fn get_body_as_str(&self) -> Result<&str, HttpFailResult> {
        let body_as_bytes = self.get_body_as_slice()?;

        match std::str::from_utf8(body_as_bytes) {
            Ok(result) => Ok(result),
            Err(err) => Err(HttpFailResult::as_fatal_error(format!("{}", err))),
        }
    }

    pub fn get_body_as_json<T>(&self) -> Result<T, HttpFailResult>
    where
        T: DeserializeOwned,
    {
        let body_as_bytes = self.get_body_as_slice()?;

        match serde_json::from_slice(body_as_bytes) {
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

    async fn extract_form_data_from_body(&mut self) -> Result<QueryString, HttpFailResult> {
        let body_as_str = self.get_body_as_str()?;

        match QueryString::new(body_as_str, QueryStringDataSource::FormData) {
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
        if let Some(query_string) = self.query_string.as_ref() {
            return Ok(query_string);
        }

        Err(HttpFailResult::as_fatal_error(
            "Query String is not initialized".to_owned(),
        ))
    }

    pub fn get_form_data(&self) -> Result<&QueryString, HttpFailResult> {
        if let Some(query_string) = self.form_data.as_ref() {
            return Ok(query_string);
        }

        Err(HttpFailResult::as_fatal_error(
            "Form Data is not initialized".to_owned(),
        ))
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
