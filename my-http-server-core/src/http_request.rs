use std::{collections::HashMap, net::SocketAddr};

use crate::{
    http_path::HttpPath, HttpFailResult, HttpRequestBody, InputParamValue, RequestIp,
    UrlEncodedData,
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
    pub http_path: HttpPath,
    pub addr: SocketAddr,
    pub content_type_header: Option<String>,
    key_values: Option<HashMap<String, Vec<u8>>>,
    x_forwarded_proto: Option<String>,
    x_forwarded_for: Option<String>,
    host: Option<String>,
    #[cfg(feature = "cache-headers-before-receive-body")]
    cached_headers: Option<crate::CachedHeaders>,
}

impl HttpRequest {
    pub fn new(req: Request<Body>, addr: SocketAddr) -> Self {
        let uri = req.uri().clone();

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
            http_path: HttpPath::from_str(req.uri().path()),
            req: RequestData::AsRaw(req),
            addr,
            uri,
            method,
            key_values: None,
            x_forwarded_proto,
            x_forwarded_for,
            host,
            content_type_header: None,
            #[cfg(feature = "cache-headers-before-receive-body")]
            cached_headers: None,
        }
    }

    pub fn get_query_string(&self) -> Result<UrlEncodedData, HttpFailResult> {
        if let Some(query) = self.uri.query() {
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

    async fn init_body(&mut self) -> Result<(), HttpFailResult> {
        if self.content_type_header.is_none() {
            if let Some(value) = self.get_optional_header("content-type") {
                self.content_type_header = Some(value.as_string()?);
            }
        }

        if self.req.is_http_body() {
            return Ok(());
        }

        if self.req.is_none() {
            return Ok(());
        }

        let mut result = RequestData::None;
        std::mem::swap(&mut self.req, &mut result);

        #[cfg(feature = "cache-headers-before-receive-body")]
        if let RequestData::AsRaw(req) = &mut result {
            self.cached_headers = Some(crate::CachedHeaders::new(req));
        }

        if let RequestData::AsRaw(req) = result {
            let body = req.into_body();
            let full_body = hyper::body::to_bytes(body).await?;

            let body = full_body.into_iter().collect::<Vec<u8>>();

            self.req = RequestData::AsHttpBody(HttpRequestBody::new(
                body,
                self.content_type_header.take(),
            ));
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

    fn get_headers(&self) -> &hyper::HeaderMap<hyper::header::HeaderValue> {
        if let RequestData::AsRaw(req) = &self.req {
            return req.headers();
        }

        panic!("Headers can no be read after reading body");
    }

    pub fn get_ip(&self) -> RequestIp {
        if let Some(x_forwarded_for) = &self.x_forwarded_for {
            let result: Vec<&str> = x_forwarded_for.split(",").map(|itm| itm.trim()).collect();
            return RequestIp::Forwarded(result);
        }

        return RequestIp::create_as_single_ip(self.addr.ip().to_string());
    }

    pub fn get_required_header(
        &self,
        header_name: &str,
    ) -> Result<InputParamValue, HttpFailResult> {
        match self.get_header(header_name) {
            Some(value) => Ok(InputParamValue::Raw {
                value,
                src: "header",
            }),
            None => {
                return HttpFailResult::invalid_value_to_parse(format!(
                    "Can not convert header {} value to string",
                    header_name
                ))
                .into_err();
            }
        }
    }

    pub fn get_optional_header(&self, header_name: &str) -> Option<InputParamValue> {
        let value = self.get_header(header_name)?;

        Some(InputParamValue::Raw {
            value,
            src: "header",
        })
    }

    pub fn get_header(&self, header_name: &str) -> Option<&str> {
        let header_name = header_name.to_lowercase();
        #[cfg(feature = "cache-headers-before-receive-body")]
        if let Some(cached_headers) = &self.cached_headers {
            match cached_headers.get(&header_name) {
                Some(header_value) => return Some(header_value.to_str().unwrap()),
                None => return None,
            }
        }

        for (key, value) in self.get_headers() {
            if key.as_str().to_lowercase() == header_name {
                match value.to_str() {
                    Ok(value) => return Some(value),
                    Err(_) => return None,
                }
            }
        }

        None
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
