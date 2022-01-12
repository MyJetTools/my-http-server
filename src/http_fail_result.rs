use crate::WebContentType;

#[derive(Debug)]
pub struct HttpFailResult {
    pub content_type: WebContentType,
    pub status_code: u16,
    pub content: Vec<u8>,
    pub write_telemetry: bool,
}

impl HttpFailResult {
    pub fn as_query_parameter_required(param_name: &str) -> Self {
        Self {
            content_type: WebContentType::Text,
            content: format!("Query parameter '{}' is required", param_name).into_bytes(),
            status_code: 400,
            write_telemetry: true,
        }
    }

    pub fn as_header_parameter_required(param_name: &str) -> Self {
        Self {
            content_type: WebContentType::Text,
            content: format!("Header '{}' is required", param_name).into_bytes(),
            status_code: 400,
            write_telemetry: true,
        }
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
}
