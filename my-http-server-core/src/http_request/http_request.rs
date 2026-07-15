use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use crate::{
    http_headers::*, CookiesReader, HttpFailResult, HttpPath, HttpPathReader,
    HttpRequestBodyContent, HttpRequestHeaders, MyHyperHttpRequest, QueryStringReader, RequestData,
    RequestIp,
};

use hyper::{Method, Uri};

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
    pub fn new(
        req: hyper::Request<hyper::body::Incoming>,
        addr: SocketAddress,
    ) -> Result<Self, HttpFailResult> {
        let method = req.method().clone();

        let http_path = HttpPath::from_str(req.uri().path());

        let result = Self {
            data: RequestData::new(req)?,
            addr,
            key_values: None,
            content_type_header: None,
            method,
            http_path,
        };

        Ok(result)
    }

    pub fn get_query_string<'s>(&'s self) -> Result<QueryStringReader<'s>, HttpFailResult> {
        let query = self.data.uri().query().unwrap_or("");
        Ok(QueryStringReader::new(query)?)
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

    pub async fn get_body(&mut self) -> Result<&HttpRequestBodyContent, HttpFailResult> {
        self.data.get_body().await
    }

    pub async fn receive_body(&mut self) -> Result<HttpRequestBodyContent, HttpFailResult> {
        self.data.receive_body().await
    }

    pub fn take_my_hyper_http_request(&mut self) -> MyHyperHttpRequest {
        self.data.take_my_hyper_http_request()
    }

    pub fn get_path<'s>(&'s self) -> HttpPathReader<'s> {
        HttpPathReader::new(self.data.uri().path())
    }

    pub fn get_ip<'s>(&'s self) -> RequestIp<'s> {
        RequestIp::new(&self.addr, self.get_headers())
    }

    pub fn get_host(&self) -> &str {
        // HTTP/1 sends the host in the `Host` header — check it first.
        if let Some(value) = self.data.headers().try_get_case_insensitive("host") {
            return value.as_str().unwrap();
        }

        // Behind a reverse proxy the original host may come in X-Forwarded-Host.
        if let Some(value) = self.data.headers().try_get_case_insensitive(X_FORWARDED_HOST) {
            return value.as_str().unwrap();
        }

        // HTTP/2 carries the host in the `:authority` pseudo-header (surfaced by hyper as the URI
        // authority).
        if let Some(authority) = self.data.uri().authority() {
            return authority.as_str();
        }

        panic!("Host is not set: neither Host header, X-Forwarded-Host, nor :authority is present");
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

    pub fn extensions(&self) -> &http::Extensions {
        self.data.extensions()
    }

    pub fn is_h2_websocket_connect(&self) -> bool {
        if self.method != Method::CONNECT {
            return false;
        }
        match self.data.extensions().get::<hyper::ext::Protocol>() {
            Some(protocol) => protocol.as_str().eq_ignore_ascii_case("websocket"),
            None => false,
        }
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

    pub fn get_cookies<'s>(&'s self) -> CookiesReader<'s> {
        let cookie_header = self.data.headers().try_get_case_insensitive("cookie");

        let Some(cookie_header) = cookie_header else {
            return CookiesReader::new(None);
        };

        match cookie_header.as_str() {
            Ok(cookie) => CookiesReader::new(Some(cookie)),
            Err(_) => {
                return CookiesReader::new(None);
            }
        }
    }
}
