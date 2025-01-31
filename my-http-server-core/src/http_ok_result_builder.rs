use std::collections::HashMap;

use rust_extensions::StrOrString;

use crate::{cookies::*, HttpOutput, WebContentType};

pub struct HttpOkResultBuilder {
    headers: Option<HashMap<String, String>>,
    content_type: Option<WebContentType>,
    cookies: CookieJar,
}

impl HttpOkResultBuilder {
    pub fn new() -> Self {
        Self {
            headers: None,
            content_type: None,
            cookies: CookieJar::new(),
        }
    }

    pub fn set_content_type(&mut self, content_type: WebContentType) -> &mut Self {
        self.content_type = Some(content_type);
        self
    }

    pub fn add_header(
        &mut self,
        key: impl Into<StrOrString<'static>>,
        value: impl Into<StrOrString<'static>>,
    ) -> &mut Self {
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

    pub fn set_cookie<'s>(&mut self, cookie: impl Into<Cookie>) -> &mut Self {
        self.cookies.set_cookie(cookie);

        self
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
