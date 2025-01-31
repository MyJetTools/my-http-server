use std::collections::HashMap;

use crate::{cookies::*, HttpFailResult, HttpOkResultBuilder, WebContentType};
use hyper::Response;
use my_hyper_utils::*;
use rust_extensions::StrOrString;
use serde::Serialize;

pub enum HttpOutput {
    Empty,

    Content {
        headers: Option<HashMap<String, String>>,
        content_type: Option<WebContentType>,
        set_cookies: Option<CookieJar>,
        content: Vec<u8>,
    },

    Redirect {
        headers: Option<HashMap<String, String>>,
        url: String,
        permanent: bool,
    },

    File {
        file_name: String,
        content: Vec<u8>,
    },

    Raw(MyHttpResponse),
}

impl HttpOutput {
    pub fn from_builder() -> HttpOkResultBuilder {
        HttpOkResultBuilder::new()
    }

    pub fn into_ok_result(self, write_telemetry: bool) -> Result<HttpOkResult, HttpFailResult> {
        Ok(HttpOkResult {
            write_telemetry,
            #[cfg(feature = "with-telemetry")]
            add_telemetry_tags: my_telemetry::TelemetryEventTagsBuilder::new(),
            output: self,
        })
    }

    #[cfg(feature = "with-telemetry")]
    pub fn into_ok_result_with_telemetry_tags(
        self,
        add_telemetry_tags: my_telemetry::TelemetryEventTagsBuilder,
    ) -> Result<HttpOkResult, HttpFailResult> {
        Ok(HttpOkResult {
            write_telemetry: true,
            add_telemetry_tags,
            output: self,
        })
    }

    pub fn get_content_size(&self) -> usize {
        match self {
            HttpOutput::Empty => 0,
            HttpOutput::Content { content, .. } => content.len(),
            HttpOutput::Redirect { url, .. } => url.len(),
            HttpOutput::File { content, .. } => content.len(),
            HttpOutput::Raw(_) => 0,
        }
    }

    pub fn get_content_type(&self) -> &str {
        match self {
            HttpOutput::Empty => "text/plain",
            HttpOutput::Content {
                headers: _,
                content_type,
                content: _,
                set_cookies: _,
            } => content_type
                .as_ref()
                .map(|ct| ct.as_str())
                .unwrap_or("text/plain"),
            HttpOutput::Redirect {
                url: _,
                headers: _,
                permanent: _,
            } => "text/plain",
            HttpOutput::File {
                file_name: _,
                content: _,
            } => "application/octet-stream",
            HttpOutput::Raw(_) => "text/plain",
        }
    }

    pub fn into_fail_result(
        self,
        status_code: u16,
        write_telemetry: bool,
    ) -> Result<HttpOkResult, HttpFailResult> {
        let result = match self {
            HttpOutput::Empty => HttpFailResult {
                content_type: WebContentType::Text,
                status_code: status_code,
                content: Vec::new(),
                write_telemetry,
                write_to_log: false,
                #[cfg(feature = "with-telemetry")]
                add_telemetry_tags: my_telemetry::TelemetryEventTagsBuilder::new(),
            },
            HttpOutput::Content {
                headers: _,
                content_type,
                content,
                set_cookies: _,
            } => HttpFailResult {
                content_type: content_type.unwrap_or(WebContentType::Text),
                status_code: status_code,
                content,
                write_telemetry,
                write_to_log: false,
                #[cfg(feature = "with-telemetry")]
                add_telemetry_tags: my_telemetry::TelemetryEventTagsBuilder::new(),
            },
            HttpOutput::Redirect {
                url: _,
                permanent: _,
                headers: _,
            } => {
                panic!("Redirect can not be turned into Http Fail result")
            }
            HttpOutput::Raw(_) => {
                panic!("Raw response can not be turned into Http Fail result")
            }
            HttpOutput::File { file_name, content } => HttpFailResult {
                content_type: if let Some(ct) =
                    WebContentType::detect_by_extension(file_name.as_str())
                {
                    ct
                } else {
                    WebContentType::Text
                },
                status_code,
                content,
                write_telemetry,
                write_to_log: false,
                #[cfg(feature = "with-telemetry")]
                add_telemetry_tags: my_telemetry::TelemetryEventTagsBuilder::new(),
            },
        };

        Err(result)
    }

    pub fn as_text<'s>(text: impl Into<StrOrString<'s>>) -> Self {
        let text = text.into().to_string();
        Self::Content {
            headers: None,
            content_type: Some(WebContentType::Text),
            content: text.into_bytes(),
            set_cookies: None,
        }
    }

    pub fn as_json<T: Serialize>(model: T) -> Self {
        let json = serde_json::to_vec(&model).unwrap();

        Self::Content {
            headers: None,
            content_type: Some(WebContentType::Json),
            content: json,
            set_cookies: None,
        }
    }

    pub fn as_yaml<T: Serialize>(model: T) -> Self {
        let yaml = serde_yaml::to_string(&model).unwrap();

        Self::Content {
            headers: None,
            content_type: Some(WebContentType::Yaml),
            content: yaml.into_bytes(),
            set_cookies: None,
        }
    }

    pub fn as_redirect(url: String, permanent: bool) -> Self {
        Self::Redirect {
            url,
            permanent,
            headers: None,
        }
    }

    pub fn as_usize(number: usize) -> Self {
        Self::Content {
            headers: None,
            content_type: Some(WebContentType::Text),
            content: number.to_string().into_bytes(),
            set_cookies: None,
        }
    }

    pub fn as_file(file_name: String, content: Vec<u8>) -> Self {
        Self::File { file_name, content }
    }

    pub fn get_status_code(&self) -> u16 {
        match self {
            Self::Empty => 204,
            Self::Content {
                headers: _,
                content_type: _,
                content: _,
                set_cookies: _,
            } => 200,
            Self::Redirect {
                url: _,
                permanent,
                headers: _,
            } => {
                if *permanent {
                    301
                } else {
                    302
                }
            }

            Self::File {
                file_name: _,
                content: _,
            } => 200,

            HttpOutput::Raw(body) => body.status().as_u16(),
        }
    }
}

pub struct HttpOkResult {
    pub write_telemetry: bool,
    #[cfg(feature = "with-telemetry")]
    pub add_telemetry_tags: my_telemetry::TelemetryEventTagsBuilder,
    pub output: HttpOutput,
}

impl HttpOkResult {
    pub fn get_status_code(&self) -> u16 {
        self.output.get_status_code()
    }
}

pub trait IntoHttpOkResult {
    fn into_http_ok_result(self) -> HttpOkResult;
}

impl Into<HttpOkResult> for String {
    fn into(self) -> HttpOkResult {
        HttpOkResult {
            write_telemetry: true,
            #[cfg(feature = "with-telemetry")]
            add_telemetry_tags: my_telemetry::TelemetryEventTagsBuilder::new(),
            output: HttpOutput::Content {
                headers: None,
                content_type: Some(WebContentType::Text),
                content: self.into_bytes(),
                set_cookies: None,
            },
        }
    }
}

impl Into<Result<HttpOkResult, HttpFailResult>> for HttpOkResult {
    fn into(self) -> Result<HttpOkResult, HttpFailResult> {
        Ok(self)
    }
}

impl Into<my_hyper_utils::MyHttpResponse> for HttpOkResult {
    fn into(self) -> MyHttpResponse {
        let status_code = self.get_status_code();

        return match self.output {
            HttpOutput::Content {
                headers,
                content_type,
                content,
                set_cookies,
            } => {
                let mut builder = Response::builder();

                if let Some(headers) = headers {
                    for (key, value) in headers {
                        builder = builder.header(key, value);
                    }
                }

                if let Some(content_type) = content_type {
                    builder = builder.header("content-type", content_type.as_str());
                }

                if let Some(cookies) = set_cookies {
                    for itm in cookies.get_cookies() {
                        builder = builder.header("Set-Cookie", itm.to_string());
                    }
                }

                (builder, content).to_my_http_response()
            }

            HttpOutput::Redirect {
                url,
                permanent: _,
                headers,
            } => {
                let mut builder = Response::builder()
                    .status(status_code)
                    .header("Location", url);

                if let Some(headers) = headers {
                    for (key, value) in headers {
                        builder = builder.header(key, value);
                    }
                }

                (builder, vec![]).to_my_http_response()
            }
            HttpOutput::Empty => {
                let builder = Response::builder().status(status_code);
                (builder, vec![]).to_my_http_response()
            }

            HttpOutput::Raw(body) => body,
            HttpOutput::File { file_name, content } => {
                let builder = Response::builder().header(
                    "content-disposition",
                    format!(
                        "attachment; filename=\"{file_name}\"; filename*=UTF-8''{file_name}",
                        file_name = file_name
                    ),
                );

                (builder, content).to_my_http_response()
            }
        };
    }
}
