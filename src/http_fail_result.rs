use crate::WebContentType;

#[derive(Debug)]
pub struct HttpFailResult {
    pub content_type: WebContentType,
    pub status_code: u16,
    pub content: Vec<u8>,
    pub write_telemetry: bool,
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
        }
    }

    pub fn as_not_found(text: String, write_telemetry: bool) -> Self {
        Self {
            content_type: WebContentType::Text,
            content: text.into_bytes(),
            status_code: 404,
            write_telemetry,
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
        }
    }

    pub fn invalid_value_to_parse(reason: String) -> Self {
        Self {
            content_type: WebContentType::Text,
            content: reason.into_bytes(),
            status_code: 400,
            write_telemetry: true,
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
        }
    }

    pub fn as_fatal_error(text: String) -> Self {
        Self {
            content_type: WebContentType::Text,
            content: text.into_bytes(),
            status_code: 500,
            write_telemetry: true,
        }
    }
}
