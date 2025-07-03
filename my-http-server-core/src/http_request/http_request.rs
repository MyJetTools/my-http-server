use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use crate::{
    http_headers_to_use::*, CookiesReader, HttpFailResult, HttpPath, HttpPathReader,
    HttpRequestBody, HttpRequestHeaders, RequestData, RequestIp, UrlEncodedData,
};

use hyper::{Method, Uri};
use url_utils::url_decoder::UrlDecodeError;

#[derive(Debug, Clone)]
pub enum SocketAddress {
    Tcp(SocketAddr),
    Unix(Arc<String>),
}

impl SocketAddress {
    pub fn to_string(&self) -> String {
        match self {
            SocketAddress::Tcp(socket_addr) => socket_addr.to_string(),
            SocketAddress::Unix(addr) => addr.to_string(),
        }
    }

    pub fn ip_as_string(&self) -> String {
        match self {
            SocketAddress::Tcp(socket_addr) => socket_addr.ip().to_string(),
            SocketAddress::Unix(addr) => addr.to_string(),
        }
    }
}

pub struct HttpRequest {
    pub data: RequestData,
    pub addr: SocketAddress,
    pub content_type_header: Option<String>,
    key_values: Option<HashMap<String, Vec<u8>>>,
    pub method: Method,
    pub http_path: HttpPath,
}

impl HttpRequest {
    pub fn new(req: hyper::Request<hyper::body::Incoming>, addr: SocketAddress) -> Self {
        let method = req.method().clone();

        let http_path = HttpPath::from_str(req.uri().path());

        Self {
            data: RequestData::Incoming(Some(req)),
            addr,
            key_values: None,
            content_type_header: None,
            method,
            http_path,
        }
    }

    pub fn get_query_string(&self) -> Result<UrlEncodedData, UrlDecodeError> {
        if let Some(query) = self.data.uri().query() {
            let result = UrlEncodedData::from_query_string(query)?;
            Ok(result)
        } else {
            Ok(UrlEncodedData::new_query_string_empty())
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

    pub async fn get_body(&mut self) -> Result<&HttpRequestBody, HttpFailResult> {
        let result = self.data.convert_to_body_if_requires().await?;
        Ok(result.unwrap())
    }

    pub async fn receive_body(&mut self) -> Result<HttpRequestBody, HttpFailResult> {
        self.data.receive_body().await
    }

    pub fn take_incoming_body(&mut self) -> hyper::Request<hyper::body::Incoming> {
        self.data.take_incoming_body()
    }

    pub fn get_path(&self) -> HttpPathReader {
        HttpPathReader::new(self.data.uri().path())
    }

    pub fn get_ip(&self) -> RequestIp {
        let x_forwarded_for = self
            .data
            .headers()
            .try_get_case_sensitive_as_str(X_FORWARDED_FOR_HEADER);

        if let Ok(x_forwarded_for) = x_forwarded_for {
            if let Some(x_forwarded_for) = x_forwarded_for {
                let result: Vec<&str> = x_forwarded_for.split(",").map(|itm| itm.trim()).collect();
                return RequestIp::Forwarded(result);
            }
        }

        return RequestIp::create_as_single_ip(self.addr.ip_as_string());
    }

    pub fn get_host(&self) -> &str {
        if let Some(value) = self.data.headers().try_get_case_insensitive("host") {
            return value.as_str().unwrap();
        }
        panic!("Host is not set");
    }

    pub fn get_path_and_query(&self) -> &str {
        match self.data.uri().path_and_query() {
            Some(path_and_query) => path_and_query.as_str(),
            None => {
                let path = self.data.uri().path();
                if path.is_empty() {
                    return "/";
                }
                path
            }
        }
    }

    pub fn get_headers(&self) -> &impl HttpRequestHeaders {
        self.data.headers()
    }

    pub fn get_uri(&self) -> &Uri {
        self.data.uri()
    }

    pub fn get_scheme(&self) -> &str {
        let x_forwarded_proto = self
            .data
            .headers()
            .try_get_case_sensitive_as_str(X_FORWARDED_PROTO);

        if let Ok(x_forwarded_proto) = x_forwarded_proto {
            if let Some(x_forwarded_proto) = x_forwarded_proto {
                return x_forwarded_proto;
            }
        }

        let scheme = self.data.uri().scheme();

        match scheme {
            Some(scheme) => {
                return scheme.as_str();
            }
            None => "http",
        }
    }

    pub fn get_cookies(&self) -> CookiesReader {
        let cookie_header = self.data.headers().try_get_case_insensitive("cookie");

        if cookie_header.is_none() {
            return CookiesReader::new(None);
        }

        match cookie_header.unwrap().as_str() {
            Ok(cookie) => CookiesReader::new(Some(cookie)),
            Err(_) => {
                return CookiesReader::new(None);
            }
        }
    }
}
