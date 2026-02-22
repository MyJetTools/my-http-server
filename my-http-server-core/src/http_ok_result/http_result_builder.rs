use rust_extensions::StrOrString;

use crate::{
    cookies::*, AddHttpHeaders, HttpFailResult, HttpOutput, HttpResponseHeaders, WebContentType,
};

use super::HttpOkResult;

const EMPTY_STATUS_CODE: u16 = 204;

pub struct HttpResultBuilder {
    pub(crate) output: HttpOutput,
    /*
    pub(crate) status_code: u16,
    pub(crate) headers: Option<HashMap<String, String>>,
    pub(crate) content_type: Option<WebContentType>,
    pub(crate) cookies: Option<CookieJar>,
    pub(crate) content: Vec<u8>,
     */
}

impl HttpResultBuilder {
    pub fn new() -> Self {
        Self {
            output: HttpOutput::Empty, /*
                                       status_code: 204,
                                       headers: None,
                                       content_type: None,
                                       cookies: Default::default(),
                                       content: Default::default(),
                                        */
        }
    }

    pub fn set_content_type(mut self, new_content_type: WebContentType) -> Self {
        match &mut self.output {
            HttpOutput::Empty => {
                self.output = HttpOutput::Content {
                    status_code: EMPTY_STATUS_CODE,
                    headers: HttpResponseHeaders::new(Some(new_content_type)),
                    content: Default::default(),
                }
            }
            HttpOutput::Content { headers, .. } => {
                headers.set_content_type(new_content_type);
            }
            HttpOutput::Redirect { .. } => {
                panic!("Can not set content type at redirect output");
            }
            HttpOutput::File { headers, .. } => {
                headers.set_content_type(new_content_type);
            }
            HttpOutput::Raw(_) => {
                panic!("Can not set content type to raw response");
            }
        }

        self
    }

    pub fn set_content_type_opt(self, content_type: Option<WebContentType>) -> Self {
        if let Some(content_type) = content_type {
            return self.set_content_type(content_type);
        }
        self
    }

    pub fn add_header(
        mut self,
        key: impl Into<StrOrString<'static>>,
        value: impl Into<String>,
    ) -> Self {
        match &mut self.output {
            HttpOutput::Empty => {
                self.output = HttpOutput::Content {
                    status_code: EMPTY_STATUS_CODE,
                    headers: HttpResponseHeaders::new_with_header(key.into(), value.into()),
                    content: Default::default(),
                }
            }
            HttpOutput::Content { headers, .. } => {
                headers.add_header(key.into(), value.into());
            }
            HttpOutput::Redirect { headers, .. } => {
                headers.add_header(key.into(), value.into());
            }
            HttpOutput::File { headers, .. } => {
                headers.add_header(key.into(), value.into());
            }
            HttpOutput::Raw(_) => {
                panic!("Can not set header to raw output")
            }
        }

        self
    }

    pub fn add_header_if_some(
        self,
        key: impl Into<StrOrString<'static>>,
        value: Option<impl Into<String>>,
    ) -> Self {
        if let Some(value) = value {
            return self.add_header(key, value);
        }

        self
    }

    pub fn add_headers(
        mut self,
        headers: impl Iterator<Item = (impl Into<StrOrString<'static>>, impl Into<String>)>,
    ) -> Self {
        for header in headers {
            self = self.add_header(header.0, header.1);
        }

        self
    }

    pub fn add_headers_opt(
        mut self,
        headers: Option<impl Iterator<Item = (impl Into<StrOrString<'static>>, impl Into<String>)>>,
    ) -> Self {
        if let Some(headers) = headers {
            for header in headers {
                self = self.add_header(header.0, header.1);
            }
        }

        self
    }

    pub fn set_cookie(mut self, cookie: impl Into<Cookie>) -> Self {
        match &mut self.output {
            HttpOutput::Empty => {
                self.output = HttpOutput::Content {
                    status_code: EMPTY_STATUS_CODE,
                    headers: HttpResponseHeaders::new_with_cookie(cookie.into()),
                    content: Default::default(),
                }
            }
            HttpOutput::Content { headers, .. } => {
                headers.set_cookie(cookie.into());
            }
            HttpOutput::Redirect { headers, .. } => {
                headers.set_cookie(cookie.into());
            }
            HttpOutput::File { headers, .. } => {
                headers.set_cookie(cookie.into());
            }
            HttpOutput::Raw(_) => {
                panic!("Can not set cookie to raw response");
            }
        }

        self
    }

    pub fn set_cookies(mut self, cookies: impl IntoIterator<Item = impl Into<Cookie>>) -> Self {
        match &mut self.output {
            HttpOutput::Empty => {
                self.output = HttpOutput::Content {
                    status_code: EMPTY_STATUS_CODE,
                    headers: HttpResponseHeaders::new_with_cookies(
                        CookieJar::new().set_cookies(cookies),
                    ),
                    content: Default::default(),
                }
            }
            HttpOutput::Content { headers, .. } => {
                headers.set_cookies(cookies);
            }
            HttpOutput::Redirect { headers, .. } => {
                headers.set_cookies(cookies);
            }
            HttpOutput::File { headers, .. } => {
                headers.set_cookies(cookies);
            }
            HttpOutput::Raw(_) => {
                panic!("Can not set cookies to raw response");
            }
        }

        self
    }

    pub fn set_content_as_text(mut self, content: impl Into<String>) -> Self {
        const CONTENT_TYPE_TO_SET: WebContentType = WebContentType::Text;
        let text_content = content.into();

        match &mut self.output {
            HttpOutput::Empty => {
                self.output = HttpOutput::Content {
                    status_code: 200,
                    headers: HttpResponseHeaders::new(CONTENT_TYPE_TO_SET.into()),
                    content: text_content.into_bytes(),
                }
            }
            HttpOutput::Content {
                headers,
                content,
                status_code,
            } => {
                if *status_code == EMPTY_STATUS_CODE {
                    *status_code = 200;
                }
                headers.set_content_type(CONTENT_TYPE_TO_SET);
                *content = text_content.into_bytes();
            }
            HttpOutput::Redirect { .. } => {
                panic!("Can not set content to redirect response");
            }
            HttpOutput::File { content, .. } => {
                *content = text_content.into_bytes();
            }
            HttpOutput::Raw(_) => {
                panic!("Can not set content to raw response");
            }
        }

        self
    }

    pub fn set_status_code(mut self, status_code_to_set: u16) -> Self {
        match &mut self.output {
            HttpOutput::Empty => {
                self.output = HttpOutput::Content {
                    status_code: status_code_to_set,
                    headers: Default::default(),
                    content: Default::default(),
                }
            }
            HttpOutput::Content { status_code, .. } => {
                *status_code = status_code_to_set;
            }
            HttpOutput::Redirect { .. } => {
                panic!("Can not set status_code to redirect response");
            }
            HttpOutput::File { .. } => {
                panic!("Can not set status_code to File output response");
            }
            HttpOutput::Raw(_) => {
                panic!("Can not set content to raw response");
            }
        }
        self
    }

    pub fn set_content(mut self, content_to_set: Vec<u8>) -> Self {
        match &mut self.output {
            HttpOutput::Empty => {
                self.output = HttpOutput::Content {
                    status_code: 200,
                    headers: Default::default(),
                    content: content_to_set,
                }
            }
            HttpOutput::Content {
                content,
                status_code,
                ..
            } => {
                if *status_code == EMPTY_STATUS_CODE {
                    *status_code = 200;
                }
                *content = content_to_set;
            }
            HttpOutput::Redirect { .. } => {
                panic!("Can not set status_code to redirect response");
            }
            HttpOutput::File { content, .. } => {
                *content = content_to_set;
            }
            HttpOutput::Raw(_) => {
                panic!("Can not set content to raw response");
            }
        }
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
        self.output
    }

    pub fn into_err<TResult>(
        self,
        write_log: bool,
        write_telemetry: bool,
    ) -> Result<TResult, HttpFailResult> {
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
        match &mut self.output {
            HttpOutput::Empty => {
                self.output = HttpOutput::Content {
                    status_code: EMPTY_STATUS_CODE,
                    headers: HttpResponseHeaders::new_with_header(key.into().into(), value.into()),
                    content: Default::default(),
                }
            }
            HttpOutput::Content { headers, .. } => {
                headers.add_header(key.into().into(), value.into());
            }
            HttpOutput::Redirect { headers, .. } => {
                headers.add_header(key.into().into(), value.into());
            }
            HttpOutput::File { headers, .. } => {
                headers.add_header(key.into().into(), value.into());
            }
            HttpOutput::Raw(_) => {
                panic!("Can not set header to raw output")
            }
        }
    }
}
