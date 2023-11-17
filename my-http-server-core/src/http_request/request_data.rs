use crate::{HttpFailResult, HttpRequestBody};

pub enum RequestData {
    Incoming(Option<hyper::body::Incoming>),
    AsBody { body: HttpRequestBody },
}

impl RequestData {
    pub async fn convert_to_body_if_requires(
        &mut self,
    ) -> Result<Option<&HttpRequestBody>, HttpFailResult> {
        let bytes = match self {
            Self::Incoming(incoming) => {
                let incoming = incoming.take().unwrap();
                read_bytes(incoming).await?
            }
            Self::AsBody { body } => {
                return Ok(Some(body));
            }
        };

        let body = HttpRequestBody::new(bytes, None);
        *self = Self::AsBody { body };
        Ok(self.try_unwrap_as_body())
    }

    fn try_unwrap_as_body(&self) -> Option<&HttpRequestBody> {
        match self {
            Self::Incoming(_) => None,
            Self::AsBody { body } => {
                return Some(body);
            }
        }
    }

    pub fn unwrap_as_request(&self) -> &hyper::Request<hyper::body::Incoming> {
        todo!("Waiting for WebSockets implementation");
    }

    pub fn try_unwrap_as_request(&self) -> &Option<hyper::Request<hyper::body::Incoming>> {
        todo!("Waiting for WebSockets implementation");
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
