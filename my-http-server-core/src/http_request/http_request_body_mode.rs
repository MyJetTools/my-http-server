use crate::{BodyContentType, ContentEncoding, HttpFailResult, HttpRequestBodyContent};

pub enum HttpRequestBody {
    Incoming {
        incoming: Option<hyper::body::Incoming>,
        content_type: BodyContentType,
    },
    Full(HttpRequestBodyContent),
}

impl HttpRequestBody {
    pub async fn get_http_request_body(
        &mut self,
    ) -> Result<&HttpRequestBodyContent, HttpFailResult> {
        match self {
            HttpRequestBody::Incoming {
                incoming,
                content_type,
            } => {
                let take = incoming.take().unwrap();

                let bytes = read_bytes(ContentEncoding::None, take).await?;
                let body = HttpRequestBodyContent::new(bytes, content_type.clone())?;
                *self = HttpRequestBody::Full(body);
            }
            HttpRequestBody::Full(http_request_body) => return Ok(http_request_body),
        }

        match self {
            HttpRequestBody::Incoming { .. } => {
                panic!("We should never be here")
            }
            HttpRequestBody::Full(http_request_body) => Ok(http_request_body),
        }
    }

    pub async fn into_http_request_body(self) -> Result<HttpRequestBodyContent, HttpFailResult> {
        match self {
            HttpRequestBody::Incoming {
                mut incoming,
                content_type,
            } => {
                let take = incoming.take().unwrap();
                let bytes = read_bytes(ContentEncoding::None, take).await?;
                let body = HttpRequestBodyContent::new(bytes, content_type)?;
                return Ok(body);
            }
            HttpRequestBody::Full(http_request_body) => return Ok(http_request_body),
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
