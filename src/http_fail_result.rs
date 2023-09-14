use crate::WebContentType;

#[derive(Debug, Clone)]
pub struct HttpFailResult {
    pub content_type: WebContentType,
    pub status_code: u16,
    pub content: Vec<u8>,
    pub write_telemetry: bool,
    pub write_to_log: bool,
    #[cfg(feature = "my-telemetry")]
    pub add_telemetry_tags: my_telemetry::TelemetryEventTagsBuilder,
}

impl From<url_utils::url_encoded_data_reader::ReadingEncodedDataError> for HttpFailResult {
    fn from(src: url_utils::url_encoded_data_reader::ReadingEncodedDataError) -> Self {
        Self {
            content_type: WebContentType::Text,
            content: format!("Reading encoded parameter failed. Err: '{:?}'", src).into_bytes(),
            status_code: 400,
            write_telemetry: true,
            write_to_log: true,
            #[cfg(feature = "my-telemetry")]
            add_telemetry_tags: my_telemetry::TelemetryEventTagsBuilder::new(),
        }
    }
}

impl HttpFailResult {
    pub fn into_err<T>(self) -> Result<T, HttpFailResult> {
        Result::Err(self)
    }

    pub fn as_path_parameter_required(param_name: &str) -> Self {
        Self {
            content_type: WebContentType::Text,
            content: format!("Path parameter '{}' is required", param_name).into_bytes(),
            status_code: 400,
            write_telemetry: true,
            write_to_log: false,
            #[cfg(feature = "my-telemetry")]
            add_telemetry_tags: my_telemetry::TelemetryEventTagsBuilder::new(),
        }
    }

    pub fn as_not_found(text: String, write_telemetry: bool) -> Self {
        Self {
            content_type: WebContentType::Text,
            content: text.into_bytes(),
            status_code: 404,
            write_telemetry,
            write_to_log: false,
            #[cfg(feature = "my-telemetry")]
            add_telemetry_tags: my_telemetry::TelemetryEventTagsBuilder::new(),
        }
    }

    pub fn as_unauthorized(text: Option<String>) -> Self {
        Self {
            content_type: WebContentType::Text,
            content: if let Some(text) = text {
                format!("Unauthorized request: {}", text).into_bytes()
            } else {
                format!("Unauthorized request").into_bytes()
            },
            status_code: 401,
            write_telemetry: true,
            write_to_log: false,
            #[cfg(feature = "my-telemetry")]
            add_telemetry_tags: my_telemetry::TelemetryEventTagsBuilder::new(),
        }
    }

    pub fn as_validation_error(text: String) -> Self {
        Self {
            content_type: WebContentType::Text,
            content: format!("Validation error: {}", text).into_bytes(),
            status_code: 401,
            write_telemetry: true,
            write_to_log: false,
            #[cfg(feature = "my-telemetry")]
            add_telemetry_tags: my_telemetry::TelemetryEventTagsBuilder::new(),
        }
    }

    pub fn as_forbidden(text: Option<String>) -> Self {
        Self {
            content_type: WebContentType::Text,
            content: if let Some(text) = text {
                text.into_bytes()
            } else {
                format!("Forbidden").into_bytes()
            },
            status_code: 403,
            write_telemetry: true,
            write_to_log: false,
            #[cfg(feature = "my-telemetry")]
            add_telemetry_tags: my_telemetry::TelemetryEventTagsBuilder::new(),
        }
    }

    pub fn invalid_value_to_parse(reason: String) -> Self {
        Self {
            content_type: WebContentType::Text,
            content: reason.into_bytes(),
            status_code: 400,
            write_telemetry: true,
            write_to_log: true,
            #[cfg(feature = "my-telemetry")]
            add_telemetry_tags: my_telemetry::TelemetryEventTagsBuilder::new(),
        }
    }

    pub fn required_parameter_is_missing(param_name: &str, where_is_parameter: &str) -> Self {
        Self {
            content_type: WebContentType::Text,
            content: format!(
                "Required parameter [{param_name}] is missing in {where_is_parameter}"
            )
            .into_bytes(),
            status_code: 400,
            write_telemetry: true,
            write_to_log: false,
            #[cfg(feature = "my-telemetry")]
            add_telemetry_tags: my_telemetry::TelemetryEventTagsBuilder::new(),
        }
    }

    pub fn as_fatal_error(text: String) -> Self {
        Self {
            content_type: WebContentType::Text,
            content: text.into_bytes(),
            status_code: 500,
            write_telemetry: true,
            write_to_log: true,
            #[cfg(feature = "my-telemetry")]
            add_telemetry_tags: my_telemetry::TelemetryEventTagsBuilder::new(),
        }
    }

    pub fn as_not_supported_content_type(text: String) -> Self {
        Self {
            content_type: WebContentType::Text,
            content: text.into_bytes(),
            status_code: 415,
            write_telemetry: true,
            write_to_log: true,
            #[cfg(feature = "my-telemetry")]
            add_telemetry_tags: my_telemetry::TelemetryEventTagsBuilder::new(),
        }
    }
}
