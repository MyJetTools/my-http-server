use std::collections::HashMap;

use crate::{cookies::*, HttpFailResult, HttpOkResult, HttpResultBuilder, WebContentType};

use http::{header::CONTENT_TYPE, Response};
use my_hyper_utils::*;
use rust_extensions::StrOrString;
use serde::Serialize;

const EMPTY_STATUS_CODE: u16 = 204;
const PERMANENT_REDIRECT_STATUS_CODE: u16 = 301;
const TEMPORARY_REDIRECT_STATUS_CODE: u16 = 302;

#[derive(Debug, Clone, Copy)]
pub enum RedirectType {
    Permanent,
    Temporary,
}

impl RedirectType {
    pub fn get_status_code(&self) -> u16 {
        match self {
            RedirectType::Permanent => PERMANENT_REDIRECT_STATUS_CODE,
            RedirectType::Temporary => TEMPORARY_REDIRECT_STATUS_CODE,
        }
    }
}

#[derive(Debug)]
pub enum HttpOutput {
    Empty,

    Content {
        status_code: u16,
        headers: Option<HashMap<String, String>>,
        content_type: Option<WebContentType>,
        set_cookies: Option<CookieJar>,
        content: Vec<u8>,
    },

    Redirect {
        headers: Option<HashMap<String, String>>,
        url: String,
        redirect_type: RedirectType,
    },

    File {
        file_name: String,
        content: Vec<u8>,
    },

    Raw(MyHttpResponse),
}

impl HttpOutput {
    pub fn from_builder() -> HttpResultBuilder {
        HttpResultBuilder::new()
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

    pub fn get_content_type_as_str(&self) -> Option<&str> {
        match self {
            HttpOutput::Empty => "text/plain".into(),
            HttpOutput::Content { content_type, .. } => content_type.as_ref().map(|ct| ct.as_str()),

            HttpOutput::Redirect { .. } => None,
            HttpOutput::File {
                file_name: _,
                content: _,
            } => Some("application/octet-stream"),

            HttpOutput::Raw(data) => data
                .headers()
                .get(CONTENT_TYPE)
                .map(|itm| itm.to_str().ok().unwrap_or_default()),
        }
    }

    pub fn into_err<TResult>(
        self,
        write_log: bool,
        write_telemetry: bool,
    ) -> Result<TResult, HttpFailResult> {
        Err(self.into_http_fail_result(write_log, write_telemetry))
    }

    pub fn into_http_fail_result(self, write_log: bool, write_telemetry: bool) -> HttpFailResult {
        HttpFailResult::new(self, write_log, write_telemetry)
    }

    pub fn as_text<'s>(text: impl Into<StrOrString<'s>>) -> HttpResultBuilder {
        let text = text.into().to_string();

        HttpResultBuilder {
            status_code: 200,
            headers: None,
            content_type: Some(WebContentType::Text),
            cookies: Default::default(),
            content: text.into_bytes(),
        }
    }

    pub fn as_html<'s>(text: impl Into<StrOrString<'s>>) -> HttpResultBuilder {
        let text = text.into().to_string();

        HttpResultBuilder {
            status_code: 200,
            headers: None,
            content_type: Some(WebContentType::Html),
            cookies: Default::default(),
            content: text.into_bytes(),
        }
    }

    pub fn as_json<T: Serialize>(model: T) -> HttpResultBuilder {
        let json = serde_json::to_vec(&model).unwrap();

        HttpResultBuilder {
            status_code: 200,
            headers: None,
            content_type: Some(WebContentType::Json),
            cookies: Default::default(),
            content: json,
        }
    }

    pub fn as_yaml<T: Serialize>(model: T) -> HttpResultBuilder {
        let yaml = serde_yaml::to_string(&model).unwrap();

        HttpResultBuilder {
            status_code: 200,
            headers: None,
            content_type: Some(WebContentType::Yaml),
            cookies: Default::default(),
            content: yaml.into_bytes(),
        }
    }

    pub fn as_redirect(url: String, permanent: bool) -> Self {
        Self::Redirect {
            url,
            redirect_type: if permanent {
                RedirectType::Permanent
            } else {
                RedirectType::Temporary
            },
            headers: None,
        }
    }

    pub fn as_usize(number: usize) -> Self {
        Self::Content {
            status_code: 200,
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
            Self::Content { status_code, .. } => *status_code,
            Self::Redirect {
                url: _,
                redirect_type,
                headers: _,
            } => redirect_type.get_status_code(),

            Self::File {
                file_name: _,
                content: _,
            } => 200,

            HttpOutput::Raw(body) => body.status().as_u16(),
        }
    }

    pub(crate) fn get_text_as_error<'s>(&'s self) -> StrOrString<'s> {
        match self {
            HttpOutput::Empty => "Empty response".into(),
            HttpOutput::Content { content, .. } => {
                let result = if content.len() > 256 {
                    std::str::from_utf8(&content[..256])
                } else {
                    std::str::from_utf8(content)
                };

                match result {
                    Ok(text) => text.into(),
                    Err(_) => "Can not get Error message. Content is not UTF8".into(),
                }
            }
            HttpOutput::Redirect {
                headers: _,
                url,
                redirect_type,
            } => format!("Redirect to '{}' with type '{:?}'", url, redirect_type).into(),
            HttpOutput::File { file_name, content } => {
                format!("File '{}' with size {} bytes", file_name, content.len()).into()
            }
            HttpOutput::Raw(response) => format!(
                "Raw response with status code {} and headers: {:?}",
                response.status().as_u16(),
                response.headers()
            )
            .into(),
        }
    }

    pub fn as_not_found(text: impl Into<String>) -> HttpResultBuilder {
        HttpResultBuilder {
            status_code: 404,
            headers: Default::default(),
            content_type: WebContentType::Text.into(),
            cookies: Default::default(),
            content: text.into().into_bytes(),
        }
    }

    pub fn as_unauthorized(text: Option<&str>) -> HttpResultBuilder {
        HttpResultBuilder {
            status_code: 401,
            headers: None,
            content_type: WebContentType::Text.into(),
            cookies: None,
            content: if let Some(text) = text {
                format!("Unauthorized request: {}", text).into_bytes()
            } else {
                format!("Unauthorized request").into_bytes()
            },
        }
    }

    pub fn as_validation_error(text: impl Into<StrOrString<'static>>) -> HttpResultBuilder {
        HttpResultBuilder {
            status_code: 400,
            headers: None,
            content_type: WebContentType::Text.into(),
            cookies: None,
            content: format!("Validation error: {}", text.into().as_str()).into_bytes(),
        }
    }

    pub fn as_forbidden(text: Option<impl Into<String>>) -> HttpResultBuilder {
        HttpResultBuilder {
            status_code: 403,
            headers: None,
            content_type: WebContentType::Text.into(),
            cookies: None,
            content: if let Some(text) = text {
                text.into().into_bytes()
            } else {
                format!("Forbidden").into_bytes()
            },
        }
    }

    pub fn invalid_value_to_parse(reason: impl Into<String>) -> HttpResultBuilder {
        HttpResultBuilder {
            status_code: 400,
            headers: None,
            content_type: WebContentType::Text.into(),
            cookies: None,
            content: reason.into().into_bytes(),
        }
    }

    pub fn required_parameter_is_missing(
        param_name: &str,
        where_is_parameter: &str,
    ) -> HttpResultBuilder {
        HttpResultBuilder {
            status_code: 400,
            headers: None,
            content_type: WebContentType::Text.into(),
            cookies: None,
            content: format!(
                "Required parameter [{param_name}] is missing in {where_is_parameter}"
            )
            .into_bytes(),
        }
    }

    pub fn as_fatal_error(text: impl Into<String>) -> HttpResultBuilder {
        HttpResultBuilder {
            status_code: 500,
            headers: None,
            content_type: WebContentType::Text.into(),
            cookies: None,
            content: text.into().into_bytes(),
        }
    }

    pub fn as_not_supported_content_type(text: impl Into<String>) -> HttpResultBuilder {
        HttpResultBuilder {
            status_code: 415,
            headers: None,
            content_type: WebContentType::Text.into(),
            cookies: None,
            content: text.into().into_bytes(),
        }
    }
}

impl Into<my_hyper_utils::MyHttpResponse> for HttpOutput {
    fn into(self) -> MyHttpResponse {
        return match self {
            HttpOutput::Content {
                status_code,
                headers,
                content_type,
                content,
                set_cookies,
            } => {
                let mut builder = Response::builder().status(status_code);

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
                redirect_type,
                headers,
            } => {
                let mut builder = Response::builder()
                    .status(redirect_type.get_status_code())
                    .header("Location", url);

                if let Some(headers) = headers {
                    for (key, value) in headers {
                        builder = builder.header(key, value);
                    }
                }

                (builder, vec![]).to_my_http_response()
            }
            HttpOutput::Empty => {
                let builder = Response::builder().status(EMPTY_STATUS_CODE);
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
