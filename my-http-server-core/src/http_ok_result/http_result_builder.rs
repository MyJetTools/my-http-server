use std::collections::HashMap;

use rust_extensions::StrOrString;

use crate::{cookies::*, AddHttpHeaders, HttpFailResult, HttpOutput, WebContentType};

use super::HttpOkResult;

pub struct HttpResultBuilder {
    pub(crate) status_code: u16,
    pub(crate) headers: Option<HashMap<String, String>>,
    pub(crate) content_type: Option<WebContentType>,
    pub(crate) cookies: Option<CookieJar>,
    pub(crate) content: Vec<u8>,
}

impl HttpResultBuilder {
    pub fn new() -> Self {
        Self {
            status_code: 200,
            headers: None,
            content_type: None,
            cookies: Default::default(),
            content: Default::default(),
        }
    }

    pub fn set_content_type(mut self, content_type: WebContentType) -> Self {
        self.content_type = Some(content_type);
        self
    }

    pub fn set_content_type_opt(mut self, content_type: Option<WebContentType>) -> Self {
        if let Some(content_type) = content_type {
            self.content_type = Some(content_type);
        }
        self
    }

    pub fn add_header<'s>(
        mut self,
        key: impl Into<StrOrString<'s>>,
        value: impl Into<StrOrString<'s>>,
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

    pub fn add_headers_opt<'s>(
        mut self,
        headers: Option<
            impl Iterator<Item = (impl Into<StrOrString<'s>>, impl Into<StrOrString<'s>>)>,
        >,
    ) -> Self {
        if let Some(headers) = headers {
            for header in headers {
                self = self.add_header(header.0, header.1);
            }
        }

        self
    }

    pub fn set_cookie(mut self, cookie: impl Into<Cookie>) -> Self {
        let cookie_jar = match self.cookies.take() {
            Some(cookie_jar) => cookie_jar,
            None => CookieJar::new(),
        };

        self.cookies = Some(cookie_jar.set_cookie(cookie));

        self
    }

    pub fn set_cookies(mut self, cookies: impl IntoIterator<Item = impl Into<Cookie>>) -> Self {
        let mut cookie_jar = match self.cookies.take() {
            Some(cookies) => cookies,
            None => CookieJar::new(),
        };

        for cookie in cookies {
            cookie_jar = cookie_jar.set_cookie(cookie);
        }

        self.cookies = Some(cookie_jar);

        self
    }

    pub fn set_content_as_text(mut self, content: impl Into<String>) -> Self {
        let content = content.into();
        self.content = content.into_bytes();
        self.content_type = Some(WebContentType::Text);

        self
    }

    pub fn set_status_code(mut self, status_code: u16) -> Self {
        self.status_code = status_code;
        self
    }

    pub fn set_content(mut self, content: Vec<u8>) -> Self {
        self.content = content;
        self
    }

    pub fn into_ok_result(self, write_telemetry: bool) -> Result<HttpOkResult, HttpFailResult> {
        let output = self.build();
        Ok(HttpOkResult {
            write_telemetry,
            #[cfg(feature = "with-telemetry")]
            add_telemetry_tags: my_telemetry::TelemetryEventTagsBuilder::new(),
            output,
        })
    }

    pub fn build(self) -> HttpOutput {
        HttpOutput::Content {
            status_code: self.status_code,
            headers: self.headers,
            content_type: self.content_type,
            set_cookies: self.cookies,
            content: self.content,
        }
    }

    pub fn into_err(
        self,
        write_log: bool,
        write_telemetry: bool,
    ) -> Result<HttpOkResult, HttpFailResult> {
        let output = self.build();

        Err(HttpFailResult::new(output, write_log, write_telemetry))
    }

    pub fn into_http_fail_result(self, write_log: bool, write_telemetry: bool) -> HttpFailResult {
        let output = self.build();
        HttpFailResult::new(output, write_log, write_telemetry)
    }
}

impl AddHttpHeaders for HttpResultBuilder {
    fn add_header(&mut self, key: impl Into<String>, value: impl Into<String>) {
        if self.headers.is_none() {
            self.headers = Some(HashMap::new());
        }
        self.headers
            .as_mut()
            .unwrap()
            .insert(key.into(), value.into());
    }
}
