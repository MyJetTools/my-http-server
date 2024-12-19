use hyper::{HeaderMap, Uri};

use crate::{HttpFailResult, HttpRequestBody, HttpRequestHeaders};

use super::ContentEncoding;

pub enum RequestData {
    Incoming(Option<hyper::Request<hyper::body::Incoming>>),
    AsBody {
        uri: Uri,
        headers: HeaderMap,
        body: Option<HttpRequestBody>,
    },
    Taken,
}

impl RequestData {
    pub async fn convert_to_body_if_requires(
        &mut self,
    ) -> Result<Option<&HttpRequestBody>, HttpFailResult> {
        let (uri, headers, bytes) = match self {
            Self::Incoming(incoming) => {
                let incoming = incoming.take().unwrap();
                let (parts, incoming) = incoming.into_parts();

                let headers = parts.headers;

                let content_encoding = headers.get_content_encoding()?;

                let body = read_bytes(content_encoding, incoming).await?;

                (parts.uri, headers, body)
            }
            Self::AsBody { body, .. } => match body.as_ref() {
                Some(itm) => {
                    return Ok(Some(itm));
                }
                None => {
                    panic!("You are trying to access body content after receiving ownership of it")
                }
            },
            Self::Taken => {
                panic!("Body is taken by some middleware before")
            }
        };

        let content_type = headers.try_get_case_insensitive("content-type");

        let content_type = match content_type {
            Some(content_type) => Some(content_type.as_str()?.to_string()),
            None => None,
        };

        let body = HttpRequestBody::new(bytes, content_type)?;
        *self = Self::AsBody {
            body: Some(body),
            uri,
            headers,
        };
        Ok(self.try_unwrap_as_body())
    }

    pub async fn receive_body(&mut self) -> Result<HttpRequestBody, HttpFailResult> {
        match self {
            Self::Incoming(incoming) => {
                let incoming = incoming.take().unwrap();

                let content_encoding = incoming.headers().get_content_encoding()?;

                let bytes = read_bytes(content_encoding, incoming).await?;

                let body = HttpRequestBody::new(bytes, None)?;
                return Ok(body);
            }
            Self::AsBody { body, .. } => match body.take() {
                Some(itm) => {
                    return Ok(itm);
                }
                None => {
                    panic!("You are trying to receive body for a second time")
                }
            },
            Self::Taken => {
                panic!("Body is taken by some middleware before")
            }
        };
    }

    fn try_unwrap_as_body(&self) -> Option<&HttpRequestBody> {
        match self {
            Self::Incoming(_) => None,
            Self::AsBody { body, .. } => {
                let body = body.as_ref()?;
                return Some(body);
            }
            Self::Taken => {
                panic!("Body is taken by some middleware before")
            }
        }
    }

    pub fn uri(&self) -> &Uri {
        match self {
            Self::Incoming(incoming) => incoming.as_ref().unwrap().uri(),
            Self::AsBody { uri, .. } => uri,
            Self::Taken => {
                panic!("Body is taken by some middleware before")
            }
        }
    }

    pub fn headers(&self) -> &impl HttpRequestHeaders {
        match self {
            Self::Incoming(incoming) => incoming.as_ref().unwrap().headers(),
            Self::AsBody { headers, .. } => headers,
            Self::Taken => {
                panic!("Body is taken by some middleware before")
            }
        }
    }

    pub fn take_incoming_body(&mut self) -> hyper::Request<hyper::body::Incoming> {
        match self {
            Self::Incoming(incoming) => {
                let result = incoming.take().unwrap();
                *self = Self::Taken;
                result
            }
            Self::AsBody { .. } => {
                panic!("Body is taken by some middleware before")
            }
            Self::Taken => {
                panic!("Body is taken by some middleware before")
            }
        }
    }
}

async fn read_bytes(
    body_compression: ContentEncoding,
    incoming: impl hyper::body::Body<Data = hyper::body::Bytes, Error = hyper::Error>,
) -> Result<Vec<u8>, HttpFailResult> {
    use http_body_util::BodyExt;

    let collected = incoming.collect().await?;
    let bytes = collected.to_bytes();

    body_compression.decompress_if_needed(bytes)
}
