use hyper::{header::*, *};
use hyper_tungstenite::tungstenite::http::Extensions;
use my_http_server_core::{CookiesReader, MyHyperHttpRequest, RequestIp, SocketAddress};

pub struct MyWebSocketHttpRequest {
    uri: Uri,
    headers: HeaderMap<HeaderValue>,
    version: Version,
    extensions: Extensions,
    addr: SocketAddress,
}

impl<'s> MyWebSocketHttpRequest {
    pub fn new(req: &MyHyperHttpRequest, addr: SocketAddress) -> Self {
        Self {
            uri: req.uri().clone(),
            headers: req.headers().clone(),
            version: req.version(),
            extensions: req.extensions().clone(),
            addr,
        }
    }

    pub fn get_uri(&self) -> &Uri {
        &self.uri
    }

    pub fn get_headers(&self) -> &HeaderMap<HeaderValue> {
        &self.headers
    }

    pub fn get_http_version(&self) -> Version {
        self.version
    }

    pub fn get_extensions(&self) -> &Extensions {
        &self.extensions
    }

    pub fn get_ip(&'s self) -> RequestIp<'s> {
        RequestIp::new(&self.addr, self.get_headers())
    }

    pub fn get_cookies(&'s self) -> CookiesReader<'s> {
        let Some(header) = self.get_headers().get("cookie") else {
            return CookiesReader::new(None);
        };

        match header.to_str() {
            Ok(cookie) => CookiesReader::new(Some(cookie)),
            Err(_) => CookiesReader::new(None),
        }
    }
}
