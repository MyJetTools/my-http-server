use std::collections::HashMap;

use crate::{HttpFailResult, WebContentType};
use hyper::{Body, Response};
use serde::Serialize;

#[derive(Clone)]
pub enum HttpOutput {
    Empty,

    Content {
        headers: Option<HashMap<String, String>>,
        content_type: Option<WebContentType>,
        content: Vec<u8>,
    },

    Redirect {
        url: String,
    },
}

impl HttpOutput {
    pub fn into_ok_result(self, write_telemetry: bool) -> HttpOkResult {
        HttpOkResult {
            write_telemetry: true,
            output: self,
        }
    }
}

#[derive(Clone)]
pub struct HttpOkResult {
    pub write_telemetry: bool,
    pub output: HttpOutput,
}

impl HttpOkResult {
    pub fn create_json_response<T: Serialize>(model: T) -> HttpOkResult {
        let json = serde_json::to_vec(&model).unwrap();

        HttpOkResult {
            write_telemetry: true,
            output: HttpOutput::Content {
                headers: None,
                content_type: Some(WebContentType::Json),
                content: json,
            },
        }
    }

    pub fn create_text_response(text: String) -> HttpOkResult {
        HttpOkResult {
            write_telemetry: true,
            output: HttpOutput::Content {
                headers: None,
                content_type: Some(WebContentType::Text),
                content: text.into_bytes(),
            },
        }
    }

    pub fn create_as_usize(number: usize) -> HttpOkResult {
        HttpOkResult {
            write_telemetry: true,
            output: HttpOutput::Content {
                headers: None,
                content_type: Some(WebContentType::Text),
                content: number.to_string().into_bytes(),
            },
        }
    }

    pub fn redirect(src: &str) -> HttpOkResult {
        HttpOkResult {
            write_telemetry: true,
            output: HttpOutput::Redirect {
                url: src.to_string(),
            },
        }
    }

    pub fn get_status_code(&self) -> u16 {
        match &self.output {
            HttpOutput::Empty => 202,
            HttpOutput::Content {
                headers: _,
                content_type: _,
                content: _,
            } => 200,
            HttpOutput::Redirect { url: _ } => 308,
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
            output: HttpOutput::Content {
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

        return match self.output {
            HttpOutput::Content {
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
            HttpOutput::Redirect { url } => Response::builder()
                .status(status_code)
                .header("Location", url)
                .body(Body::empty())
                .unwrap(),
            HttpOutput::Empty => Response::builder()
                .status(status_code)
                .body(Body::empty())
                .unwrap(),
        };
    }
}
