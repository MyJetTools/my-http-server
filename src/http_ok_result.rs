use std::collections::HashMap;

use crate::{HttpFailResult, WebContentType};
use hyper::{Body, Response};
use serde::Serialize;

#[derive(Clone)]
pub enum HttpContent {
    Ok,

    Empty,

    Content {
        headers: Option<HashMap<String, String>>,
        content_type: Option<WebContentType>,
        content: Vec<u8>,
    },
    Text {
        text: String,
    },

    Redirect {
        url: String,
    },
}

#[derive(Clone)]
pub struct HttpOkResult {
    pub write_telemetry: bool,
    pub content: HttpContent,
}

impl HttpOkResult {
    pub fn create_json_response<T: Serialize>(model: T) -> HttpOkResult {
        let json = serde_json::to_vec(&model).unwrap();

        HttpOkResult {
            write_telemetry: true,
            content: HttpContent::Content {
                headers: None,
                content_type: Some(WebContentType::Json),
                content: json,
            },
        }
    }

    pub fn create_as_usize(number: usize) -> HttpOkResult {
        HttpOkResult {
            write_telemetry: true,
            content: HttpContent::Content {
                headers: None,
                content_type: Some(WebContentType::Text),
                content: number.to_string().into_bytes(),
            },
        }
    }

    pub fn redirect(src: &str) -> HttpOkResult {
        HttpOkResult {
            write_telemetry: true,
            content: HttpContent::Redirect {
                url: src.to_string(),
            },
        }
    }

    pub fn get_status_code(&self) -> u16 {
        match &self.content {
            HttpContent::Ok => 200,
            HttpContent::Empty => 202,
            HttpContent::Content {
                headers: _,
                content_type: _,
                content: _,
            } => 200,
            HttpContent::Text { text: _ } => 200,
            HttpContent::Redirect { url: _ } => 308,
        }
    }
}

pub trait IntoHttpOkResult {
    fn into_http_ok_result(self) -> HttpOkResult;
}

impl Into<HttpOkResult> for String {
    fn into(self) -> HttpOkResult {
        HttpOkResult {
            write_telemetry: true,
            content: HttpContent::Content {
                headers: None,
                content_type: Some(WebContentType::Text),
                content: self.into_bytes(),
            },
        }
    }
}

impl Into<Result<HttpOkResult, HttpFailResult>> for HttpOkResult {
    fn into(self) -> Result<HttpOkResult, HttpFailResult> {
        Ok(self)
    }
}

impl Into<Response<Body>> for HttpOkResult {
    fn into(self) -> Response<Body> {
        let status_code = self.get_status_code();

        return match self.content {
            HttpContent::Ok => Response::builder()
                .header("Content-Type", WebContentType::Text.as_str())
                .status(status_code)
                .body(Body::from("OK"))
                .unwrap(),
            HttpContent::Content {
                headers,
                content_type,
                content,
            } => match content_type {
                Some(content_type) => {
                    let mut builder =
                        Response::builder().header("Content-Type", content_type.as_str());

                    if let Some(headers) = headers {
                        for (key, value) in headers {
                            builder = builder.header(key, value);
                        }
                    }

                    builder
                        .status(status_code)
                        .body(Body::from(content))
                        .unwrap()
                }
                None => Response::builder()
                    .status(status_code)
                    .body(Body::from(content))
                    .unwrap(),
            },
            HttpContent::Text { text } => Response::builder()
                .header("Content-Type", WebContentType::Text.as_str())
                .status(status_code)
                .body(Body::from(text))
                .unwrap(),
            HttpContent::Redirect { url } => Response::builder()
                .status(status_code)
                .header("Location", url)
                .body(Body::empty())
                .unwrap(),
            HttpContent::Empty => Response::builder()
                .status(status_code)
                .body(Body::empty())
                .unwrap(),
        };
    }
}
