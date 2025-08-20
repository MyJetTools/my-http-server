use rust_extensions::StrOrString;

use crate::{HttpOkResult, HttpOutput, WebContentType};

#[derive(Debug)]
pub struct HttpFailResult {
    pub write_to_log: bool,
    pub write_telemetry: bool,
    #[cfg(feature = "with-telemetry")]
    pub add_telemetry_tags: my_telemetry::TelemetryEventTagsBuilder,
    pub output: HttpOutput,
}

impl HttpFailResult {
    pub fn new(output: HttpOutput, write_to_log: bool, write_telemetry: bool) -> Self {
        Self {
            write_to_log,
            write_telemetry,
            output,
            #[cfg(feature = "with-telemetry")]
            add_telemetry_tags: Default::default(),
        }
    }
}

impl From<url_utils::url_encoded_data_reader::ReadingEncodedDataError> for HttpFailResult {
    fn from(src: url_utils::url_encoded_data_reader::ReadingEncodedDataError) -> Self {
        let output = HttpOutput::Content {
            status_code: 400,
            headers: Default::default(),
            content_type: Some(WebContentType::Text),
            set_cookies: Default::default(),
            content: format!("Reading encoded parameter failed. Err: '{:?}'", src).into_bytes(),
        };

        HttpFailResult::new(output, true, false)
    }
}

impl Into<Result<HttpOkResult, HttpFailResult>> for HttpFailResult {
    fn into(self) -> Result<HttpOkResult, HttpFailResult> {
        Result::Err(self)
    }
}

impl HttpFailResult {
    pub fn into_err<T>(self) -> Result<T, HttpFailResult> {
        Result::Err(self)
    }

    pub fn as_path_parameter_required(param_name: &str) -> Self {
        let output = HttpOutput::Content {
            status_code: 400,
            headers: None,
            content_type: WebContentType::Text.into(),
            set_cookies: None,
            content: format!("Path parameter '{}' is required", param_name).into_bytes(),
        };

        Self::new(output, false, true)
    }

    pub fn as_not_found(text: impl Into<String>, write_telemetry: bool) -> Self {
        let output = HttpOutput::Content {
            status_code: 404,
            headers: None,
            content_type: WebContentType::Text.into(),
            set_cookies: None,
            content: text.into().into_bytes(),
        };

        Self::new(output, false, write_telemetry)
    }

    pub fn as_unauthorized(text: Option<&str>) -> Self {
        let output = HttpOutput::Content {
            status_code: 401,
            headers: None,
            content_type: WebContentType::Text.into(),
            set_cookies: None,
            content: if let Some(text) = text {
                format!("Unauthorized request: {}", text).into_bytes()
            } else {
                format!("Unauthorized request").into_bytes()
            },
        };

        Self::new(output, false, false)
    }

    pub fn as_validation_error(text: impl Into<StrOrString<'static>>) -> Self {
        let output = HttpOutput::Content {
            status_code: 400,
            headers: None,
            content_type: WebContentType::Text.into(),
            set_cookies: None,
            content: format!("Validation error: {}", text.into().as_str()).into_bytes(),
        };

        Self::new(output, false, true)
    }

    pub fn as_forbidden(text: Option<impl Into<String>>) -> Self {
        let output = HttpOutput::Content {
            status_code: 403,
            headers: None,
            content_type: WebContentType::Text.into(),
            set_cookies: None,
            content: if let Some(text) = text {
                text.into().into_bytes()
            } else {
                format!("Forbidden").into_bytes()
            },
        };

        Self::new(output, false, true)
    }

    pub fn invalid_value_to_parse(reason: impl Into<String>) -> Self {
        let output = HttpOutput::Content {
            status_code: 400,
            headers: None,
            content_type: WebContentType::Text.into(),
            set_cookies: None,
            content: reason.into().into_bytes(),
        };

        Self::new(output, true, true)
    }

    pub fn required_parameter_is_missing(param_name: &str, where_is_parameter: &str) -> Self {
        let output = HttpOutput::Content {
            status_code: 400,
            headers: None,
            content_type: WebContentType::Text.into(),
            set_cookies: None,
            content: format!(
                "Required parameter [{param_name}] is missing in {where_is_parameter}"
            )
            .into_bytes(),
        };

        Self::new(output, false, true)
    }

    pub fn as_fatal_error(text: impl Into<String>) -> Self {
        let output = HttpOutput::Content {
            status_code: 500,
            headers: None,
            content_type: WebContentType::Text.into(),
            set_cookies: None,
            content: text.into().into_bytes(),
        };

        Self::new(output, true, true)
    }

    pub fn as_not_supported_content_type(text: impl Into<String>) -> Self {
        let output = HttpOutput::Content {
            status_code: 415,
            headers: None,
            content_type: WebContentType::Text.into(),
            set_cookies: None,
            content: text.into().into_bytes(),
        };

        Self::new(output, true, true)
    }
}

impl Into<HttpFailResult> for HttpOutput {
    fn into(self) -> HttpFailResult {
        HttpFailResult {
            write_telemetry: false,
            write_to_log: true,
            #[cfg(feature = "with-telemetry")]
            add_telemetry_tags: my_telemetry::TelemetryEventTagsBuilder::new(),
            output: self,
        }
    }
}

impl From<(HttpOutput, bool)> for HttpFailResult {
    fn from((output, write_to_log_and_telemetry): (HttpOutput, bool)) -> Self {
        HttpFailResult {
            write_to_log: write_to_log_and_telemetry,
            write_telemetry: write_to_log_and_telemetry,
            #[cfg(feature = "with-telemetry")]
            add_telemetry_tags: my_telemetry::TelemetryEventTagsBuilder::new(),
            output,
        }
    }
}

impl From<(u16, String)> for HttpFailResult {
    fn from(value: (u16, String)) -> Self {
        let output = HttpOutput::Content {
            status_code: value.0,
            headers: Default::default(),
            content_type: WebContentType::Text.into(),
            set_cookies: Default::default(),
            content: value.1.into_bytes(),
        };

        Self::new(output, false, true)
    }
}

impl From<(u16, &'static str)> for HttpFailResult {
    fn from(value: (u16, &'static str)) -> Self {
        let output = HttpOutput::Content {
            status_code: value.0,
            headers: Default::default(),
            content_type: WebContentType::Text.into(),
            set_cookies: Default::default(),
            content: value.1.as_bytes().to_vec(),
        };

        Self::new(output, false, true)
    }
}
