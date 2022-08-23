use std::collections::HashMap;

use crate::{HttpFailResult, WebContentType};
use hyper::{server::conn::Http, Body, Response};
use serde::Serialize;

pub enum HttpOutput {
    Empty,

    Content {
        headers: Option<HashMap<String, String>>,
        content_type: Option<WebContentType>,
        content: Vec<u8>,
    },

    Redirect {
        url: String,
        permanent: bool,
    },

    Raw(Response<Body>),
}

impl HttpOutput {
    pub fn into_ok_result(self, write_telemetry: bool) -> Result<HttpOkResult, HttpFailResult> {
        Ok(HttpOkResult {
            write_telemetry,
            output: self,
        })
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
            },
            HttpOutput::Content {
                headers: _,
                content_type,
                content,
            } => HttpFailResult {
                content_type: content_type.unwrap_or(WebContentType::Text),
                status_code: status_code,
                content,
                write_telemetry,
            },
            HttpOutput::Redirect {
                url: _,
                permanent: _,
            } => {
                panic!("Redirect can not be turned into Http Fail result")
            }
            HttpOutput::Raw(_) => {
                panic!("Raw response can not be turned into Http Fail result")
            }
        };

        Err(result)
    }

    pub fn as_text(text: String) -> Self {
        Self::Content {
            headers: None,
            content_type: Some(WebContentType::Text),
            content: text.into_bytes(),
        }
    }

    pub fn as_json<T: Serialize>(model: T) -> Self {
        let json = serde_json::to_vec(&model).unwrap();

        Self::Content {
            headers: None,
            content_type: Some(WebContentType::Json),
            content: json,
        }
    }

    pub fn as_redirect(src: &str, permanent: bool) -> Self {
        Self::Redirect {
            url: src.to_string(),
            permanent,
        }
    }

    pub fn as_usize(number: usize) -> Self {
        Self::Content {
            headers: None,
            content_type: Some(WebContentType::Text),
            content: number.to_string().into_bytes(),
        }
    }

    pub fn get_status_code(&self) -> u16 {
        match self {
            Self::Empty => 202,
            Self::Content {
                headers: _,
                content_type: _,
                content: _,
            } => 200,
            Self::Redirect { url: _, permanent } => {
                if *permanent {
                    301
                } else {
                    302
                }
            }

            HttpOutput::Raw(body) => body.status().as_u16(),
        }
    }
}

pub struct HttpOkResult {
    pub write_telemetry: bool,
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
            HttpOutput::Redirect { url, permanent: _ } => Response::builder()
                .status(status_code)
                .header("Location", url)
                .body(Body::empty())
                .unwrap(),
            HttpOutput::Empty => Response::builder()
                .status(status_code)
                .body(Body::empty())
                .unwrap(),

            HttpOutput::Raw(body) => body,
        };
    }
}
