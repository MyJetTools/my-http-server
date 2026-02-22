use http::response::Builder;
use rust_extensions::StrOrString;

use crate::{cookies::*, WebContentType};

#[derive(Default, Debug)]
pub struct HttpResponseHeaders {
    pub headers: Vec<(StrOrString<'static>, String)>,
    pub content_type: Option<WebContentType>,
    pub set_cookie: Option<CookieJar>,
}

impl HttpResponseHeaders {
    pub fn new(content_type: Option<WebContentType>) -> Self {
        Self {
            headers: Default::default(),
            content_type,
            set_cookie: Default::default(),
        }
    }

    pub fn new_with_header(name: StrOrString<'static>, value: String) -> Self {
        let mut result = Self {
            headers: Default::default(),
            content_type: Default::default(),
            set_cookie: Default::default(),
        };

        result.add_header(name, value);

        result
    }

    pub fn new_with_cookie(cookie: Cookie) -> Self {
        Self {
            headers: Default::default(),
            content_type: Default::default(),
            set_cookie: Some(CookieJar::new().set_cookie(cookie)),
        }
    }

    pub fn new_with_cookies(cookies: CookieJar) -> Self {
        Self {
            headers: Default::default(),
            content_type: Default::default(),
            set_cookie: Some(cookies),
        }
    }

    pub fn add_header(&mut self, key: StrOrString<'static>, value: String) {
        self.headers.push((key, value));
    }

    pub fn set_content_type(&mut self, content_type: WebContentType) {
        self.content_type = Some(content_type);
    }

    pub fn set_cookie(&mut self, cookie: Cookie) {
        let cookies = if let Some(cookies) = self.set_cookie.take() {
            cookies
        } else {
            CookieJar::new()
        };

        self.set_cookie = Some(cookies.set_cookie(cookie));
    }

    pub fn set_cookies(&mut self, cookies_to_set: impl IntoIterator<Item = impl Into<Cookie>>) {
        let mut cookies = if let Some(cookies) = self.set_cookie.take() {
            cookies
        } else {
            CookieJar::new()
        };

        for cookie in cookies_to_set {
            cookies = cookies.set_cookie(cookie);
        }

        self.set_cookie = Some(cookies);
    }

    pub fn populate_headers(self, mut builder: Builder) -> Builder {
        if let Some(content_type) = self.content_type {
            builder = builder.header("content-type", content_type.as_str());
        }

        for (key, value) in self.headers {
            builder = builder.header(key.as_str(), value);
        }

        if let Some(cookies) = self.set_cookie {
            for itm in cookies.get_cookies() {
                builder = builder.header("Set-Cookie", itm.to_string());
            }
        }

        builder
    }
}
