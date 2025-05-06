use std::collections::HashMap;

use rust_extensions::StrOrString;

use crate::{cookies::*, HttpFailResult, HttpOutput, WebContentType};

use super::HttpOkResult;

pub struct HttpOkResultBuilder {
    pub(crate) headers: Option<HashMap<String, String>>,
    pub(crate) content_type: Option<WebContentType>,
    pub(crate) cookies: CookieJar,
    pub(crate) body: Vec<u8>,
}

impl HttpOkResultBuilder {
    pub fn new() -> Self {
        Self {
            headers: None,
            content_type: None,
            cookies: Default::default(),
            body: Default::default(),
        }
    }

    pub fn set_content_type(&mut self, content_type: WebContentType) -> &mut Self {
        self.content_type = Some(content_type);
        self
    }

    pub fn add_header(
        mut self,
        key: impl Into<StrOrString<'static>>,
        value: impl Into<StrOrString<'static>>,
    ) -> Self {
        let key = key.into();
        let value = value.into();

        if self.headers.is_none() {
            self.headers = Some(HashMap::new());
        }
        self.headers
            .as_mut()
            .unwrap()
            .insert(key.to_string(), value.to_string());

        self
    }

    pub fn add_headers(
        mut self,
        headers: impl Iterator<
            Item = (
                impl Into<StrOrString<'static>>,
                impl Into<StrOrString<'static>>,
            ),
        >,
    ) -> Self {
        for header in headers {
            self = self.add_header(header.0, header.1);
        }

        self
    }

    pub fn set_cookie(mut self, cookie: impl Into<Cookie>) -> Self {
        self.cookies = self.cookies.set_cookie(cookie);
        self
    }

    pub fn set_cookies(mut self, cookies: impl IntoIterator<Item = impl Into<Cookie>>) -> Self {
        for cookie in cookies {
            self.cookies = self.cookies.set_cookie(cookie);
        }

        self
    }

    pub fn into_ok_result(self, write_telemetry: bool) -> Result<HttpOkResult, HttpFailResult> {
        Ok(HttpOkResult {
            write_telemetry,
            #[cfg(feature = "with-telemetry")]
            add_telemetry_tags: my_telemetry::TelemetryEventTagsBuilder::new(),
            output: HttpOutput::Content {
                headers: self.headers,
                content_type: self.content_type,
                set_cookies: if self.cookies.is_empty() {
                    None
                } else {
                    Some(self.cookies)
                },
                content: self.body,
            },
        })
    }

    pub fn build(self, content: Vec<u8>) -> HttpOutput {
        HttpOutput::Content {
            headers: self.headers,
            content_type: self.content_type,
            set_cookies: if self.cookies.is_empty() {
                None
            } else {
                Some(self.cookies)
            },
            content,
        }
    }
}
