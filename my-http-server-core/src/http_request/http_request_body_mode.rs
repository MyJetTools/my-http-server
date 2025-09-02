use crate::{ContentEncoding, HttpFailResult, HttpRequestBody};

pub enum HttpRequestBodyMode {
    Incoming(Option<hyper::body::Incoming>),
    Full(HttpRequestBody),
}

impl HttpRequestBodyMode {
    pub async fn get_http_request_body(&mut self) -> Result<&HttpRequestBody, HttpFailResult> {
        match self {
            HttpRequestBodyMode::Incoming(incoming) => {
                let take = incoming.take().unwrap();
                let bytes = read_bytes(ContentEncoding::None, take).await?;
                let body = HttpRequestBody::new(bytes, None)?;
                *self = HttpRequestBodyMode::Full(body);
            }
            HttpRequestBodyMode::Full(http_request_body) => return Ok(http_request_body),
        }

        match self {
            HttpRequestBodyMode::Incoming(_) => {
                panic!("We should never be here")
            }
            HttpRequestBodyMode::Full(http_request_body) => Ok(http_request_body),
        }
    }

    pub async fn into_http_request_body(self) -> Result<HttpRequestBody, HttpFailResult> {
        match self {
            HttpRequestBodyMode::Incoming(mut incoming) => {
                let take = incoming.take().unwrap();
                let bytes = read_bytes(ContentEncoding::None, take).await?;
                let body = HttpRequestBody::new(bytes, None)?;
                return Ok(body);
            }
            HttpRequestBodyMode::Full(http_request_body) => return Ok(http_request_body),
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
