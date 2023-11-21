use hyper::{HeaderMap, Uri};

use crate::{HttpFailResult, HttpRequestBody, HttpRequestHeaders};

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
        let (uri, headers, bytes, content_type) = match self {
            Self::Incoming(incoming) => {
                let incoming = incoming.take().unwrap();
                let (parts, incoming) = incoming.into_parts();

                let headers = parts.headers;

                let content_type = if let Some(content_type) = headers.get("content-type") {
                    Some(content_type.to_str().unwrap().to_string())
                } else {
                    None
                };
                (
                    parts.uri,
                    headers,
                    read_bytes(incoming).await?,
                    content_type,
                )
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

        let body = HttpRequestBody::new(bytes, content_type);
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
                let bytes = read_bytes(incoming).await?;
                let body = HttpRequestBody::new(bytes, None);
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
    incoming: impl hyper::body::Body<Data = hyper::body::Bytes, Error = hyper::Error>,
) -> Result<Vec<u8>, HttpFailResult> {
    use http_body_util::BodyExt;

    let collected = incoming.collect().await?;
    let bytes = collected.to_bytes();
    Ok(bytes.into())
}
