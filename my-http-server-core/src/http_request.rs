use std::{collections::HashMap, net::SocketAddr};

use crate::{
    HttpFailResult, HttpPath, HttpRequestBody, InputParamValue, MyHttpServerHyperRequest,
    RequestIp, UrlEncodedData,
};

use hyper::Method;

const X_FORWARDED_FOR_HEADER: &str = "X-Forwarded-For";
const X_FORWARDED_PROTO: &str = "X-Forwarded-Proto";

pub enum RequestData {
    AsRaw(MyHttpServerHyperRequest),
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
    req: MyHttpServerHyperRequest,
    pub addr: SocketAddr,
    pub content_type_header: Option<String>,
    key_values: Option<HashMap<String, Vec<u8>>>,
}

impl HttpRequest {
    pub fn new(req: MyHttpServerHyperRequest, addr: SocketAddr) -> Self {
        Self {
            req,
            addr,
            key_values: None,
            content_type_header: None,
        }
    }

    pub fn get_query_string(&self) -> Result<UrlEncodedData, HttpFailResult> {
        if let Some(query) = self.req.get_uri().query() {
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

    /*
       async fn init_body(&mut self, cache_headers: bool) -> Result<(), HttpFailResult> {
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

           if cache_headers {
               if let RequestData::AsRaw(req) = &mut result {
                   self.cached_headers = Some(crate::CachedHeaders::new(req));
               }
           }

           if let RequestData::AsRaw(req) = result {
               let (parts, incoming) = req.into_parts();

               let body = read_bytes(incoming).await;

               self.req = RequestData::AsHttpBody(HttpRequestBody::new(
                   body,
                   self.content_type_header.take(),
               ));
           }

           Ok(())
       }
    */

    pub async fn get_body(&mut self) -> Result<&HttpRequestBody, HttpFailResult> {
        self.req.get_body().await
    }

    /*
        pub async fn receive_body(
            &mut self,
            cache_headers: bool,
        ) -> Result<HttpRequestBody, HttpFailResult> {
            self.init_body(cache_headers).await?;

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
    */
    pub fn get_path(&self) -> &str {
        self.req.get_uri().path()
    }

    /*
    fn get_headers(&self) -> &hyper::HeaderMap<hyper::header::HeaderValue> {
        if let RequestData::AsRaw(req) = &self.req {
            return req.headers();
        }

        panic!("Headers can no be read after reading body");
    }
     */

    pub fn get_ip(&self) -> RequestIp {
        if let Some(x_forwarded_for) = &self
            .req
            .get_header_as_str_case_sensitive(X_FORWARDED_FOR_HEADER)
        {
            let result: Vec<&str> = x_forwarded_for.split(",").map(|itm| itm.trim()).collect();
            return RequestIp::Forwarded(result);
        }

        return RequestIp::create_as_single_ip(self.addr.ip().to_string());
    }

    pub fn get_required_header(
        &self,
        header_name: &str,
    ) -> Result<InputParamValue, HttpFailResult> {
        match self.req.get_header_case_insensitive(header_name) {
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

    pub fn get_http_path(&self) -> &HttpPath {
        self.req.get_http_path()
    }

    pub fn get_optional_header(&self, header_name: &str) -> Option<InputParamValue> {
        let value = self.req.get_header_case_insensitive(header_name)?;

        Some(InputParamValue::Raw {
            value,
            src: "header",
        })
    }

    pub fn get_method(&self) -> Method {
        self.req.get_method()
    }

    pub fn get_host(&self) -> &str {
        if let Some(value) = self.req.get_header_case_insensitive("host") {
            return value;
        }
        panic!("Host is not set");
    }

    pub fn get_scheme(&self) -> &str {
        if let Some(x_forwarded_proto) =
            self.req.get_header_as_str_case_sensitive(X_FORWARDED_PROTO)
        {
            return std::str::from_utf8(x_forwarded_proto.as_bytes()).unwrap();
        }

        let scheme = self.req.get_uri().scheme();

        match scheme {
            Some(scheme) => {
                return scheme.as_str();
            }
            None => "http",
        }
    }

    pub fn unwrap_raw_request(&self) -> &hyper::Request<hyper::body::Incoming> {
        self.req.unwrap_as_request()
    }

    pub fn try_unwrap_raw_request(&self) -> Option<&hyper::Request<hyper::body::Incoming>> {
        self.req.try_unwrap_as_request()
    }
}
